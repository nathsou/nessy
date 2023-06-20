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
        while !self.cpu.bus.ppu.frame_complete {
            self.step();
        }

        self.cpu.bus.ppu.frame_complete = false;
        self.cpu.bus.ppu.render_frame(frame);
    }

    #[inline]
    pub fn joypad1(&mut self) -> &mut Joypad {
        &mut self.cpu.bus.joypad1
    }
}
