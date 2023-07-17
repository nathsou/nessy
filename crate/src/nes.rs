use crate::{
    bus::{controller::Joypad, Bus},
    cpu::{rom::ROM, CPU},
    savestate::{self, Save, SaveState, SaveStateError},
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
    pub fn next_samples(&mut self, audio_buffer: &mut [f32]) -> bool {
        let mut count = 0;
        let mut new_frame = false;

        while count < audio_buffer.len() {
            loop {
                match self.cpu.bus.apu.pull_sample() {
                    Some(sample) => {
                        audio_buffer[count] = sample;
                        count += 1;
                        if self.cpu.bus.ppu.frame_complete {
                            self.cpu.bus.ppu.frame_complete = false;
                            new_frame = true;
                        }
                        break;
                    }
                    None => self.step(),
                }
            }
        }

        new_frame
    }

    pub fn wait_for_samples(&mut self, count: usize) {
        let mut i = 0;

        while i < count {
            loop {
                match self.cpu.bus.apu.pull_sample() {
                    Some(_) => {
                        i += 1;
                        break;
                    }
                    None => self.step(),
                }
            }
        }
    }

    pub fn fill_audio_buffer(&mut self, buffer: &mut [f32], avoid_underruns: bool) {
        if avoid_underruns {
            let remaining_samples_in_bufffer =
                self.cpu.bus.apu.remaining_buffered_samples() as usize;

            // ensure that the buffer is filled with enough samples
            // wihtout skipping a frame
            if remaining_samples_in_bufffer < buffer.len() {
                let remaining_samples_in_frame = self.cpu.bus.apu.remaining_samples_in_frame();
                let remaining_samples = buffer.len() - remaining_samples_in_bufffer;
                let wait_for = remaining_samples.min(remaining_samples_in_frame - 1);
                self.wait_for_samples(wait_for);
            }
        }

        self.cpu.bus.apu.fill(buffer);
    }

    pub fn soft_reset(&mut self) {
        self.cpu.soft_reset();
    }

    #[inline]
    pub fn get_joypad1_mut(&mut self) -> &mut Joypad {
        &mut self.cpu.bus.joypad1
    }

    #[inline]
    pub fn get_joypad2_mut(&mut self) -> &mut Joypad {
        &mut self.cpu.bus.joypad2
    }

    #[inline]
    pub fn get_frame(&self) -> &[u8] {
        self.cpu.bus.ppu.get_frame()
    }

    pub fn save_state(&self) -> SaveState {
        let mut state = SaveState::new(&self.cpu.bus.ppu.rom.cart.hash);
        self.save(state.get_root_mut());
        state
    }

    pub fn load_state(&mut self, data: &[u8]) -> Result<(), SaveStateError> {
        let mut state = SaveState::decode(data)?;

        let save_state_rom_hash = state.get_rom_hash();
        let cart_rom_hash = self.cpu.bus.ppu.rom.cart.hash;

        if cart_rom_hash != save_state_rom_hash {
            return Err(SaveStateError::IncoherentRomHash {
                save_state_rom_hash,
                cart_rom_hash,
            });
        }

        self.load(state.get_root_mut())?;

        Ok(())
    }
}

impl savestate::Save for Nes {
    fn save(&self, s: &mut savestate::Section) {
        self.cpu.save(s);
    }

    fn load(&mut self, s: &mut savestate::Section) -> Result<(), SaveStateError> {
        self.cpu.load(s)?;

        Ok(())
    }
}
