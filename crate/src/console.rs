use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    bus::{controller::Joypad, Bus},
    cpu::{rom::ROM, CPU},
};

#[wasm_bindgen]
pub struct Nes {
    cpu: CPU,
}

impl Nes {
    pub fn new(rom: ROM) -> Self {
        let bus = Bus::new(rom);
        Nes { cpu: CPU::new(bus) }
    }

    #[inline]
    fn step(&mut self) {
        let cpu_cycles = self.cpu.step();
        self.cpu.bus.advance_ppu(cpu_cycles);
    }

    pub fn next_frame(&mut self, frame: &mut [u8]) {
        loop {
            let vblank_before = self.cpu.bus.ppu.is_vblank();
            self.step();
            let vblank_after = self.cpu.bus.ppu.is_vblank();

            if !vblank_before && vblank_after {
                break;
            }
        }

        self.cpu.bus.ppu.render_frame(frame);
    }

    #[inline]
    pub fn joypad1(&mut self) -> &mut Joypad {
        &mut self.cpu.bus.joypad1
    }
}
