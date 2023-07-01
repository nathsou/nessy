mod bus;
mod cpu;
mod js;
mod nes;
mod ppu;
use cfg_if::cfg_if;
use cpu::rom::ROM;
use nes::Nes;
mod savestate;
extern crate console_error_panic_hook;
use savestate::{Save, SaveState};
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

#[wasm_bindgen(js_name = resetConsole)]
pub fn reset_console(console: &mut Nes) {
    console.reset();
}

#[wasm_bindgen(js_name = nextFrame)]
pub fn next_frame(console: &mut Nes, buffer: &mut [u8]) {
    console.next_frame(buffer);
}

#[wasm_bindgen(js_name = setJoypad1)]
pub fn set_joypad1(console: &mut Nes, buttons: u8) {
    console.joypad1().update(buttons);
}

#[wasm_bindgen(js_name = saveState)]
pub fn save_state(console: &mut Nes) -> Vec<u8> {
    let mut state = SaveState::new();
    console.save(&mut state);
    state.get_data()
}

#[wasm_bindgen(js_name = loadState)]
pub fn load_state(console: &mut Nes, data: Vec<u8>) {
    let mut state = SaveState::from(data);
    console.load(&mut state);
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::path::Path;

    use crate::cpu::rom::ROM;
    use crate::nes::Nes;

    fn load_rom(path: &str) -> ROM {
        let bytes = std::fs::read(path).unwrap();
        ROM::new(bytes).expect("Failed to load ROM")
    }

    #[test]
    fn test_dump() {
        let rom =
            load_rom("/Users/nathan/Documents/Code/Rust/nessy/web/public/roms/Chessmaster.nes");
        let path = Path::new("/Users/nathan/Desktop/dump.log");
        let file = File::open(path).expect("Failed to open dump file");
        let reader = io::BufReader::new(file);
        let mut nes = Nes::new(rom);
        let mut lines = reader.lines().map(|l| l.unwrap()).peekable();
        let mut frame = [0u8; 256 * 240 * 4];
        let mut pcs = [0, 0, 0, 0];

        while let Some(line) = lines.peek() {
            if let Some(input) = line.strip_prefix('!') {
                let joypad1 = u8::from_str_radix(input, 2).unwrap();
                nes.joypad1().update(joypad1);
                lines.next();
            } else {
                nes.step(&mut frame);
                let pc = nes.get_cpu().pc;
                let is_loop = pcs.contains(&pc);

                pcs[3] = pcs[2];
                pcs[2] = pcs[1];
                pcs[1] = pcs[0];
                pcs[0] = pc;

                if !is_loop {
                    let trace = nes.trace();
                    println!("{trace}");
                    assert_eq!(&trace, line);
                    lines.next();
                }
            }
        }
    }
}
