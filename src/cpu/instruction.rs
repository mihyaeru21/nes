#[derive(Debug, PartialEq, Eq)]
pub struct Instruction {
    pub kind: Kind,
    pub addressing: Addressing,
}

impl Instruction {
    pub fn from_opcode(opcode: u8) -> Self {
        // とりあえずhello worldを動かすのに必要なopcode
        let (kind, addressing) = match opcode {
            0x4c => (Kind::JMP, Addressing::Absolute),
            0x78 => (Kind::SEI, Addressing::Implied),
            0x88 => (Kind::DEY, Addressing::Implied),
            0x8d => (Kind::STA, Addressing::Absolute),
            0x9a => (Kind::TXS, Addressing::Implied),
            0xa0 => (Kind::LDY, Addressing::Immediate),
            0xa2 => (Kind::LDX, Addressing::Immediate),
            0xa9 => (Kind::LDA, Addressing::Immediate),
            0xbd => (Kind::LDA, Addressing::AbsoluteX),
            0xd0 => (Kind::BNE, Addressing::Relative),
            0xe8 => (Kind::INX, Addressing::Implied),
            _ => panic!("Instruction is not implemented! 0x{:x}", opcode),
        };
        Self { kind, addressing }
    }

    pub fn clock(&self) -> u8 {
        // とりあえずhello worldを動かすのに必要なやつ
        let base = match self.kind {
            Kind::JMP => 1,
            Kind::SEI => 2,
            Kind::DEY => 2,
            Kind::STA => 2,
            Kind::TXS => 2,
            Kind::LDY => 2,
            Kind::LDX => 2,
            Kind::LDA => 2,
            Kind::BNE => 2,
            Kind::INX => 2,
        };

        base + match self.addressing {
            Addressing::Implied => 0,
            Addressing::Immediate => 0,
            Addressing::Relative => 0,
            Addressing::Absolute => 2,
            Addressing::AbsoluteX => 2,
        }
    }

    pub fn affects_status_negative(&self) -> bool {
        match self.kind {
            Kind::DEY | Kind::LDY | Kind::LDX | Kind::LDA | Kind::TXS | Kind::INX => true,
            _ => false,
        }
    }

    pub fn affects_status_zero(&self) -> bool {
        match self.kind {
            Kind::DEY | Kind::LDY | Kind::LDX | Kind::LDA | Kind::TXS | Kind::INX => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    // 転送
    LDA,
    LDX,
    LDY,
    STA,
    // STX,
    // STY,
    // TAX,
    // TAY,
    // TSX,
    // TXA,
    TXS,
    // TYA,
    // 算術
    // ADC,
    // AND,
    // ASL,
    // BIT,
    // CMP,
    // CPX,
    // CPY,
    // DEC,
    // DEX,
    DEY,
    // EOR,
    // INC,
    INX,
    // INY,
    // LSR,
    // ORA,
    // ROL,
    // ROR,
    // SBC,
    // stack
    // PHA,
    // PHP,
    // PLA,
    // PLP,
    // jump
    JMP,
    // JSR,
    // RTS,
    // RTI,
    // 分岐
    // BCC,
    // BCS,
    // BEQ,
    // BMI,
    BNE,
    // BPL,
    // BVC,
    // BVS,
    // フラグ変更
    // CLC,
    // CLD,
    // IRQ,
    // CLV,
    // SEC,
    // SED,
    SEI,
    // その他
    // BRK,
    // NOP,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Addressing {
    Implied,
    // Accumulator,
    Immediate,
    // ZeroPage,
    // ZeroPageX,
    // ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    // AbsoluteY,
    // Indirect,
    // IndirectX,
    // IndirectY,
}

#[cfg(test)]
mod test {
    use super::{Addressing, Instruction, Kind};

    #[test]
    fn test_from_opcode() {
        let instruction = Instruction::from_opcode(0xa9);
        let expectation = Instruction {
            kind: Kind::LDA,
            addressing: Addressing::Immediate,
        };
        assert_eq!(instruction, expectation);
    }
}
