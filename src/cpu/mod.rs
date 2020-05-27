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

    pub fn run(&mut self) {
        let opcode = self.fetch();
        let instruction = Instruction::from_opcode(opcode);

        let operand: Operand = match instruction.addressing {
            Addressing::Immediate => Operand::Value(self.fetch()),
            Addressing::Absolute => Operand::Address(self.fetch_word()),
            Addressing::AbsoluteX => {
                let addr = self.fetch_word();
                let x = self.registers.index_x as u16;
                Operand::Address(addr + x)
            }
            _ => Operand::None,
        };

        let mut calc_result = 0x00;
        match instruction.kind {
            Kind::SEI => {
                self.registers.status.irq_prohibited = true;
            }
            Kind::TXS => {
                self.registers.stack_pointer = self.registers.index_x;
                calc_result = self.registers.stack_pointer;
            }
            Kind::LDX => {
                match operand {
                    Operand::Value(v) => self.registers.index_x = v,
                    Operand::Address(addr) => self.registers.index_x = self.read(addr),
                    _ => {}
                }
                calc_result = self.registers.index_x;
            }
            Kind::LDA => {
                match operand {
                    Operand::Value(v) => self.registers.accumulator = v,
                    Operand::Address(addr) => self.registers.accumulator = self.read(addr),
                    _ => {}
                }
                calc_result = self.registers.accumulator;
            }
            _ => {}
        }

        if instruction.affects_status_negative() {
            self.registers.status.negative = (calc_result >> 7) == 0x01;
        }

        if instruction.affects_status_zero() {
            self.registers.status.zero = calc_result == 0x00;
        }
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

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x07ff => self.ram.borrow()[addr as usize],
            0x8000..=0xffff => {
                let i = addr - 0x8000;
                self.rom[i as usize]
            }
            _ => {
                panic!("Not implemented!");
            }
        }
    }

    fn read_word(&self, addr: u16) -> u16 {
        let lower_byte = self.read(addr) as u16;
        let upper_byte = self.read(addr + 1) as u16;
        lower_byte | (upper_byte << 8)
    }

    #[cfg(test)]
    pub fn get_registers(&mut self) -> &mut Registers {
        &mut self.registers
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Operand {
    Address(u16),
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
    fn test_instruction_sei_0x78() {
        let (mut cpu, _ram) = prepare(&[0x78]);
        cpu.run();
        assert!(cpu.get_registers().status.irq_prohibited);
    }

    #[test]
    fn test_instruction_txs_0x9a() {
        let (mut cpu, _ram) = prepare(&[0x9a, 0x9a]);
        cpu.get_registers().index_x = 0xff;
        cpu.run();
        assert_eq!(cpu.get_registers().stack_pointer, 0xff);
        assert_eq!(cpu.get_registers().status.negative, true);
        assert_eq!(cpu.get_registers().status.zero, false);
        cpu.get_registers().index_x = 0x00;
        cpu.run();
        assert_eq!(cpu.get_registers().stack_pointer, 0x00);
        assert_eq!(cpu.get_registers().status.negative, false);
        assert_eq!(cpu.get_registers().status.zero, true);
    }

    #[test]
    fn test_instruction_ldx_0xa2() {
        let (mut cpu, _ram) = prepare(&[0xa2, 0xff, 0xa2, 0x00]);
        cpu.run();
        assert_eq!(cpu.get_registers().index_x, 0xff);
        assert_eq!(cpu.get_registers().status.negative, true);
        assert_eq!(cpu.get_registers().status.zero, false);
        cpu.run();
        assert_eq!(cpu.get_registers().index_x, 0x00);
        assert_eq!(cpu.get_registers().status.negative, false);
        assert_eq!(cpu.get_registers().status.zero, true);
    }

    #[test]
    fn test_instruction_lda_0xa9() {
        let (mut cpu, _ram) = prepare(&[0xa9, 0xff, 0xa9, 0x00]);
        cpu.run();
        assert_eq!(cpu.get_registers().accumulator, 0xff);
        assert_eq!(cpu.get_registers().status.negative, true);
        assert_eq!(cpu.get_registers().status.zero, false);
        cpu.run();
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

        cpu.run();
        assert_eq!(cpu.get_registers().accumulator, 0xff);
        cpu.run();
        assert_eq!(cpu.get_registers().accumulator, 0x45);
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
