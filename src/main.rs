mod cpu;
mod graphics;
mod ppu;
use cpu::rom::ROM;
use cpu::MOS6502;
use std::env;

fn main() {
    match graphics::begin() {
        Ok(_) => {}
        Err(err) => println!("{}", err),
    };

    let args = env::args().take(2).collect::<Vec<_>>();

    if args.len() < 2 {
        panic!("usage: nessy rom.nes");
    }

    let rom_path = &args[1];

    let rom = ROM::from(rom_path).unwrap();

    let mut cpu = MOS6502::new(rom);

    let mut cycles = 0;

    while !cpu.interrupted() {
        cycles += cpu.trace_step();
    }

    println!("{:?}", cpu);
    println!("cycles: {}", cycles);
}
