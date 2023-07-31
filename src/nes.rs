use crate::{
    bus::{controller::Joypad, Bus},
    cpu::{rom::ROM, CPU},
    ppu::registers::Status,
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

    pub fn step(&mut self, frame: &mut [u8]) {
        let cpu_cycles = self.cpu.step();
        self.cpu.bus.advance(cpu_cycles, frame);
    }

    #[inline]
    fn on_frame_complete(&mut self) {
        self.cpu.bus.ppu.frame_complete = false;
    }

    pub fn next_frame(&mut self, frame: &mut [u8]) {
        while !self.cpu.bus.ppu.frame_complete {
            self.step(frame);
        }

        self.on_frame_complete();
    }

    pub fn next_frame_inaccurate(&mut self, frame: &mut [u8]) {
        let mut cycles = 0;

        let (spzx, spzy) = self.cpu.bus.ppu.sprite_zero_coords();
        let sprite_zero_cycles = (spzy as f32 * 113.667 + spzx as f32 * 3.0) as u32;

        while cycles < sprite_zero_cycles {
            cycles += self.cpu.step();
        }

        self.cpu.bus.ppu.regs.status.insert(Status::SPRITE_ZERO_HIT);

        while cycles < 27_508 {
            cycles += self.cpu.step();
        }

        self.cpu.bus.ppu.render_frame(frame);
        self.cpu.bus.ppu.start_vblank();
        self.cpu.bus.ppu.tick_inaccurate();

        while cycles < 29_781 {
            cycles += self.cpu.step();
        }

        self.cpu.bus.ppu.end_vblank();
    }

    /// emulates enough cycles to fill the audio buffer,
    pub fn next_samples(&mut self, audio_buffer: &mut [f32], frame_buffer: &mut [u8]) -> bool {
        let mut count = 0;
        let mut new_frame = false;

        while count < audio_buffer.len() {
            loop {
                match self.cpu.bus.apu.pull_sample() {
                    Some(sample) => {
                        audio_buffer[count] = sample;
                        count += 1;
                        if self.cpu.bus.ppu.frame_complete {
                            self.on_frame_complete();
                            new_frame = true;
                        }
                        break;
                    }
                    None => self.step(frame_buffer),
                }
            }
        }

        new_frame
    }

    pub fn wait_for_samples(&mut self, count: usize, frame: &mut [u8]) {
        let mut i = 0;

        while i < count {
            loop {
                match self.cpu.bus.apu.pull_sample() {
                    Some(_) => {
                        i += 1;
                        break;
                    }
                    None => self.step(frame),
                }
            }
        }
    }

    pub fn fill_audio_buffer(
        &mut self,
        audio_buffer: &mut [f32],
        frame_buffer: &mut [u8],
        avoid_underruns: bool,
    ) {
        let remaining_samples_in_bufffer = self.cpu.bus.apu.remaining_samples() as usize;

        if avoid_underruns {
            // ensure that the buffer is filled with enough samples
            if remaining_samples_in_bufffer < audio_buffer.len() {
                let wait_for = audio_buffer.len() - remaining_samples_in_bufffer + 1;
                self.wait_for_samples(wait_for, frame_buffer);
            }
        }

        self.cpu.bus.apu.fill(audio_buffer);

        for i in remaining_samples_in_bufffer..audio_buffer.len() {
            audio_buffer[i] = 0.0;
        }
    }

    pub fn clear_audio_buffer(&mut self) {
        self.cpu.bus.apu.clear_buffer();
    }

    pub fn soft_reset(&mut self) {
        self.cpu.soft_reset();
    }

    pub fn get_joypad1_mut(&mut self) -> &mut Joypad {
        &mut self.cpu.bus.joypad1
    }

    pub fn get_joypad2_mut(&mut self) -> &mut Joypad {
        &mut self.cpu.bus.joypad2
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
