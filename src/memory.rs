use serde::Serialize;

pub const FONT_5_5: [u8; 0x50] = //include_bytes!("data/rom.bin");
    [
        0xf0, 0x90, 0x90, 0x90, 0xf0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xf0, 0x10, 0xf0, 0x80, 0xf0, // 2
        0xf0, 0x10, 0xf0, 0x10, 0xf0, // 3
        0x90, 0x90, 0xf0, 0x10, 0x10, // 4
        0xf0, 0x80, 0xf0, 0x10, 0xf0, // 5
        0xf0, 0x80, 0xf0, 0x90, 0xf0, // 6
        0xf0, 0x10, 0x20, 0x40, 0x40, // 7
        0xf0, 0x90, 0xf0, 0x90, 0xf0, // 8
        0xf0, 0x90, 0xf0, 0x10, 0xf0, // 9
        0xf0, 0x90, 0xf0, 0x90, 0x90, // A
        0xe0, 0x90, 0xe0, 0x90, 0xe0, // B
        0xf0, 0x80, 0x80, 0x80, 0xf0, // C
        0xe0, 0x90, 0x90, 0x90, 0xe0, // D
        0xf0, 0x80, 0xf0, 0x80, 0xf0, // E
        0xf0, 0x80, 0xf0, 0x80, 0x80, // F
    ];

pub const FONT_10_10: [u8; 0xA0] = [
    0x3C, 0x7E, 0xE7, 0xC3, 0xC3, 0xC3, 0xC3, 0xE7, 0x7E, 0x3C, // 0
    0x18, 0x38, 0x58, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, // 1
    0x3E, 0x7F, 0xC3, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xFF, 0xFF, // 2
    0x3C, 0x7E, 0xC3, 0x03, 0x0E, 0x0E, 0x03, 0xC3, 0x7E, 0x3C, // 3
    0x06, 0x0E, 0x1E, 0x36, 0x66, 0xC6, 0xFF, 0xFF, 0x06, 0x06, // 4
    0xFF, 0xFF, 0xC0, 0xC0, 0xFC, 0xFE, 0x03, 0xC3, 0x7E, 0x3C, // 5
    0x3E, 0x7C, 0xE0, 0xC0, 0xFC, 0xFE, 0xC3, 0xC3, 0x7E, 0x3C, // 6
    0xFF, 0xFF, 0x03, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x60, 0x60, // 7
    0x3C, 0x7E, 0xC3, 0xC3, 0x7E, 0x7E, 0xC3, 0xC3, 0x7E, 0x3C, // 8
    0x3C, 0x7E, 0xC3, 0xC3, 0x7F, 0x3F, 0x03, 0x03, 0x3E, 0x7C, // 9
    // hex chars from octo font
    0x7E, 0xFF, 0xC3, 0xC3, 0xC3, 0xFF, 0xFF, 0xC3, 0xC3, 0xC3, // A
    0xFC, 0xFC, 0xC3, 0xC3, 0xFC, 0xFC, 0xC3, 0xC3, 0xFC, 0xFC, // B
    0x3C, 0xFF, 0xC3, 0xC0, 0xC0, 0xC0, 0xC0, 0xC3, 0xFF, 0x3C, // C
    0xFC, 0xFE, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xFE, 0xFC, // D
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, // E
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC0, 0xC0, 0xC0, 0xC0, // F
];

#[derive(Debug, Serialize)]
pub struct Memory {
    #[serde(serialize_with = "<[_]>::serialize")]
    pub memory: [u8; 0x1000],
}

impl Default for Memory {
    fn default() -> Memory {
        Memory::new()
    }
}

impl Memory {
    pub fn new() -> Memory {
        let mut m = Memory {
            memory: [0; 0x1000],
        };
        m.reset();
        m
    }

    pub fn reset(&mut self) {
        self.memory.fill(0);
        self.memory[0x000..0x050].copy_from_slice(&FONT_5_5);
        self.memory[0x050..0x0F0].copy_from_slice(&FONT_10_10);
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.memory[0x200..(0x200 + program.len())].copy_from_slice(&program);
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.memory[(addr & 0xfff) as usize]
    }

    pub fn write(&mut self, addr: u16, byte: u8) {
        self.memory[(addr & 0xfff) as usize] = byte;
    }
}

#[cfg(test)]
mod tests {
    use super::{Memory, FONT_10_10, FONT_5_5};

    #[test]
    fn memory_contains_5x5_font_at_0x00() {
        let m = Memory::new();

        assert_eq!(m.memory[0..0x050], FONT_5_5);
    }

    #[test]
    fn memory_contains_10x10_font_at_0x50() {
        let m = Memory::new();

        assert_eq!(m.memory[0x050..0x0F0], FONT_10_10);
    }

    #[test]
    fn memory_writes_work() {
        let mut m = Memory::new();

        m.write(0x200, 0xff);
        assert_eq!(m.memory[0x200], 0xff);

        m.write(0x1200, 0xcc);
        assert_eq!(m.memory[0x200], 0xcc);
    }

    #[test]
    fn memory_reads_work() {
        let mut m = Memory::new();

        m.memory[0x200] = 0xff;

        assert_eq!(
            m.read(0x200),
            0xff,
            "Testing that memory reads the right address"
        );
        assert_eq!(
            m.read(0x1200),
            0xff,
            "Testing that memory reads wrap the address"
        );
    }

    #[test]
    fn memory_loads_programs() {
        let mut m = Memory::new();
        m.load_program(&[0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);

        assert_eq!(
            m.memory[0x200..0x209],
            [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x00]
        );
    }
}