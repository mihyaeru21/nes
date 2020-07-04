use nes::Nes;
use rom::Rom;
use std::{fs::File, io::BufReader};

mod cpu;
mod nes;
mod ram;
mod rom;

fn main() {
    let file = File::open("./tests/rom/hello_world.nes").unwrap();
    let rom = Rom::load(&mut BufReader::new(file)).unwrap();

    let mut nes = Nes::new();
    nes.set_rom(rom);
    nes.run();
}
