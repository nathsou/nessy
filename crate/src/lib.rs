mod apu;
mod bus;
mod cpu;
mod js;
mod nes;
mod ppu;
use cpu::rom::{RomError, ROM};
use nes::Nes;
mod savestate;
extern crate console_error_panic_hook;
use savestate::SaveStateError;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

impl From<RomError> for JsValue {
    fn from(err: RomError) -> JsValue {
        match err {
            RomError::InvalidiNesHeader => JsValue::from_str("Invalid iNES header"),
            RomError::InvalidSaveStateHeader => JsValue::from_str("Invalid save state header"),
            RomError::UnsupportedMapper(mapper_id) => {
                JsValue::from_str(&format!("Unsupported mapper: {}", mapper_id))
            }
        }
    }
}

impl From<SaveStateError> for JsValue {
    fn from(err: SaveStateError) -> JsValue {
        match err {
            SaveStateError::InvalidHeader => JsValue::from_str("Invalid savestate header"),
            SaveStateError::InvalidVersion(v) => {
                JsValue::from_str(&format!("Invalid savestate version: {}", v))
            }
            SaveStateError::IncoherentRomHash { .. } => {
                JsValue::from_str("Incoherent savestate hash")
            }
            SaveStateError::InvalidData => JsValue::from_str("Invalid save state data"),
            SaveStateError::MissingSection(name) => {
                JsValue::from_str(&format!("Missing savestate section: {:?}", name))
            }
        }
    }
}

#[wasm_bindgen(js_name = Nes)]
pub struct WasmNes {
    nes: Nes,
}

#[wasm_bindgen(js_class = Nes)]
impl WasmNes {
    #[wasm_bindgen(js_name = initPanicHook)]
    pub fn init_panic_hook() {
        console_error_panic_hook::set_once();
    }

    pub fn new(rom: Vec<u8>, sample_rate: f64) -> Result<WasmNes, RomError> {
        let rom = ROM::new(rom)?;

        Ok(WasmNes {
            nes: Nes::new(rom, sample_rate),
        })
    }

    #[wasm_bindgen(js_name = softReset)]
    pub fn soft_reset(&mut self) {
        self.nes.soft_reset();
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
    pub fn save_state(&self) -> Vec<u8> {
        self.nes.save_state().encode()
    }

    #[wasm_bindgen(js_name = loadState)]
    pub fn load_state(&mut self, data: &[u8]) -> Result<(), SaveStateError> {
        self.nes.load_state(data)
    }

    #[wasm_bindgen(js_name = fillAudioBuffer)]
    pub fn fill_audio_buffer(&mut self, buffer: &mut [f32], avoid_underruns: bool) {
        self.nes.fill_audio_buffer(buffer, avoid_underruns);
    }
}
