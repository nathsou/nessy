extern crate nessy;
extern crate wasm_bindgen;

mod js;

use nessy::{
    cpu::rom::{RomError, ROM},
    savestate::SaveStateError,
    Nes,
};
extern crate console_error_panic_hook;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

pub struct RomErrorWrapper(RomError);
pub struct SaveStateErrorWrapper(SaveStateError);

impl From<RomErrorWrapper> for JsValue {
    fn from(err: RomErrorWrapper) -> JsValue {
        match err.0 {
            RomError::InvalidiNesHeader => JsValue::from_str("Invalid iNES header"),
            RomError::InvalidSaveStateHeader => JsValue::from_str("Invalid save state header"),
            RomError::UnsupportedMapper(mapper_id) => {
                JsValue::from_str(&format!("Unsupported mapper: {}", mapper_id))
            }
        }
    }
}

impl From<SaveStateErrorWrapper> for JsValue {
    fn from(err: SaveStateErrorWrapper) -> JsValue {
        match err.0 {
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

    pub fn new(rom: Vec<u8>, sample_rate: f64) -> Result<WasmNes, RomErrorWrapper> {
        ROM::new(rom)
            .map(|rom| WasmNes {
                nes: Nes::new(rom, sample_rate),
            })
            .map_err(RomErrorWrapper)
    }

    #[wasm_bindgen(js_name = softReset)]
    pub fn soft_reset(&mut self) {
        self.nes.soft_reset();
    }

    #[wasm_bindgen(js_name = nextFrame)]
    pub fn next_frame(&mut self, buffer: &mut [u8]) {
        self.nes.next_frame();
        buffer.copy_from_slice(self.nes.get_frame());
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
        self.nes.get_joypad1_mut().update(buttons);
    }

    #[wasm_bindgen(js_name = setJoypad2)]
    pub fn set_joypad2(&mut self, buttons: u8) {
        self.nes.get_joypad2_mut().update(buttons);
    }

    #[wasm_bindgen(js_name = saveState)]
    pub fn save_state(&self) -> Vec<u8> {
        self.nes.save_state().encode()
    }

    #[wasm_bindgen(js_name = loadState)]
    pub fn load_state(&mut self, data: &[u8]) -> Result<(), SaveStateErrorWrapper> {
        self.nes.load_state(data).map_err(SaveStateErrorWrapper)
    }

    #[wasm_bindgen(js_name = fillAudioBuffer)]
    pub fn fill_audio_buffer(&mut self, buffer: &mut [f32], avoid_underruns: bool) {
        self.nes.fill_audio_buffer(buffer, avoid_underruns);
    }

    #[wasm_bindgen(js_name = clearAudioBuffer)]
    pub fn clear_audio_buffer(&mut self) {
        self.nes.clear_audio_buffer();
    }
}
