use instruction::{Addressing, Instruction, Kind};
use register::Registers;
use std::{cell::RefCell, rc::Rc};

mod instruction;
mod register;

type Ram = Rc<RefCell<Vec<u8>>>;

#[derive(Debug)]
pub struct Cpu {
    registers: Registers,
    rom: Rc<Vec<u8>>, // TODO: RcのやつはBusに切り出す？
    ram: Ram,
}

impl Cpu {
    pub fn new(rom: Rc<Vec<u8>>, ram: Ram) -> Self {
        Cpu {
            registers: Registers::default(),
            rom,
            ram,
        }
    }

    pub fn reset(&mut self) {
        self.registers = Registers::default();
        self.registers.program_counter = self.read_word(0xfffc)
    }

    pub fn run(&mut self) -> u8 {
        let opcode = self.fetch();
        let instruction = Instruction::from_opcode(opcode);

        let mut clock_count = instruction.clock();
        let calc_result = match instruction.kind {
            Kind::JMP => {
                match self.fetch_operand(&instruction.addressing) {
                    Operand::Address(addr, _) => {
                        self.registers.program_counter = addr;
                    }
                    _ => {}
                }
                None
            }
            Kind::SEI => {
                self.registers.status.irq_prohibited = true;
                None
            }
            Kind::DEY => {
                self.registers.index_y = self.registers.index_y.wrapping_sub(1);
                Some(self.registers.index_y)
            }
            Kind::STA => {
                match self.fetch_operand(&instruction.addressing) {
                    Operand::Address(addr, page_crossed) => {
                        self.write(addr, self.registers.accumulator);
                        if page_crossed {
                            clock_count += 1;
                        }
                    }
                    _ => {}
                };
                None
            }
            Kind::TXS => {
                self.registers.stack_pointer = self.registers.index_x;
                Some(self.registers.stack_pointer)
            }
            Kind::LDY => {
                match self.fetch_operand(&instruction.addressing) {
                    Operand::Value(v) => self.registers.index_y = v,
                    Operand::Address(addr, page_crossed) => {
                        self.registers.index_y = self.read(addr);
                        if page_crossed {
                            clock_count += 1;
                        }
                    }
                    _ => {}
                };
                Some(self.registers.index_y)
            }
            Kind::LDX => {
                match self.fetch_operand(&instruction.addressing) {
                    Operand::Value(v) => self.registers.index_x = v,
                    Operand::Address(addr, page_crossed) => {
                        self.registers.index_x = self.read(addr);
                        if page_crossed {
                            clock_count += 1;
                        }
                    }
                    _ => {}
                };
                Some(self.registers.index_x)
            }
            Kind::LDA => {
                match self.fetch_operand(&instruction.addressing) {
                    Operand::Value(v) => self.registers.accumulator = v,
                    Operand::Address(addr, page_crossed) => {
                        self.registers.accumulator = self.read(addr);
                        if page_crossed {
                            clock_count += 1;
                        }
                    }
                    _ => {}
                };
                Some(self.registers.accumulator)
            }
            Kind::BNE => {
                match self.fetch_operand(&instruction.addressing) {
                    Operand::Address(addr, page_crossed) => {
                        if !self.registers.status.zero {
                            self.registers.program_counter = addr;
                            clock_count += if page_crossed { 2 } else { 1 };
                        }
                    }
                    _ => {}
                };
                None
            }
            Kind::INX => {
                self.registers.index_x = self.registers.index_x.wrapping_add(1);
                Some(self.registers.index_x)
            }
        };

        if let Some(result) = calc_result {
            if instruction.affects_status_negative() {
                self.registers.status.negative = (result >> 7) == 0x01;
            }

            if instruction.affects_status_zero() {
                self.registers.status.zero = result == 0x00;
            }
        }

        clock_count
    }

    fn fetch(&mut self) -> u8 {
        let value = self.read(self.registers.program_counter);
        self.registers.program_counter += 1;
        value
    }

    fn fetch_word(&mut self) -> u16 {
        let lower = self.fetch() as u16;
        let upper = self.fetch() as u16;
        lower + (upper << 8)
    }

    fn fetch_operand(&mut self, addressing: &Addressing) -> Operand {
        match addressing {
            Addressing::Immediate => Operand::Value(self.fetch()),
            Addressing::Relative => {
                let offset = self.fetch() as i8;
                let pc = self.registers.program_counter;
                let addr = if offset >= 0 {
                    pc.wrapping_add(offset as u16)
                } else {
                    pc.wrapping_sub(offset.abs() as u16)
                };
                let page_crossed = (pc >> 8) != (addr >> 8);
                Operand::Address(addr, page_crossed)
            }
            Addressing::Absolute => Operand::Address(self.fetch_word(), false),
            Addressing::AbsoluteX => {
                let orig = self.fetch_word();
                let x = self.registers.index_x as u16;
                let addr = orig.wrapping_add(x);
                let page_crossed = (orig >> 8) != (addr >> 8);
                Operand::Address(addr, page_crossed)
            }
            _ => Operand::None,
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x07ff => self.ram.borrow()[addr as usize],
            0x8000..=0xffff => {
                let i = addr - 0x8000;
                self.rom[i as usize]
            }
            _ => panic!("Not implemented!"),
        }
    }

    fn read_word(&self, addr: u16) -> u16 {
        let lower_byte = self.read(addr) as u16;
        let upper_byte = self.read(addr + 1) as u16;
        lower_byte | (upper_byte << 8)
    }

    fn write(&self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x07ff => {
                self.ram.borrow_mut()[addr as usize] = value;
            }
            _ => panic!("Not implemented!"),
        }
    }

    #[cfg(test)]
    pub fn get_registers(&mut self) -> &mut Registers {
        &mut self.registers
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Operand {
    Address(u16, bool),
    Value(u8),
    None,
}

#[cfg(test)]
mod test {
    use super::{Cpu, Ram};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn test_reset() {
        let mut rom = vec![0; 0x8000];
        rom[0x7ffc] = 0x00;
        rom[0x7ffd] = 0x80;

        let rom = Rc::new(rom);
        let ram = Rc::new(RefCell::new(vec![0; 0x800]));
        let mut cpu = Cpu::new(rom, ram);
        assert_eq!(cpu.get_registers().program_counter, 0);

        cpu.reset();
        assert_eq!(cpu.get_registers().program_counter, 0x8000);
    }

    #[test]
    fn test_instruction_jmp_0x4c() {
        let (mut cpu, _ram) = prepare(&[0x4c, 0xff, 0x80]);

        let clock = cpu.run();
        assert_eq!(clock, 3);
        assert_eq!(cpu.get_registers().program_counter, 0x80ff);
    }

    #[test]
    fn test_instruction_sei_0x78() {
        let (mut cpu, _ram) = prepare(&[0x78]);
        let clock = cpu.run();
        assert_eq!(clock, 2);
        assert!(cpu.get_registers().status.irq_prohibited);
    }

    #[test]
    fn test_instruction_dey_0x88() {
        let (mut cpu, _ram) = prepare(&[0x88, 0x88]);
        cpu.get_registers().index_y = 0x01;

        let clock = cpu.run();
        assert_eq!(clock, 2);
        assert_eq!(cpu.get_registers().index_y, 0x00);
        assert_eq!(cpu.get_registers().status.negative, false);
        assert_eq!(cpu.get_registers().status.zero, true);

        let clock = cpu.run();
        assert_eq!(clock, 2);
        assert_eq!(cpu.get_registers().index_y, 0xff);
        assert_eq!(cpu.get_registers().status.negative, true);
        assert_eq!(cpu.get_registers().status.zero, false);
    }

    #[test]
    fn test_instruction_sta_0x8d() {
        let (mut cpu, ram) = prepare(&[0x8d, 0x23, 0x01]);
        cpu.get_registers().accumulator = 0x56;
        let clock = cpu.run();
        assert_eq!(clock, 4);
        assert_eq!(ram.borrow()[0x0123], 0x56);
    }

    #[test]
    fn test_instruction_txs_0x9a() {
        let (mut cpu, _ram) = prepare(&[0x9a, 0x9a]);

        cpu.get_registers().index_x = 0xff;
        let clock = cpu.run();
        assert_eq!(clock, 2);
        assert_eq!(cpu.get_registers().stack_pointer, 0xff);
        assert_eq!(cpu.get_registers().status.negative, true);
        assert_eq!(cpu.get_registers().status.zero, false);

        cpu.get_registers().index_x = 0x00;
        let clock = cpu.run();
        assert_eq!(clock, 2);
        assert_eq!(cpu.get_registers().stack_pointer, 0x00);
        assert_eq!(cpu.get_registers().status.negative, false);
        assert_eq!(cpu.get_registers().status.zero, true);
    }

    #[test]
    fn test_instruction_ldy_0xa0() {
        let (mut cpu, _ram) = prepare(&[0xa0, 0xff, 0xa0, 0x00]);

        let clock = cpu.run();
        assert_eq!(clock, 2);
        assert_eq!(cpu.get_registers().index_y, 0xff);
        assert_eq!(cpu.get_registers().status.negative, true);
        assert_eq!(cpu.get_registers().status.zero, false);

        let clock = cpu.run();
        assert_eq!(clock, 2);
        assert_eq!(cpu.get_registers().index_y, 0x00);
        assert_eq!(cpu.get_registers().status.negative, false);
        assert_eq!(cpu.get_registers().status.zero, true);
    }

    #[test]
    fn test_instruction_ldx_0xa2() {
        let (mut cpu, _ram) = prepare(&[0xa2, 0xff, 0xa2, 0x00]);

        let clock = cpu.run();
        assert_eq!(clock, 2);
        assert_eq!(cpu.get_registers().index_x, 0xff);
        assert_eq!(cpu.get_registers().status.negative, true);
        assert_eq!(cpu.get_registers().status.zero, false);

        let clock = cpu.run();
        assert_eq!(clock, 2);
        assert_eq!(cpu.get_registers().index_x, 0x00);
        assert_eq!(cpu.get_registers().status.negative, false);
        assert_eq!(cpu.get_registers().status.zero, true);
    }

    #[test]
    fn test_instruction_lda_0xa9() {
        let (mut cpu, _ram) = prepare(&[0xa9, 0xff, 0xa9, 0x00]);

        let clock = cpu.run();
        assert_eq!(clock, 2);
        assert_eq!(cpu.get_registers().accumulator, 0xff);
        assert_eq!(cpu.get_registers().status.negative, true);
        assert_eq!(cpu.get_registers().status.zero, false);

        let clock = cpu.run();
        assert_eq!(clock, 2);
        assert_eq!(cpu.get_registers().accumulator, 0x00);
        assert_eq!(cpu.get_registers().status.negative, false);
        assert_eq!(cpu.get_registers().status.zero, true);
    }

    #[test]
    fn test_instruction_lda_0xbd() {
        let (mut cpu, ram) = prepare(&[0xbd, 0x00, 0x00, 0xbd, 0x23, 0x01]);
        {
            let mut ram = ram.borrow_mut();
            ram[0x0056] = 0xff;
            ram[0x0179] = 0x45;
        }
        cpu.get_registers().index_x = 0x56;

        let clock = cpu.run();
        assert_eq!(clock, 4);
        assert_eq!(cpu.get_registers().accumulator, 0xff);

        let clock = cpu.run();
        assert_eq!(clock, 4);
        assert_eq!(cpu.get_registers().accumulator, 0x45);

        // TODO: またぎ
    }

    #[test]
    fn test_instruction_bne_0xd0() {
        // INXの1回目は0になるから分岐せず、2回目のINXを実行したあとに分岐する
        let (mut cpu, _ram) = prepare(&[0xe8, 0xd0, 0xfa, 0xe8, 0xd0, 0xfa]);
        cpu.get_registers().index_x = 0xff;
        assert_eq!(cpu.get_registers().program_counter, 0x8000);

        cpu.run();
        let clock = cpu.run();
        assert_eq!(clock, 2);
        assert_eq!(cpu.get_registers().program_counter, 0x8003);

        cpu.run();
        let clock = cpu.run();
        assert_eq!(clock, 3);
        assert_eq!(cpu.get_registers().program_counter, 0x8000);

        // TODO: 4 のケース
    }

    #[test]
    fn test_instruction_inx_0xe8() {
        let (mut cpu, _ram) = prepare(&[0xe8, 0xe8]);
        cpu.get_registers().index_x = 0xfe;

        let clock = cpu.run();
        assert_eq!(clock, 2);
        assert_eq!(cpu.get_registers().index_x, 0xff);
        assert_eq!(cpu.get_registers().status.negative, true);
        assert_eq!(cpu.get_registers().status.zero, false);

        let clock = cpu.run();
        assert_eq!(clock, 2);
        assert_eq!(cpu.get_registers().index_x, 0x00);
        assert_eq!(cpu.get_registers().status.negative, false);
        assert_eq!(cpu.get_registers().status.zero, true);
    }

    fn prepare(initial_bytes: &[u8]) -> (Cpu, Ram) {
        let mut rom = vec![0; 0x8000];
        rom[0x7ffc] = 0x00;
        rom[0x7ffd] = 0x80;

        for (i, b) in initial_bytes.iter().enumerate() {
            rom[i] = b.clone();
        }

        let rom = Rc::new(rom);
        let ram = Rc::new(RefCell::new(vec![0; 0x800]));
        let mut cpu = Cpu::new(rom, ram.clone());
        cpu.reset();
        (cpu, ram)
    }
}
