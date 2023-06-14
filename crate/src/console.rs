use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    bus::{controller::Joypad, Bus},
    cpu::{rom::ROM, CPU},
};

#[wasm_bindgen]
pub struct Console {
    cpu: CPU,
}

impl Console {
    pub fn new(rom: ROM) -> Self {
        let bus = Bus::new(rom);
        Console { cpu: CPU::new(bus) }
    }

    #[inline]
    fn step(&mut self) {
        let cpu_cycles = self.cpu.step();
        self.cpu.bus.advance_ppu(cpu_cycles);
    }

    pub fn next_frame(&mut self) -> &[u8] {
        loop {
            let vblank_before = self.cpu.bus.ppu.is_vblank();
            self.step();
            let vblank_after = self.cpu.bus.ppu.is_vblank();

            if !vblank_before && vblank_after {
                break;
            }
        }

        self.cpu.bus.ppu.render_frame();
        &self.cpu.bus.ppu.screen.pixels
    }

    pub fn controller(&mut self) -> &mut Joypad {
        &mut self.cpu.bus.controller
    }
}
