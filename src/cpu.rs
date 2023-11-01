use core::fmt;
use log::{debug, info};
use rand::Rng;
use serde::Serialize;

use super::{instruction::Instruction, memory::Memory, quirks::Quirks};

#[derive(Default, Serialize)]
pub struct CPU {
    pub quirks: Quirks,
    pub clock_speed: u32,

    pub running: bool,
    pub hires: bool,

    pub memory: Memory,
    pub keys: [bool; 16],

    pub pc: u16,
    stack: Vec<u16>,

    vx: [u8; 16],
    dt: u8,
    st: u8,
    i: u16,

    save: [u8; 8],

    pub width: usize,
    pub height: usize,
    pub vram: Vec<bool>,
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CPU(pc=${:04x} i={:04x} dt=${:02x} st=${:02x} sp={})",
            self.pc,
            self.i,
            self.dt,
            self.st,
            self.stack.len(),
        )
    }
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            quirks: Quirks::default(),
            clock_speed: 1_000_000, // MHz

            running: true,
            hires: false,

            memory: Memory::new(),
            keys: [false; 16],

            pc: 0x200,
            i: 0,

            vx: [0; 16],
            dt: 0,
            st: 0,
            stack: vec![],

            save: [0; 8],

            width: 64,
            height: 32,
            vram: vec![false; 64 * 32],
        }
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn keydown(&mut self, key: u8) {
        self.keys[(key & 0xf) as usize] = true;
    }

    pub fn keyup(&mut self, key: u8) {
        self.keys[(key & 0xf) as usize] = false;
    }

    fn push(&mut self, v: u16) {
        self.stack.push(v);
    }

    fn pop(&mut self) -> u16 {
        self.stack.pop().unwrap_or(0)
    }

    pub fn step(&mut self) -> u32 {
        let op: u16 =
            ((self.memory.read(self.pc) as u16) << 8) + (self.memory.read(self.pc + 1) as u16);

        let inst = Instruction::parse(op);

        debug!("{} {:?}", self, inst);

        self.pc += 2;

        let cycles = self.execute(inst);

        match (inst, self.quirks.display_wait) {
            (Instruction::DRW_Vx_Vy_n(_, _, _), true) => self.clock_speed / 6000,
            _ => cycles,
        }
    }

    pub fn tick_timers(&mut self) {
        if self.st > 0 {
            self.st -= 1;
        }

        if self.dt > 0 {
            self.dt -= 1;
        }
    }

    pub fn execute(&mut self, inst: Instruction) -> u32 {
        #[allow(unused_variables)]
        match inst {
            Instruction::SYS_addr(_) => {}
            Instruction::EXIT => {
                self.running = false;
                self.stop();
            }

            Instruction::RET => {
                self.pc = self.pop();
            }

            Instruction::JP_addr(addr) => {
                if self.pc == addr + 2 {
                    info!("Infinite loop detected at {}, halting", addr);
                    self.running = false;
                }

                self.pc = addr;
            }

            Instruction::JP_Vx_addr(x, addr) => {
                let vx: usize = if self.quirks.jumping { x as usize } else { 0 };
                self.pc = addr + (self.vx[vx] as u16);
            }

            Instruction::CALL_addr(addr) => {
                self.push(self.pc);
                self.pc = addr;
            }

            Instruction::LD_Vx_kk(x, kk) => self.vx[x as usize] = kk,
            Instruction::LD_Vx_Vy(x, y) => self.vx[x as usize] = self.vx[y as usize],
            Instruction::LD_I_addr(addr) => self.i = addr,

            Instruction::LD_DT_Vx(x) => self.dt = self.vx[x as usize],
            Instruction::LD_ST_Vx(x) => self.st = self.vx[x as usize],
            Instruction::LD_Vx_DT(x) => self.vx[x as usize] = self.dt,

            Instruction::LD_iI_Vx(x) => {
                for i in 0..(x + 1) {
                    self.memory.write(self.i + (i as u16), self.vx[i as usize])
                }
                if self.quirks.memory {
                    self.i += (x as u16) + 1
                }
            }
            Instruction::LD_Vx_iI(x) => {
                for i in 0..(x + 1) {
                    self.vx[i as usize] = self.memory.read(self.i + (i as u16))
                }
                if self.quirks.memory {
                    self.i += (x as u16) + 1
                }
            }

            Instruction::LD_B_Vx(x) => {
                self.memory
                    .write(self.i + 0, (self.vx[x as usize] / 100) % 10);
                self.memory
                    .write(self.i + 1, (self.vx[x as usize] / 10) % 10);
                self.memory.write(self.i + 2, (self.vx[x as usize]) % 10);
            }

            Instruction::ADD_Vx_kk(x, kk) => {
                let (result, _) = self.vx[x as usize].overflowing_add(kk);
                self.vx[x as usize] = result;
            }
            Instruction::ADD_I_Vx(x) => {
                self.i += self.vx[x as usize] as u16;
                self.vx[0xf] = if self.i > 0x0fff { 1 } else { 0 };
                self.i &= 0x0fff;
            }
            Instruction::ADD_Vx_Vy(x, y) => {
                let (result, overflow) = self.vx[x as usize].overflowing_add(self.vx[y as usize]);

                self.vx[x as usize] = result;
                self.vx[0xf] = if overflow { 1 } else { 0 };
            }

            Instruction::SE_Vx_kk(x, kk) => {
                if self.vx[x as usize] == kk {
                    self.pc += 2;
                }
            }
            Instruction::SNE_Vx_kk(x, kk) => {
                if self.vx[x as usize] != kk {
                    self.pc += 2;
                }
            }

            Instruction::SE_Vx_Vy(x, y) => {
                if self.vx[x as usize] == self.vx[y as usize] {
                    self.pc += 2
                }
            }
            Instruction::SNE_Vx_Vy(x, y) => {
                if self.vx[x as usize] != self.vx[y as usize] {
                    self.pc += 2
                }
            }

            Instruction::AND_Vx_Vy(x, y) => {
                self.vx[x as usize] &= self.vx[y as usize];
                if self.quirks.vf_reset {
                    self.vx[0xf] = 0;
                }
            }
            Instruction::OR_Vx_Vy(x, y) => {
                self.vx[x as usize] |= self.vx[y as usize];
                if self.quirks.vf_reset {
                    self.vx[0xf] = 0;
                }
            }
            Instruction::XOR_Vx_Vy(x, y) => {
                self.vx[x as usize] ^= self.vx[y as usize];
                if self.quirks.vf_reset {
                    self.vx[0xf] = 0;
                }
            }

            Instruction::SUB_Vx_Vy(x, y) => {
                let (result, borrow) = self.vx[x as usize].overflowing_sub(self.vx[y as usize]);
                self.vx[x as usize] = result;
                self.vx[0xf] = if borrow { 0 } else { 1 }
            }
            Instruction::SUBN_Vx_Vy(x, y) => {
                let (result, borrow) = self.vx[y as usize].overflowing_sub(self.vx[x as usize]);
                self.vx[x as usize] = result;
                self.vx[0xf] = if borrow { 0 } else { 1 }
            }

            Instruction::SHL_Vx_Vy(x, y) => {
                if !self.quirks.shifting {
                    self.vx[x as usize] = self.vx[y as usize]
                }

                let overflow = (self.vx[x as usize] & 0b10000000) > 0;
                self.vx[x as usize] <<= 1;
                self.vx[0xf] = if overflow { 1 } else { 0 }
            }
            Instruction::SHR_Vx_Vy(x, y) => {
                if !self.quirks.shifting {
                    self.vx[x as usize] = self.vx[y as usize]
                }
                let underflow = (self.vx[x as usize] & 0b00000001) > 0;
                self.vx[x as usize] >>= 1;
                self.vx[0xf] = if underflow { 1 } else { 0 }
            }

            /* Input Opcodes */
            Instruction::LD_Vx_K(x) => {
                if let Some(res) = self.keys.iter().position(|v| *v) {
                    self.vx[x as usize] = res as u8;
                } else {
                    self.pc -= 2;
                }
            }
            Instruction::SKP_Vx(x) => {
                if self.keys[(self.vx[x as usize] & 0xf) as usize] {
                    self.pc += 2;
                }
            }
            Instruction::SKNP_Vx(x) => {
                if !self.keys[(self.vx[x as usize] & 0xf) as usize] {
                    self.pc += 2;
                }
            }

            Instruction::RND_Vx_kk(x, kk) => {
                self.vx[x as usize] = rand::thread_rng().gen_range(0..=kk);
            }

            Instruction::LD_F_Vx(x) => {
                self.i = 5 * (self.vx[x as usize] as u16);
            }
            Instruction::LD_HF_Vx(x) => {
                self.i = 0x050 + (10 * (self.vx[x as usize] as u16));
            }

            Instruction::CLS => {
                self.vram.fill(false);
            }

            Instruction::LORES => {
                self.hires = false;
                self.width = 64;
                self.height = 32;
                self.vram.resize(64 * 32, false)
            }

            Instruction::HIRES => {
                self.hires = true;
                self.width = 128;
                self.height = 64;
                self.vram.resize(128 * 64, false)
            }

            Instruction::SCD_n(n) => {
                // Scroll down, need to verify expected wrapping behavior
                self.vram.copy_within(
                    // First (h-n) rows
                    0..((self.height - n as usize) * self.width),
                    (n as usize) * self.width,
                );
                self.vram[0..((n as usize) * self.width)].fill(false);
            }

            Instruction::SCR => {
                // for i in 1..self.height {
                //     self.vram.copy_within(0..(self.width - 4), 4)
                // }
                // For each row, offset right by 4 pixels
                for row in self.vram.chunks_mut(self.width) {
                    row.copy_within(..(self.width - 4), 4);
                    row[..4].fill(false);
                }
            }

            Instruction::SCL => {
                // For each row, offset left by 8 pixels
                for row in self.vram.chunks_mut(self.width) {
                    row.copy_within(4.., 0);
                    row[(self.width - 4)..].fill(false);
                }
            }

            Instruction::DRW_Vx_Vy_n(vx, vy, n) => {
                let mut x: usize = self.vx[vx as usize] as usize;
                let mut y: usize = self.vx[vy as usize] as usize;

                if x >= self.width {
                    x = x % self.width;
                }
                if y >= self.height {
                    y = y % self.height;
                }

                self.vx[0xf] = 0;

                if n == 0 {
                    // 16x16 sprite!
                    for row in 0..16 {
                        if (y + row) >= self.height {
                            self.vx[0xf] += 1;

                            if !self.quirks.sprite_wrapping {
                                break;
                            }
                        }

                        let row_offset = if (y + row) >= self.height {
                            self.width * ((y + row) as usize - self.height)
                        } else {
                            self.width * ((y + row) as usize)
                        };

                        let bits = (self.memory.read(self.i + (row as u16) * 2) as u16) << 8
                            | (self.memory.read(self.i + (row as u16) * 2 + 1) as u16);

                        let mut clobber = false;
                        for col in 0..16 {
                            if (x + col) >= self.width && !self.quirks.sprite_wrapping {
                                break;
                            }

                            let offset = if (x + col) >= self.width {
                                row_offset + x + col - self.width
                            } else {
                                row_offset + x + col
                            };

                            if bits & (1 << (15 - col)) > 0 {
                                if self.vram[offset] {
                                    clobber = true;
                                }

                                self.vram[offset] = !self.vram[offset];
                            }
                        }

                        if clobber {
                            self.vx[0xf] += 1;
                        }
                    }
                } else {
                    for row in 0..(n as usize) {
                        if (y + row) >= self.height {
                            if !self.quirks.sprite_wrapping {
                                break;
                            }
                        }

                        let row_offset = if (y + row) >= self.height {
                            self.width * ((y + row) as usize - self.height)
                        } else {
                            self.width * ((y + row) as usize)
                        };

                        let bits = self.memory.read(self.i + (row as u16));

                        for col in 0..8 {
                            if (x + col) >= self.width && !self.quirks.sprite_wrapping {
                                break;
                            }

                            let offset = if (x + col) >= self.width {
                                row_offset + x + col - self.width
                            } else {
                                row_offset + x + col
                            };

                            if bits & (1 << (7 - col)) > 0 {
                                if self.vram[offset] {
                                    self.vx[0xf] = 1;
                                }

                                self.vram[offset] = !self.vram[offset];
                            }
                        }
                    }
                }
            }

            Instruction::SAVE_Vx(x) => {
                for i in 0..(x.min(7) + 1) {
                    self.save[i as usize] = self.vx[i as usize];
                }
            }

            Instruction::LOAD_Vx(x) => {
                for i in 0..(x.min(7) + 1) {
                    self.vx[i as usize] = self.save[i as usize];
                }
            }
        }

        8
    }
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use crate::instruction::Instruction;

    use super::CPU;

    #[test]
    pub fn test_SYS_addr() {
        let mut cpu = CPU::new();
        let inst = Instruction::SYS_addr(0);
        cpu.execute(inst);
    }

    #[test]
    pub fn test_EXIT() {
        let mut cpu = CPU::new();
        let inst = Instruction::EXIT;

        cpu.execute(inst);
    }

    #[test]
    pub fn test_CLS() {
        let mut cpu = CPU::new();
        let inst = Instruction::CLS;
        cpu.vram.fill(true);
        cpu.execute(inst);
        assert_eq!(cpu.vram[0], false, "VRAM should be filled with `false`");
    }

    #[test]
    pub fn test_RET() {
        let mut cpu = CPU::new();
        let inst = Instruction::RET;
        cpu.push(0x444);

        assert_eq!(cpu.pc, 0x200, "PC should default to 0x200");
        cpu.execute(inst);
        assert_eq!(cpu.pc, 0x444, "PC should be the top stack value, 0x444");
    }

    #[test]
    pub fn test_JP_addr() {
        let mut cpu = CPU::new();
        let inst = Instruction::JP_addr(0x444);

        cpu.execute(inst);
        assert_eq!(cpu.pc, 0x444, "PC should be set to the jump address");
        assert_eq!(
            cpu.stack,
            vec![],
            "The previous address should not be on the stack"
        );
    }

    #[test]
    pub fn test_JP_Vx_addr() {
        let mut cpu = CPU::new();
        let inst = Instruction::JP_Vx_addr(2, 0x444);
        cpu.quirks.jumping = false;

        cpu.vx[0] = 2;
        cpu.vx[2] = 4;

        cpu.execute(inst);

        assert_eq!(cpu.pc, 0x446, "JP_Vx_addr should jump to `V0 + addr`");
    }

    #[test]
    pub fn test_JP_Vx_addr_quirk() {
        let mut cpu = CPU::new();
        let inst = Instruction::JP_Vx_addr(2, 0x444);
        cpu.quirks.jumping = true;

        cpu.vx[0] = 2;
        cpu.vx[2] = 4;

        cpu.execute(inst);

        assert_eq!(
            cpu.pc, 0x448,
            "[QUIRK] JP_Vx_addr should jump to `Vx + addr`"
        );
    }

    #[test]
    pub fn test_CALL_addr() {
        let mut cpu = CPU::new();
        let inst = Instruction::CALL_addr(0x444);

        cpu.execute(inst);
        assert_eq!(cpu.pc, 0x444, "PC should be set to the subroutine address");
        assert_eq!(
            cpu.stack,
            vec![0x200],
            "The previous address should be on the stack"
        );
    }

    // #[test]
    // pub fn test_SE_Vx_kk() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::SE_Vx_kk(u8, u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_SNE_Vx_kk() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::SNE_Vx_kk(u8, u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_SE_Vx_Vy() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::SE_Vx_Vy(u8, u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_LD_Vx_kk() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::LD_Vx_kk(u8, u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_ADD_Vx_kk() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::ADD_Vx_kk(u8, u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_LD_Vx_Vy() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::LD_Vx_Vy(u8, u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_OR_Vx_Vy() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::OR_Vx_Vy(u8, u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_AND_Vx_Vy() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::AND_Vx_Vy(u8, u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_XOR_Vx_Vy() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::XOR_Vx_Vy(u8, u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_ADD_Vx_Vy() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::ADD_Vx_Vy(u8, u8);
    //     cpu.execute(inst);
    // }
    #[test]
    pub fn test_SUB_Vx_Vy() {
        let mut cpu = CPU::new();

        cpu.vx[0] = 0x66;
        cpu.vx[1] = 0x44;

        cpu.vx[6] = 0x1;
        cpu.vx[7] = 0x2;

        cpu.execute(Instruction::SUB_Vx_Vy(0, 1));
        assert_eq!(cpu.vx[0x0], 0x22);
        assert_eq!(cpu.vx[0x1], 0x44);
        assert_eq!(cpu.vx[0xF], 1);

        cpu.execute(Instruction::SUB_Vx_Vy(6, 7));
        assert_eq!(cpu.vx[0x6], 0xff);
        assert_eq!(cpu.vx[0x7], 0x2);
        assert_eq!(cpu.vx[0xF], 0);
    }
    #[test]
    pub fn test_SUBN_Vx_Vy() {
        let mut c = CPU::new();

        c.vx[0] = 0x44;
        c.vx[1] = 0x66;

        c.vx[6] = 140;
        c.vx[7] = 120;

        c.execute(Instruction::SUBN_Vx_Vy(0, 1));

        assert_eq!(c.vx[0x0], 0x22);
        assert_eq!(c.vx[0x1], 0x66);
        assert_eq!(c.vx[0xF], 1);

        c.execute(Instruction::SUBN_Vx_Vy(6, 7));

        assert_eq!(c.vx[0x6], 236);
        assert_eq!(c.vx[0x7], 120);
        assert_eq!(c.vx[0xF], 0);
    }

    #[test]
    pub fn test_SHR_Vx_Vy() {
        let mut cpu = CPU::new();

        cpu.quirks.shifting = false;

        cpu.vx[0] = 0;
        cpu.vx[1] = 0;
        cpu.vx[2] = 0b01111110;
        cpu.vx[3] = 0b11111111;

        cpu.execute(Instruction::SHR_Vx_Vy(0, 0x2));
        assert_eq!(cpu.vx[0x0], 0b00111111);
        assert_eq!(cpu.vx[0x2], 0b01111110);
        assert_eq!(cpu.vx[0xF], 0);

        cpu.execute(Instruction::SHR_Vx_Vy(1, 0x3));
        assert_eq!(cpu.vx[0x1], 0b01111111);
        assert_eq!(cpu.vx[0x3], 0b11111111);
        assert_eq!(cpu.vx[0xF], 1);
    }

    #[test]
    pub fn test_SHR_Vx_Vy_quirk() {
        let mut cpu = CPU::new();
        cpu.quirks.shifting = true;

        cpu.vx[0] = 0b01111110;
        cpu.vx[1] = 0b11111111;

        cpu.execute(Instruction::SHR_Vx_Vy(0, 0xf));
        assert_eq!(cpu.vx[0], 0b00111111);
        assert_eq!(cpu.vx[0xF], 0);

        cpu.execute(Instruction::SHR_Vx_Vy(1, 0xf));
        assert_eq!(cpu.vx[0x1], 0b01111111);
        assert_eq!(cpu.vx[0xF], 1);
    }

    #[test]
    pub fn test_SHL_Vx_Vy() {
        let mut cpu = CPU::new();
        cpu.quirks.shifting = false;

        cpu.vx[0] = 0;
        cpu.vx[1] = 0;
        cpu.vx[2] = 0b01111110;
        cpu.vx[3] = 0b11111111;

        cpu.execute(Instruction::SHL_Vx_Vy(0, 0x2));
        assert_eq!(cpu.vx[0x0], 0b11111100);
        assert_eq!(cpu.vx[0xF], 0);

        cpu.execute(Instruction::SHL_Vx_Vy(1, 0x3));
        assert_eq!(cpu.vx[0x1], 0b11111110);
        assert_eq!(cpu.vx[0xF], 1);
    }

    #[test]
    pub fn test_SHL_Vx_Vy_quirk() {
        let mut cpu = CPU::new();
        cpu.quirks.shifting = true;

        cpu.vx[0] = 0b01111110;
        cpu.vx[1] = 0b11111111;

        cpu.execute(Instruction::SHL_Vx_Vy(0, 0xf));
        assert_eq!(cpu.vx[0], 0b11111100);
        assert_eq!(cpu.vx[0xF], 0);

        cpu.execute(Instruction::SHL_Vx_Vy(1, 0xf));
        assert_eq!(cpu.vx[0x1], 0b11111110);
        assert_eq!(cpu.vx[0xF], 1);
    }
    // #[test]
    // pub fn test_SNE_Vx_Vy() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::SNE_Vx_Vy(u8, u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_LD_I_addr() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::LD_I_addr(u16);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_RND_Vx_kk() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::RND_Vx_kk(u8, u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_SKP_Vx() {
    //     let mut cpu = CPU::new();

    //     cpu.pc = 0x200;
    //     cpu.vx
    //     cpu.execute(Instruction::SKP_Vx(2));
    // }
    // #[test]
    // pub fn test_SKNP_Vx() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::SKNP_Vx(u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_LD_Vx_DT() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::LD_Vx_DT(u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_LD_Vx_K() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::LD_Vx_K(u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_LD_DT_Vx() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::LD_DT_Vx(u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_LD_ST_Vx() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::LD_ST_Vx(u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_ADD_I_Vx() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::ADD_I_Vx(u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_LD_F_Vx() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::LD_F_Vx(u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_LD_B_Vx() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::LD_B_Vx(u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_LD_iI_Vx() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::LD_iI_Vx(u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_LD_Vx_iI() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::LD_Vx_iI(u8);
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_DRW_Vx_Vy_n() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::DRW_Vx_Vy_n(u8, u8, u8);
    //     cpu.execute(inst);
    // }

    #[test]
    pub fn test_SCD_n() {
        let mut cpu = CPU::new();

        cpu.width = 10;
        cpu.height = 10;
        cpu.vram = vec![false; 100];
        cpu.vram.fill(true);

        cpu.execute(Instruction::SCD_n(2));

        let (chunks, _) = cpu.vram.as_chunks::<10>();
        assert_eq!(
            chunks,
            [
                [false, false, false, false, false, false, false, false, false, false],
                [false, false, false, false, false, false, false, false, false, false],
                [true, true, true, true, true, true, true, true, true, true],
                [true, true, true, true, true, true, true, true, true, true],
                [true, true, true, true, true, true, true, true, true, true],
                [true, true, true, true, true, true, true, true, true, true],
                [true, true, true, true, true, true, true, true, true, true],
                [true, true, true, true, true, true, true, true, true, true],
                [true, true, true, true, true, true, true, true, true, true],
                [true, true, true, true, true, true, true, true, true, true],
            ]
        );
    }

    #[test]
    pub fn test_SCR() {
        let mut cpu = CPU::new();

        cpu.width = 7;
        cpu.height = 7;
        cpu.vram = vec![false; 7 * 7];

        for i in 0..(cpu.width * cpu.height) {
            cpu.vram[i] = if i % 2 == 0 { true } else { false };
        }

        let (chunks, _) = cpu.vram.as_chunks::<7>();
        assert_eq!(
            chunks,
            [
                [true, false, true, false, true, false, true],
                [false, true, false, true, false, true, false],
                [true, false, true, false, true, false, true],
                [false, true, false, true, false, true, false],
                [true, false, true, false, true, false, true],
                [false, true, false, true, false, true, false],
                [true, false, true, false, true, false, true]
            ]
        );

        cpu.execute(Instruction::SCR);
        let (chunks, _) = cpu.vram.as_chunks::<7>();
        assert_eq!(
            chunks,
            [
                [false, false, false, false, true, false, true],
                [false, false, false, false, false, true, false],
                [false, false, false, false, true, false, true],
                [false, false, false, false, false, true, false],
                [false, false, false, false, true, false, true],
                [false, false, false, false, false, true, false],
                [false, false, false, false, true, false, true]
            ]
        );
    }

    #[test]
    pub fn test_SCL() {
        let mut cpu = CPU::new();

        cpu.width = 7;
        cpu.height = 7;
        cpu.vram = vec![false; 7 * 7];

        for i in 0..(cpu.width * cpu.height) {
            cpu.vram[i] = if i % 2 == 0 { true } else { false };
        }

        let (chunks, _) = cpu.vram.as_chunks::<7>();
        assert_eq!(
            chunks,
            [
                [true, false, true, false, true, false, true],
                [false, true, false, true, false, true, false],
                [true, false, true, false, true, false, true],
                [false, true, false, true, false, true, false],
                [true, false, true, false, true, false, true],
                [false, true, false, true, false, true, false],
                [true, false, true, false, true, false, true]
            ]
        );

        cpu.execute(Instruction::SCL);

        let (chunks, _) = cpu.vram.as_chunks::<7>();
        assert_eq!(
            chunks,
            [
                [true, false, true, false, false, false, false],
                [false, true, false, false, false, false, false],
                [true, false, true, false, false, false, false],
                [false, true, false, false, false, false, false],
                [true, false, true, false, false, false, false],
                [false, true, false, false, false, false, false],
                [true, false, true, false, false, false, false]
            ]
        );
    }
    // #[test]
    // pub fn test_SCL() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::SCL;
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_LORES() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::LORES;
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_HIRES() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::HIRES;
    //     cpu.execute(inst);
    // }
    // #[test]
    // pub fn test_LD_HF_Vx() {
    //     let mut cpu = CPU::new();
    //     let inst = Instruction::LD_HF_Vx(u8);
    //     cpu.execute(inst);
    // }

    #[test]
    pub fn test_SAVE_Vx() {
        let mut cpu = CPU::new();

        cpu.vx = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

        cpu.execute(Instruction::SAVE_Vx(2));
        assert_eq!(cpu.save, [0, 1, 2, 0, 0, 0, 0, 0]);

        cpu.execute(Instruction::SAVE_Vx(12));
        assert_eq!(cpu.save, [0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    pub fn test_LOAD_Vx() {
        let mut cpu = CPU::new();
        cpu.save = [0, 1, 2, 3, 4, 5, 6, 7];

        cpu.execute(Instruction::LOAD_Vx(2));
        assert_eq!(cpu.vx, [0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }
}
