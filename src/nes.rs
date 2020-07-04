use crate::{cpu::Cpu, ram::Ram, rom::Rom};
use std::{cell::RefCell, rc::Rc, thread::sleep, time};

#[derive(Debug)]
pub struct Nes {
    cpu: Cpu,
    wram: Ram,
    rom: Option<Rc<Rom>>,
}

impl Nes {
    pub fn new() -> Self {
        let wram = Rc::new(RefCell::new(vec![0; 0x800]));
        let cpu = Cpu::new(wram.clone());

        Self {
            cpu,
            wram,
            rom: None,
        }
    }

    pub fn set_rom(&mut self, rom: Rom) {
        let program = Rc::new(rom.program.clone());
        self.rom = Some(Rc::new(rom));
        self.cpu.set_rom(Some(program));
    }

    pub fn run(&mut self) {
        self.cpu.reset();

        loop {
            let clock = self.cpu.run();
            println!("#################################################");
            println!("clock: {}", clock);
            self.cpu.dump_registers();
            sleep(time::Duration::from_millis(500));
        }
    }
}
