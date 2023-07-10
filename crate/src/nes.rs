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
    pub fn step(&mut self) {
        let cpu_cycles = self.cpu.step();
        self.cpu.bus.advance(cpu_cycles);
    }

    pub fn next_frame(&mut self) {
        while !self.cpu.bus.ppu.frame_complete {
            self.step();
        }

        self.cpu.bus.ppu.frame_complete = false;
    }

    /// emulates enough cycles to fill the audio buffer,
    /// the frame buffer gets updated when a new frame is ready
    pub fn next_samples(&mut self, frame_buffer: &mut [u8], audio_buffer: &mut [f32]) {
        let mut count = 0;

        while count < audio_buffer.len() {
            loop {
                match self.cpu.bus.apu.pull_sample() {
                    Some(sample) => {
                        audio_buffer[count] = sample;
                        count += 1;
                        if self.cpu.bus.ppu.frame_complete {
                            self.cpu.bus.ppu.frame_complete = false;
                            frame_buffer.copy_from_slice(self.cpu.bus.ppu.get_frame());
                        }
                        break;
                    }
                    None => self.step(),
                }
            }
        }
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
    pub fn get_frame(&self) -> &[u8] {
        self.cpu.bus.ppu.get_frame()
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
