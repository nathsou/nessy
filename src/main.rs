mod bus;
mod cpu;
mod graphics;
mod ppu;
use bus::Bus;
use cpu::rom::ROM;
use cpu::CPU;

fn main() {
    // let args = env::args().take(2).collect::<Vec<_>>();

    // if args.len() < 2 {
    //     eprintln!("usage: nessy rom.nes");
    // } else {
    // let rom_path = &args[1];
    // let rom = ROM::from(rom_path).unwrap();
}

#[test]
fn test_nestest_dump() {
    let rom = ROM::load("src/tests/nestest.nes").expect("nestest.nes not found");
    let logs = include_str!("tests/nestest.log").lines();
    let bus = Bus::new(rom);
    let mut cpu = CPU::new(bus);
    cpu.pc = 0xc000;

    for log_line in logs {
        if cpu.pc == 0xc6bd {
            // illegal opcodes after this point
            break;
        }

        let expected_pc = &log_line[0..4];
        let actual_pc = format!("{:04X}", cpu.pc);
        assert_eq!(expected_pc, actual_pc, "PC mismatch");

        let expected_regs = &log_line[48..73];
        let actual_regs = format!("{cpu:?}");
        assert_eq!(expected_regs, actual_regs, "Registers mismatch");

        cpu.step();
    }
}
