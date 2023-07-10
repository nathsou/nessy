use crate::{
    bus::{controller::Joypad, Bus},
    cpu::{rom::ROM, CPU},
    savestate::{Save, SaveState},
};

pub struct Nes {
    cpu: CPU,
}

impl Nes {
    pub fn new(rom: ROM, sample_rate: f64) -> Self {
        let bus = Bus::new(rom, sample_rate);
        Nes { cpu: CPU::new(bus) }
    }

    #[inline]
    pub fn step(&mut self, frame: &mut [u8]) {
        let cpu_cycles = self.cpu.step();
        self.cpu.bus.advance(frame, cpu_cycles);
    }

    pub fn next_frame(&mut self, frame: &mut [u8]) {
        while !self.cpu.bus.ppu.frame_complete {
            self.step(frame);
        }

        self.cpu.bus.ppu.frame_complete = false;
    }

    pub fn fill_audio_buffer(&mut self, buffer: &mut [f32]) {
        self.cpu.bus.apu.fill(buffer);
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
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

impl Save for Nes {
    fn save(&self, s: &mut SaveState) {
        self.cpu.save(s);
    }

    fn load(&mut self, s: &mut SaveState) {
        self.cpu.load(s);
    }
}
