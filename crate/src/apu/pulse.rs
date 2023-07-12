use super::LENGTH_LOOKUP;

const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

pub struct PulseChannel {
    pub enabled: bool,
    pub duty_mode: u8,
    pub duty_cycle: u8,
    pub timer: u16,
    pub timer_reload: u16,
    pub length_counter: u8,
    pub halt_length_counter: bool,
    pub envelope_constant_mode: bool,
    pub envelope_constant_volume: u8,
    pub envelope_loop: bool,
    pub envelope_start: bool,
    pub envelope_period: u8,
    pub envelope_divider: u8,
    pub envelope_decay: u8,
}

impl PulseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            duty_mode: 0,
            duty_cycle: 0,
            timer: 0,
            timer_reload: 0,
            length_counter: 0,
            halt_length_counter: false,
            envelope_loop: false,
            envelope_constant_mode: false,
            envelope_start: false,
            envelope_period: 0,
            envelope_divider: 0,
            envelope_decay: 0,
            envelope_constant_volume: 0,
        }
    }

    pub fn step(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_reload;
            self.duty_cycle = (self.duty_cycle + 1) & 7;
        } else {
            self.timer -= 1;
        }
    }

    pub fn step_length_counter(&mut self) {
        if self.length_counter > 0 && !self.halt_length_counter {
            self.length_counter -= 1;
        }
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

    pub fn write_control(&mut self, val: u8) {
        self.duty_mode = (val >> 6) & 0b11;
        self.halt_length_counter = val & 0b0010_0000 != 0;
        self.envelope_loop = self.halt_length_counter;
        self.envelope_constant_mode = val & 0b0001_0000 != 0;
        self.envelope_period = val & 0b1111;
        self.envelope_constant_volume = val & 0b1111;
        self.envelope_start = true;
    }

    #[inline]
    pub fn write_reload_low(&mut self, val: u8) {
        self.timer_reload = (self.timer_reload & 0xFF00) | (val as u16);
    }

    #[inline]
    pub fn write_reload_high(&mut self, val: u8) {
        self.timer_reload = (self.timer_reload & 0x00FF) | (((val & 7) as u16) << 8);
        self.duty_cycle = 0;
        self.envelope_start = true;
        self.length_counter = LENGTH_LOOKUP[val as usize >> 3];
    }

    pub fn output(&self) -> u8 {
        if !self.enabled
            || self.timer_reload < 8
            || self.timer_reload > 0x7FF
            || self.length_counter == 0
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
