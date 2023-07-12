use super::common::LengthCounter;

const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

#[derive(Default)]
pub struct PulseChannel {
    enabled: bool,
    length_counter: LengthCounter,
    duty_mode: u8,
    duty_cycle: u8,
    timer: u16,
    timer_period: u16,
    envelope_constant_mode: bool,
    envelope_constant_volume: u8,
    envelope_loop: bool,
    envelope_start: bool,
    envelope_period: u8,
    envelope_divider: u8,
    envelope_decay: u8,
    sweep_enabled: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_reload: bool,
    sweep_divider: u8,
    sweep_mute: bool,
}

impl PulseChannel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn step_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            self.duty_cycle = (self.duty_cycle + 1) & 7;
        } else {
            self.timer -= 1;
        }
    }

    #[inline]
    pub fn step_length_counter(&mut self) {
        self.length_counter.step();
    }

    pub fn step_envelope(&mut self) {
        if self.envelope_start {
            self.envelope_start = false;
            self.envelope_decay = 15;
            self.envelope_divider = self.envelope_period;
        } else if self.envelope_divider == 0 {
            if self.envelope_decay > 0 {
                self.envelope_decay -= 1;
            } else if self.envelope_loop {
                self.envelope_decay = 15;
            }

            self.envelope_divider = self.envelope_period;
        } else {
            self.envelope_divider -= 1;
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        if !enabled {
            self.length_counter.reset_to_zero();
        }
    }

    pub fn write_control(&mut self, val: u8) {
        self.duty_mode = (val >> 6) & 0b11;
        let halt_length_counter = val & 0b0010_0000 != 0;
        self.length_counter.set_enabled(!halt_length_counter);
        self.envelope_loop = halt_length_counter;
        self.envelope_constant_mode = val & 0b0001_0000 != 0;
        self.envelope_period = val & 0b1111;
        self.envelope_constant_volume = val & 0b1111;
        self.envelope_start = true;
    }

    #[inline]
    pub fn write_reload_low(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | (val as u16);
    }

    pub fn write_reload_high(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | (((val & 7) as u16) << 8);
        self.duty_cycle = 0;
        self.envelope_start = true;
        self.length_counter.set(val >> 3);
    }

    pub fn write_sweep(&mut self, val: u8) {
        self.sweep_enabled = val & 0b1000_0000 != 0;
        self.sweep_period = (val >> 4) & 0b111;
        self.sweep_negate = val & 0b1000 != 0;
        self.sweep_shift = val & 0b111;
        self.sweep_reload = true;
    }

    fn sweep_target_period(&self) -> u16 {
        let change_amount = self.timer_period >> self.sweep_shift;

        // TODO: Handle difference between pulse 1 and 2

        if self.sweep_negate {
            if change_amount > self.timer_period {
                0
            } else {
                self.timer_period - change_amount
            }
        } else {
            self.timer_period + change_amount
        }
    }

    pub fn step_sweep(&mut self) {
        let target_period = self.sweep_target_period();
        self.sweep_mute = self.timer_period < 8 || target_period > 0x7FF;

        if self.sweep_divider == 0 && self.sweep_enabled && !self.sweep_mute {
            self.timer_period = target_period;
        }

        if self.sweep_divider == 0 || self.sweep_reload {
            self.sweep_divider = self.sweep_period;
            self.sweep_reload = false;
        } else {
            self.sweep_divider -= 1;
        }
    }

    pub fn output(&self) -> u8 {
        if !self.enabled
            || self.sweep_mute
            || self.length_counter.is_zero()
            || DUTY_TABLE[self.duty_mode as usize][self.duty_cycle as usize] == 0
        {
            return 0;
        }

        if self.envelope_constant_mode {
            self.envelope_constant_volume
        } else {
            self.envelope_decay
        }
    }
}
