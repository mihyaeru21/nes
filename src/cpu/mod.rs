use instruction::{Instruction, Kind};
use register::Registers;
use std::{cell::RefCell, rc::Rc};

mod instruction;
mod register;

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
        match instruction.kind {
            Kind::SEI => {
                self.registers.status.irq_prohibited = true;
            }
            _ => {}
        }
    }

    fn fetch(&mut self) -> u8 {
        let value = self.read(self.registers.program_counter);
        self.registers.program_counter += 1;
        value
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
    pub fn get_registers(&self) -> &Registers {
        &self.registers
    }
}

type Ram = Rc<RefCell<Vec<u8>>>;

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
    fn test_instruction_sei() {
        let (mut cpu, mut _ram) = prepare(&[0x78]);
        cpu.run();
        assert!(cpu.get_registers().status.irq_prohibited);
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
