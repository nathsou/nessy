use super::common::{LengthCounter, Timer};

#[rustfmt::skip]
const SEQUENCER_LOOKUP: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
];

#[derive(Default)]
pub struct TriangleChannel {
    enabled: bool,
    control_flag: bool,
    counter_reload: u8,
    timer: Timer,
    length_counter: LengthCounter,
    linear_counter: u8,
    linear_counter_reload: bool,
    duty_cycle: u8,
}

impl TriangleChannel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x4008 => {
                self.control_flag = val & 0b1000_0000 != 0;
                self.counter_reload = val & 0b0111_1111;
            }
            0x400A => {
                self.timer.period = (self.timer.period & 0xFF00) | (val as u16);
            }
            0x400B => {
                self.timer.period = (self.timer.period & 0x00FF) | (((val & 0b111) as u16) << 8);
                self.timer.counter = self.timer.period;
                self.length_counter.set(val >> 3);
                self.linear_counter_reload = true;
            }
            _ => {}
        }
    }

    pub fn step_linear_counter(&mut self) {
        if self.linear_counter_reload {
            self.linear_counter = self.counter_reload;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }

        if !self.control_flag {
            self.linear_counter_reload = false;
        }
    }

    #[inline]
    pub fn step_length_counter(&mut self) {
        self.length_counter.step();
    }

    pub fn step_timer(&mut self) {
        if self.timer.step() && self.linear_counter > 0 && !self.length_counter.is_zero() {
            self.duty_cycle = (self.duty_cycle + 1) & 31;
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        if !enabled {
            self.length_counter.reset_to_zero();
        }
    }

    #[inline]
    pub fn is_length_counter_active(&self) -> bool {
        !self.length_counter.is_zero()
    }

    pub fn output(&self) -> u8 {
        if !self.enabled
            || self.length_counter.is_zero()
            || self.linear_counter == 0
            || self.timer.period <= 2
        {
            0
        } else {
            SEQUENCER_LOOKUP[self.duty_cycle as usize]
        }
    }
}
