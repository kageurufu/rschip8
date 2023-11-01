use std::collections::HashSet;

use super::cpu::CPU;
use log::{self, info};

pub struct Chip8 {
    pub cpu: CPU,

    pub halted: bool,

    breakpoints: HashSet<u16>,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            cpu: CPU::new(),
            breakpoints: HashSet::new(),
            halted: false,
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.cpu.memory.load_program(program);
    }

    pub fn tick(&mut self) {
        if !self.cpu.running || self.halted {
            return;
        }

        // 1_000_000 / 600 =
        let max_cycles = self.cpu.clock_speed / 6000; // Cycles to run per tick
        let mut cycles = 0;

        while self.cpu.running && cycles < max_cycles {
            cycles += self.cpu.step();

            if self.breakpoints.contains(&self.cpu.pc) {
                info!("Breakpoint hit at {}", self.cpu.pc);
                self.halted = true;
                break;
            }

            if cycles >= max_cycles {
                break;
            }
        }

        self.cpu.tick_timers();
    }

    pub fn keydown(&mut self, key: u8) {
        self.cpu.keydown(key);
    }

    pub fn keyup(&mut self, key: u8) {
        self.cpu.keyup(key);
    }

    pub fn resume(&mut self) {
        self.halted = false;
    }

    pub fn set_breakpoint(&mut self, addr: u16) {
        self.breakpoints.insert(addr);
    }

    pub fn remove_breakpoint(&mut self, addr: u16) {
        self.breakpoints.remove(&addr);
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{quirks::Quirks, Chip8};

    macro_rules! assert_vram_matches {
        ($chip8:expr, $expected_results:expr) => {
            let lhs = $chip8
                .cpu
                .vram
                .chunks_exact($chip8.cpu.width)
                .map(|r| {
                    r.iter()
                        .map(|v| if *v { "█" } else { " " })
                        .collect::<Vec<&str>>()
                        .join("")
                })
                .collect::<Vec<String>>()
                .join("\n");

            assert_eq!(
                lhs, $expected_results,
                "Chip8 display does not match the expected results"
            );
        };
    }

    fn run_until_finished(chip8: &mut Chip8, cycles: u32) -> Result<u32, String> {
        for i in 0..cycles {
            chip8.tick();

            if !chip8.cpu.running {
                return Ok(i);
            }
        }

        Err(format!("Did not exit in {} cycles", cycles))
    }

    macro_rules! test_roms {
        ($($func_name:ident: $rom_name:expr,)*)=>{
        $(
            #[test]
            pub fn $func_name() {
                const PROGRAM: &[u8] = include_bytes!(concat!("test_data/", $rom_name, ".ch8"));
                const RESULTS: &str = include_str!(concat!("test_data/", $rom_name, ".txt"));

                let mut c = Chip8::new();
                c.load_program(PROGRAM);

                run_until_finished(&mut c, 1000).expect("Did not finish");
                assert_vram_matches!(c, RESULTS);
            }
        )*
        };
    }

    test_roms! {
        chip8_logo: "1-chip8-logo",
        ibm_logo: "2-ibm-logo",
        corax: "3-corax+",
        flags: "4-flags",

        ibm_logo_shift_down: "ibm-logo-shift-down",
        ibm_logo_shift_left: "ibm-logo-shift-left",
        ibm_logo_shift_right: "ibm-logo-shift-right",
    }

    macro_rules! quirks_test {
        ($($func_name:ident: ($quirk:ident, $mode_value:expr),)*)=>{
            $(
                #[test]
                pub fn $func_name() {
                    let mut c = Chip8::new();
                    c.cpu.quirks = Quirks::$quirk();
                    c.load_program(include_bytes!("test_data/5-quirks.ch8"));
                    c.cpu.memory.write(0x1ff, $mode_value); // Set chip8 mode
                    c.set_breakpoint(0x05d2);

                    for _ in 0..1000 {
                        c.tick();
                        if c.halted {
                            break;
                        }
                    }

                    assert_vram_matches!(c, include_str!(concat!("test_data/5-quirks-", stringify!($quirk), ".txt")));
                }
            )*
        };
    }

    quirks_test! {
        quirks_chip8: (chip8, 1),
        quirks_superchip: (superchip, 2),
        quirks_xochip: (xochip, 3),
    }

    macro_rules! hires_quirks_test {
        ($($func_name:ident: ($quirk:ident, $mode_value:expr),)*)=>{
            $(
                #[test]
                pub fn $func_name() {
                    let mut c = Chip8::new();
                    c.cpu.quirks = Quirks::$quirk();
                    c.load_program(include_bytes!("test_data/7-hires-quirks.ch8"));
                    c.cpu.memory.write(0x1ff, $mode_value); // Set chip8 mode
                    c.set_breakpoint(0x05d8);

                    for _ in 0..1000 {
                        c.tick();
                        if c.halted {
                            break;
                        }
                    }

                    assert_vram_matches!(c, include_str!(concat!("test_data/7-hires-quirks-", stringify!($quirk), ".txt")));
                }
            )*
        };
    }

    hires_quirks_test! {
        hires_quirks_chip8: (chip8, 1),
        hires_quirks_superchip: (superchip, 2),
        hires_quirks_xochip: (xochip, 3),
    }
}
