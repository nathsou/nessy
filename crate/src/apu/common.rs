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
    pub fn new() -> Self {
        Self::new()
    }

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
