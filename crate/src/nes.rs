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
    pub fn step(&mut self, frame: &mut [u8]) {
        let cpu_cycles = self.cpu.step();
        self.cpu.bus.advance_ppu(frame, cpu_cycles);
    }

    pub fn next_frame(&mut self, frame: &mut [u8]) {
        while !self.cpu.bus.ppu.frame_complete {
            self.step(frame);
        }

        self.cpu.bus.ppu.frame_complete = false;
    }

    #[inline]
    pub fn joypad1(&mut self) -> &mut Joypad {
        &mut self.cpu.bus.joypad1
    }

    #[inline]
    pub fn get_cpu(&self) -> &CPU {
        &self.cpu
    }

    pub fn trace(&self) -> String {
        let cpu_trace = self.cpu.trace();
        let ppu_trace = self.cpu.bus.ppu.trace();
        format!("{cpu_trace}|{ppu_trace}")
    }
}
