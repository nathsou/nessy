#[derive(Default)]
pub struct Timer {
    pub counter: u16,
    pub period: u16,
}

impl Timer {
    #[inline]
    pub fn step(&mut self) -> bool {
        if self.counter == 0 {
            self.counter = self.period;
            true
        } else {
            self.counter -= 1;
            false
        }
    }
}

#[rustfmt::skip]
const LENGTH_LOOKUP: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6,
    160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

#[derive(Default)]
pub struct LengthCounter {
    enabled: bool,
    counter: u8,
}

impl LengthCounter {
    #[inline]
    pub fn reset_to_zero(&mut self) {
        self.counter = 0;
    }

    #[inline]
    pub fn step(&mut self) {
        if self.enabled && self.counter > 0 {
            self.counter -= 1;
        }
    }

    #[inline]
    pub fn set(&mut self, val: u8) {
        self.counter = LENGTH_LOOKUP[val as usize];
    }

    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.counter == 0
    }
}

#[derive(Default)]
pub struct Envelope {
    pub constant_mode: bool,
    pub looping: bool,
    pub start: bool,
    pub constant_volume: u8,
    pub period: u8,
    pub divider: u8,
    pub decay: u8,
}

impl Envelope {
    pub fn step(&mut self) {
        if self.start {
            self.start = false;
            self.decay = 15;
            self.divider = self.period;
        } else if self.divider == 0 {
            if self.decay > 0 {
                self.decay -= 1;
            } else if self.looping {
                self.decay = 15;
            }

            self.divider = self.period;
        } else {
            self.divider -= 1;
        }
    }

    #[inline]
    pub fn output(&self) -> u8 {
        if self.constant_mode {
            self.constant_volume
        } else {
            self.decay
        }
    }
}
