#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    SYS_addr(u16),
    CLS,
    RET,
    JP_addr(u16),
    CALL_addr(u16),
    SE_Vx_kk(u8, u8),
    SNE_Vx_kk(u8, u8),
    SE_Vx_Vy(u8, u8),
    LD_Vx_kk(u8, u8),
    ADD_Vx_kk(u8, u8),
    LD_Vx_Vy(u8, u8),
    OR_Vx_Vy(u8, u8),
    AND_Vx_Vy(u8, u8),
    XOR_Vx_Vy(u8, u8),
    ADD_Vx_Vy(u8, u8),
    SUB_Vx_Vy(u8, u8),
    SHR_Vx_Vy(u8, u8),
    SUBN_Vx_Vy(u8, u8),
    SHL_Vx_Vy(u8, u8),
    SNE_Vx_Vy(u8, u8),
    LD_I_addr(u16),
    JP_Vx_addr(u8, u16),
    RND_Vx_kk(u8, u8),
    SKP_Vx(u8),
    SKNP_Vx(u8),
    LD_Vx_DT(u8),
    LD_Vx_K(u8),
    LD_DT_Vx(u8),
    LD_ST_Vx(u8),
    ADD_I_Vx(u8),
    LD_F_Vx(u8),
    LD_B_Vx(u8),
    LD_iI_Vx(u8),
    LD_Vx_iI(u8),
    DRW_Vx_Vy_n(u8, u8, u8),
    SCD_n(u8),
    SCR,
    SCL,
    EXIT,
    LORES,
    HIRES,
    LD_HF_Vx(u8),
    SAVE_Vx(u8),
    LOAD_Vx(u8),
}

impl Instruction {
    pub fn parse(op: u16) -> Instruction {
        let nibbles = (
            ((op & 0xf000) >> 12) as u8,
            ((op & 0x0f00) >> 8) as u8,
            ((op & 0x00f0) >> 4) as u8,
            (op & 0x000f) as u8,
        );

        let nnn = op & 0x0fff;
        let kk = (op & 0x00ff) as u8;

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => Instruction::CLS,
            (0x0, 0x0, 0xE, 0xE) => Instruction::RET,
            (0x0, 0x0, 0xF, 0xB) => Instruction::SCR,
            (0x0, 0x0, 0xF, 0xC) => Instruction::SCL,
            (0x0, 0x0, 0xF, 0xD) => Instruction::EXIT,
            (0x0, 0x0, 0xF, 0xE) => Instruction::LORES,
            (0x0, 0x0, 0xF, 0xF) => Instruction::HIRES,
            (0x0, 0x0, 0xC, n) => Instruction::SCD_n(n),

            // Special case for hires $0230
            (0x0, 0x2, 0x3, 0x0) => Instruction::CLS,

            (0x0, _, _, _) => Instruction::SYS_addr(nnn),

            (0x1, _, _, _) => Instruction::JP_addr(nnn),
            (0x2, _, _, _) => Instruction::CALL_addr(nnn),

            (0x3, x, _, _) => Instruction::SE_Vx_kk(x, kk), // Instruction::SE_Vx_kk(Reg::Vx(x), kk),
            (0x4, x, _, _) => Instruction::SNE_Vx_kk(x, kk),

            (0x5, x, y, 0x0) => Instruction::SE_Vx_Vy(x, y),
            (0x6, x, _, _) => Instruction::LD_Vx_kk(x, kk),
            (0x7, x, _, _) => Instruction::ADD_Vx_kk(x, kk),

            (0x8, x, y, 0x0) => Instruction::LD_Vx_Vy(x, y),

            (0x8, x, y, 0x1) => Instruction::OR_Vx_Vy(x, y),
            (0x8, x, y, 0x2) => Instruction::AND_Vx_Vy(x, y),
            (0x8, x, y, 0x3) => Instruction::XOR_Vx_Vy(x, y),
            (0x8, x, y, 0x4) => Instruction::ADD_Vx_Vy(x, y),
            (0x8, x, y, 0x5) => Instruction::SUB_Vx_Vy(x, y),
            (0x8, x, y, 0x6) => Instruction::SHR_Vx_Vy(x, y),
            (0x8, x, y, 0x7) => Instruction::SUBN_Vx_Vy(x, y),
            (0x8, x, y, 0xe) => Instruction::SHL_Vx_Vy(x, y),
            (0x9, x, y, 0x0) => Instruction::SNE_Vx_Vy(x, y),

            (0xA, _, _, _) => Instruction::LD_I_addr(nnn),
            (0xB, x, _, _) => Instruction::JP_Vx_addr(x, nnn),

            (0xC, x, _, _) => Instruction::RND_Vx_kk(x, kk),
            (0xD, x, y, n) => Instruction::DRW_Vx_Vy_n(x, y, n),

            (0xE, x, 0x9, 0xE) => Instruction::SKP_Vx(x),
            (0xE, x, 0xA, 0x1) => Instruction::SKNP_Vx(x),

            (0xF, x, 0x0, 0x7) => Instruction::LD_Vx_DT(x),
            (0xF, x, 0x0, 0xA) => Instruction::LD_Vx_K(x),
            (0xF, x, 0x1, 0x5) => Instruction::LD_DT_Vx(x),
            (0xF, x, 0x1, 0x8) => Instruction::LD_ST_Vx(x),
            (0xF, x, 0x1, 0xE) => Instruction::ADD_I_Vx(x),

            (0xF, x, 0x3, 0x3) => Instruction::LD_B_Vx(x),
            (0xF, x, 0x5, 0x5) => Instruction::LD_iI_Vx(x),
            (0xF, x, 0x6, 0x5) => Instruction::LD_Vx_iI(x),

            (0xF, x, 0x2, 0x9) => Instruction::LD_F_Vx(x),
            (0xF, x, 0x3, 0x0) => Instruction::LD_HF_Vx(x),

            (0xF, x, 0x7, 0x5) => Instruction::SAVE_Vx(x),
            (0xF, x, 0x8, 0x5) => Instruction::LOAD_Vx(x),

            _ => panic!("Invalid opcode ${:04x}", op),
        }
    }
}

#[cfg(test)]
mod tests {}
