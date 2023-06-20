mod bus;
mod console;
mod cpu;
mod js;
mod ppu;
use bus::controller::JoypadStatus;
use cfg_if::cfg_if;
use console::Nes;
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
pub fn create_console(rom: Vec<u8>) -> Nes {
    console_error_panic_hook::set_once();
    let rom = ROM::new(rom).unwrap();
    Nes::new(rom)
}

#[wasm_bindgen(js_name = nextFrame)]
pub fn next_frame(console: &mut Nes, buffer: &mut [u8]) {
    console.next_frame(buffer);
}

#[wasm_bindgen(js_name = updateJoypad1)]
pub fn update_joypad1(console: &mut Nes, button: u8, pressed: bool) {
    let btn = JoypadStatus::from_bits(button).unwrap();
    console.joypad1().update_button_state(btn, pressed);
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
}
