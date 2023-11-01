extern crate chip8;
extern crate sdl2;

use chip8::{quirks::Quirks, Chip8};
use log::trace;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use std::io::Read;
use std::time::{Duration, Instant};

const DEFAULT_PROGRAM: &[u8] = include_bytes!("../../roms/1-tests/1-chip8-logo.ch8");

pub fn main() {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    let mut program = Vec::from(DEFAULT_PROGRAM);

    let mut chip8 = Chip8::new();

    let mut set_values: Vec<(u16, u8)> = vec![];
    let mut stepping = false;
    let mut stepping_steps = 0u32;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--chip8" | "--quirks=chip8" => chip8.cpu.quirks = Quirks::chip8(),
            "--superchip" | "--quirks=superchip" => chip8.cpu.quirks = Quirks::superchip(),
            "--xochip" | "--quirks=xochip" => chip8.cpu.quirks = Quirks::xochip(),
            "--stepping" | "-s" => stepping = true,

            set if arg.starts_with("--set=") => {
                // Parse --set=hex:hex, and apply to chip8
                for (s_addr, s_val) in set
                    .split(|c| c == '=' || c == ':' || c == ',')
                    .skip(1)
                    .collect::<Vec<&str>>()
                    .chunks_exact(2)
                    .map(|x| (x[0], x[1]))
                {
                    let addr = u16::from_str_radix(s_addr, 16).expect(&format!(
                        "Failed to parse address of set {}:{}",
                        s_addr, s_val
                    ));
                    let val = u8::from_str_radix(s_val, 16).expect(&format!(
                        "Failed to parse value of set {}:{}",
                        s_addr, s_val
                    ));

                    set_values.push((addr, val));
                }
            }

            // --break=feed,123,42
            bp if arg.starts_with("--break=") => {
                for s_addr in bp.split(|c| c == '=' || c == ',').skip(1) {
                    let addr = u16::from_str_radix(s_addr, 16)
                        .expect(&format!("Failed to parse breakpoint {}", s_addr));

                    chip8.set_breakpoint(addr);
                }
            }

            filename if filename.ends_with(".ch8") => {
                program.truncate(0);
                std::fs::File::open(filename)
                    .expect(format!("Unable to read {}", filename).as_str())
                    .read_to_end(&mut program)
                    .expect("Buffer overflow");
            }

            x => {
                panic!("Invalid argument {}", x)
            }
        }
    }

    chip8.load_program(&program);
    for (addr, val) in set_values {
        chip8.cpu.memory.write(addr, val)
    }

    println!("Chip8 running!");
    println!("  [J] to step through instructions");
    println!("  [K] disables stepping");
    println!("  [L] continues after a breakpoint");

    let sdl_context = sdl2::init().unwrap();
    let sdl_video = sdl_context.video().unwrap();

    let mut canvas = sdl_video
        .window("rschip8", 640, 320)
        .position_centered()
        .build()
        .unwrap()
        .into_canvas()
        .build()
        .unwrap();

    let black = Color::RGB(0, 0, 0);
    let white = Color::RGB(255, 255, 255);

    canvas.set_draw_color(black);
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let desired_frame_time = Duration::from_secs_f64(1.0 / 60.0);

    'running: loop {
        let start_time = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Num1 => chip8.keydown(0x1),
                    Keycode::Num2 => chip8.keydown(0x2),
                    Keycode::Num3 => chip8.keydown(0x3),
                    Keycode::Num4 => chip8.keydown(0xc),
                    Keycode::Q => chip8.keydown(0x4),
                    Keycode::W => chip8.keydown(0x5),
                    Keycode::E => chip8.keydown(0x6),
                    Keycode::R => chip8.keydown(0xd),
                    Keycode::A => chip8.keydown(0x7),
                    Keycode::S => chip8.keydown(0x8),
                    Keycode::D => chip8.keydown(0x9),
                    Keycode::F => chip8.keydown(0xe),
                    Keycode::Z => chip8.keydown(0xa),
                    Keycode::X => chip8.keydown(0x0),
                    Keycode::C => chip8.keydown(0xb),
                    Keycode::V => chip8.keydown(0xf),

                    #[cfg(debug_assertions)]
                    Keycode::M => {
                        println!("CPU: {}", chip8.cpu);
                        println!("");
                    }

                    #[cfg(debug_assertions)]
                    Keycode::P => {
                        println!("{}", "-".repeat(chip8.cpu.width));
                        println!(
                            "{}",
                            chip8
                                .cpu
                                .vram
                                .chunks_exact(chip8.cpu.width)
                                .map(|chunk| {
                                    chunk
                                        .iter()
                                        .map(|v| if *v { "█" } else { " " })
                                        .collect::<Vec<&str>>()
                                        .join("")
                                })
                                .collect::<Vec<String>>()
                                .join("\n")
                        );
                        println!("{}", "-".repeat(chip8.cpu.width));
                    }

                    Keycode::J => {
                        if !stepping {
                            stepping = true;
                            stepping_steps = 0;
                        }
                        stepping_steps += chip8.cpu.step();
                        if stepping_steps >= (chip8.cpu.clock_speed / 60000) {
                            chip8.cpu.tick_timers();
                            stepping_steps -= chip8.cpu.clock_speed / 60000;
                        }
                    }
                    Keycode::K if stepping => stepping = false,
                    Keycode::L if chip8.halted => chip8.resume(),

                    _ => {}
                },

                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Num1 => chip8.keyup(0x1),
                    Keycode::Num2 => chip8.keyup(0x2),
                    Keycode::Num3 => chip8.keyup(0x3),
                    Keycode::Num4 => chip8.keyup(0xc),
                    Keycode::Q => chip8.keyup(0x4),
                    Keycode::W => chip8.keyup(0x5),
                    Keycode::E => chip8.keyup(0x6),
                    Keycode::R => chip8.keyup(0xd),
                    Keycode::A => chip8.keyup(0x7),
                    Keycode::S => chip8.keyup(0x8),
                    Keycode::D => chip8.keyup(0x9),
                    Keycode::F => chip8.keyup(0xe),
                    Keycode::Z => chip8.keyup(0xa),
                    Keycode::X => chip8.keyup(0x0),
                    Keycode::C => chip8.keyup(0xb),
                    Keycode::V => chip8.keyup(0xf),

                    _ => {}
                },

                _ => {}
            }
        }

        let tick_start_time = Instant::now();
        if !stepping {
            chip8.tick();
        }
        let tick_elapsed = Instant::now() - tick_start_time;

        canvas.set_draw_color(black);
        canvas.clear();

        canvas.set_draw_color(white);
        let pixel_width = 640 / chip8.cpu.width;
        let pixel_height = 320 / chip8.cpu.height;

        let blit_start_time = Instant::now();
        for x in 0..chip8.cpu.width {
            for y in 0..chip8.cpu.height {
                if chip8.cpu.vram[chip8.cpu.width * y + x] {
                    canvas
                        .fill_rect(Rect::new(
                            (x * pixel_width) as i32,
                            (y * pixel_height) as i32,
                            640 / chip8.cpu.width as u32,
                            320 / chip8.cpu.height as u32,
                        ))
                        .unwrap();
                }
            }
        }
        let blit_elapsed = Instant::now() - blit_start_time;

        canvas.present();

        let elapsed = Instant::now() - start_time;

        // canvas
        //     .window_mut()
        //     .set_title(&format!("Chip8 :: {:?}ms / {:?}ms", tick_elapsed, elapsed,))
        //     .expect("Failed to set window title");

        trace!(
            "Timings: tick {:?}, draw {:?}, total {:?}",
            tick_elapsed,
            blit_elapsed,
            elapsed
        );

        if elapsed < desired_frame_time {
            ::std::thread::sleep(desired_frame_time - elapsed);
        }
    }
}
