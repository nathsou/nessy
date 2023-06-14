mod bus;
mod console;
mod cpu;
mod ppu;
use cfg_if::cfg_if;
use console::Console;
use cpu::rom::ROM;
extern crate console_error_panic_hook;
use wasm_bindgen::prelude::wasm_bindgen;

cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen(js_name = createConsole)]
pub fn create_console(rom: Vec<u8>) -> Console {
    console_error_panic_hook::set_once();
    let rom = ROM::new(rom).unwrap();
    Console::new(rom)
}

#[wasm_bindgen(js_name = nextFrame)]
pub fn next_frame(console: &mut Console) -> Vec<u8> {
    console.next_frame().to_vec()
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu::rom::ROM;
    use crate::cpu::CPU;

    fn load_rom(path: &str) -> ROM {
        let bytes = std::fs::read(path).unwrap();
        ROM::new(bytes).expect("Failed to load ROM")
    }

    #[test]
    fn test_nestest_dump() {
        let rom = load_rom("tests/nestest.nes");
        let logs = include_str!("tests/nestest.log").lines();
        let bus = Bus::new(rom);
        let mut cpu = CPU::new(bus);
        cpu.pc = 0xc000;

        for log_line in logs {
            if cpu.pc == 0xc6bd {
                // illegal opcodes after this point
                break;
            }

            println!("{log_line}");

            let expected_pc = &log_line[0..4];
            let actual_pc = format!("{:04X}", cpu.pc);
            assert_eq!(expected_pc, actual_pc, "PC mismatch");

            let expected_regs = &log_line[48..73];
            let actual_regs = format!("{cpu:?}");
            assert_eq!(expected_regs, actual_regs, "Registers mismatch");

            cpu.step();
        }
    }

    #[test]
    #[ignore]
    fn test_smb_dump() {
        let rom = load_rom("roms/smb.nes");
        let dump = std::fs::read_to_string("smb.log").expect("smb.log not found");
        let bus = Bus::new(rom);
        let mut cpu = CPU::new(bus);
        let mut i = 0;
        let lines = dump.lines().collect::<Vec<_>>();

        while i < lines.len() {
            let line = lines[i];

            if line.starts_with('!') {
                let joypad1 = u8::from_str_radix(&line[1..], 2).unwrap();
                cpu.bus.controller.update(joypad1);
                i += 1;
            } else {
                let actual = cpu.state_fmt();
                let prev_pc = cpu.pc;
                let cycles = cpu.step();
                cpu.bus.advance_ppu(cycles);

                if prev_pc != cpu.pc {
                    let with_cycles = format!(
                        "{actual} CY {cycles} {}|{}",
                        cpu.bus.ppu.scanline, cpu.bus.ppu.cycle
                    );

                    assert_eq!(with_cycles, line);
                    i += 1;
                }
            }
        }
    }
}
