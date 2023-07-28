use super::common::{Envelope, LengthCounter, Timer};

const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

#[derive(Copy, Clone)]
pub enum PulseChannelId {
    Pulse1,
    Pulse2,
}

pub struct PulseChannel {
    id: PulseChannelId,
    enabled: bool,
    duty_mode: u8,
    duty_cycle: u8,
    sweep_enabled: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_reload: bool,
    sweep_divider: u8,
    sweep_mute: bool,
    length_counter: LengthCounter,
    envelope: Envelope,
    timer: Timer,
}

impl PulseChannel {
    pub fn new(id: PulseChannelId) -> Self {
        PulseChannel {
            id,
            enabled: false,
            length_counter: LengthCounter::default(),
            envelope: Envelope::default(),
            duty_mode: 0,
            duty_cycle: 0,
            timer: Timer::default(),
            sweep_enabled: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_reload: false,
            sweep_divider: 0,
            sweep_mute: false,
        }
    }

    pub fn step_timer(&mut self) {
        if self.timer.step() {
            self.duty_cycle = (self.duty_cycle + 1) & 7;
        }
    }

    pub fn step_length_counter(&mut self) {
        self.length_counter.step();
    }

    pub fn step_envelope(&mut self) {
        self.envelope.step();
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        if !enabled {
            self.length_counter.reset_to_zero();
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x4000 | 0x4004 => {
                self.duty_mode = (val >> 6) & 0b11;
                let halt_length_counter = val & 0b0010_0000 != 0;
                self.length_counter.set_enabled(!halt_length_counter);
                self.envelope.looping = halt_length_counter;
                self.envelope.constant_mode = val & 0b0001_0000 != 0;
                self.envelope.period = val & 0b1111;
                self.envelope.constant_volume = val & 0b1111;
                self.envelope.start = true;
            }
            0x4001 | 0x4005 => {
                self.sweep_enabled = val & 0b1000_0000 != 0;
                self.sweep_period = (val >> 4) & 0b111;
                self.sweep_negate = val & 0b1000 != 0;
                self.sweep_shift = val & 0b111;
                self.sweep_reload = true;
            }
            0x4002 | 0x4006 => {
                self.timer.period = (self.timer.period & 0xFF00) | (val as u16);
            }
            0x4003 | 0x4007 => {
                self.timer.period = (self.timer.period & 0x00FF) | (((val & 7) as u16) << 8);
                self.duty_cycle = 0;
                self.length_counter.set(val >> 3);
            }
            _ => {}
        }
    }

    fn sweep_target_period(&self) -> u16 {
        let change_amount = self.timer.period >> self.sweep_shift;

        // TODO: Handle difference between pulse 1 and 2

        if self.sweep_negate {
            if change_amount > self.timer.period {
                0
            } else {
                self.timer.period - change_amount
            }
        } else {
            self.timer.period + change_amount
        }
    }

    pub fn step_sweep(&mut self) {
        let target_period = self.sweep_target_period();
        self.sweep_mute = self.timer.period < 8 || target_period > 0x7FF;

        if self.sweep_divider == 0 && self.sweep_enabled && !self.sweep_mute {
            self.timer.period = target_period;
        }

        if self.sweep_divider == 0 || self.sweep_reload {
            self.sweep_divider = self.sweep_period;
            self.sweep_reload = false;
        } else {
            self.sweep_divider -= 1;
        }
    }

    pub fn is_length_counter_active(&self) -> bool {
        !self.length_counter.is_zero()
    }

    pub fn output(&self) -> u8 {
        if !self.enabled
            || self.sweep_mute
            || self.length_counter.is_zero()
            || DUTY_TABLE[self.duty_mode as usize][self.duty_cycle as usize] == 0
        {
            return 0;
        }

        self.envelope.output()
    }
}
