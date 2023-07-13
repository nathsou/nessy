use super::common::{Envelope, LengthCounter, Timer};

const NOISE_PERIOD_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

pub struct NoiseChannel {
    enabled: bool,
    length_counter: LengthCounter,
    envelope: Envelope,
    timer: Timer,
    shift_register: u16,
    mode: bool,
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            length_counter: LengthCounter::default(),
            envelope: Envelope::default(),
            timer: Timer::default(),
            shift_register: 1,
            mode: false,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        if !enabled {
            self.length_counter.reset_to_zero();
        }
    }

    pub fn step_timer(&mut self) {
        if self.timer.step() {
            let bit = if self.mode { 6 } else { 1 };
            let bit0 = self.shift_register & 1;
            let other_bit = (self.shift_register >> bit) & 1;
            let feedback = bit0 ^ other_bit;
            self.shift_register >>= 1;
            self.shift_register |= feedback << 14;
        }
    }

    #[inline]
    pub fn step_length_counter(&mut self) {
        self.length_counter.step();
    }

    #[inline]
    pub fn step_envelope(&mut self) {
        self.envelope.step();
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x400C => {
                let halt_length_counter = val & 0b0010_0000 != 0;
                self.length_counter.set_enabled(!halt_length_counter);
                self.envelope.looping = halt_length_counter;
                self.envelope.constant_mode = val & 0b0001_0000 != 0;
                self.envelope.period = val & 0b1111;
                self.envelope.constant_volume = val & 0b1111;
            }
            0x400E => {
                self.mode = val & 0b1000_0000 != 0;
                self.timer.period = NOISE_PERIOD_TABLE[(val & 0b1111) as usize];
            }
            0x400F => {
                self.length_counter.set(val >> 3);
                self.envelope.start = true;
            }
            _ => {}
        }
    }

    pub fn output(&self) -> u8 {
        if self.shift_register & 1 == 1 || self.length_counter.is_zero() {
            0
        } else {
            self.envelope.output()
        }
    }
}
