mod apu;
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

#[wasm_bindgen(js_name = Nes)]
pub struct WasmNes {
    nes: Nes,
}

#[wasm_bindgen(js_class = Nes)]
impl WasmNes {
    pub fn new(rom: Vec<u8>, sample_rate: f64) -> WasmNes {
        console_error_panic_hook::set_once();
        let rom = ROM::new(rom).unwrap();
        WasmNes {
            nes: Nes::new(rom, sample_rate),
        }
    }

    pub fn reset(&mut self) {
        self.nes.reset();
    }

    #[wasm_bindgen(js_name = nextFrame)]
    pub fn next_frame(&mut self, buffer: &mut [u8]) {
        self.nes.next_frame();
        buffer.copy_from_slice(self.nes.get_frame())
    }

    #[wasm_bindgen(js_name = nextSamples)]
    pub fn next_samples(&mut self, audio_buffer: &mut [f32]) -> bool {
        self.nes.next_samples(audio_buffer)
    }

    #[wasm_bindgen(js_name = fillFrameBuffer)]
    pub fn fill_frame_buffer(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(self.nes.get_frame())
    }

    #[wasm_bindgen(js_name = setJoypad1)]
    pub fn set_joypad1(&mut self, buttons: u8) {
        self.nes.joypad1().update(buttons);
    }

    #[wasm_bindgen(js_name = saveState)]
    pub fn save_state(&mut self) -> Vec<u8> {
        let mut state = SaveState::new();
        self.nes.save(&mut state);
        state.get_data()
    }

    #[wasm_bindgen(js_name = loadState)]
    pub fn load_state(&mut self, data: Vec<u8>) {
        let mut state = SaveState::from(data);
        self.nes.load(&mut state);
    }

    #[wasm_bindgen(js_name = fillAudioBuffer)]
    pub fn fill_audio_buffer(&mut self, buffer: &mut [f32], avoid_underruns: bool) {
        self.nes.fill_audio_buffer(buffer, avoid_underruns);
    }
}
