use bitflags::bitflags;

use crate::savestate::{Save, SaveState};

bitflags! {
    #[derive(Copy, Clone)]
    pub struct JoypadStatus: u8 {
        const A = 0b0000_0001;
        const B = 0b0000_0010;
        const SELECT = 0b0000_0100;
        const START = 0b0000_1000;
        const UP = 0b0001_0000;
        const DOWN = 0b0010_0000;
        const LEFT = 0b0100_0000;
        const RIGHT = 0b1000_0000;
    }
}

pub struct Joypad {
    strobe: bool,
    index: u8,
    status: JoypadStatus,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            strobe: false,
            index: 0,
            status: JoypadStatus::empty(),
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.index > 7 {
            return 1;
        }

        let pressed = self.status.bits() & (1 << self.index) != 0;

        if !self.strobe && self.index <= 7 {
            self.index += 1;
        }

        if pressed {
            1
        } else {
            0
        }
    }

    pub fn write(&mut self, val: u8) {
        self.strobe = val & 1 == 1;

        if self.strobe {
            self.index = 0;
        }
    }

    pub fn update_button_state(&mut self, button: JoypadStatus, pressed: bool) {
        self.status.set(button, pressed);
    }

    pub fn update(&mut self, val: u8) {
        *self.status.0.bits_mut() = val;
    }
}

impl Save for Joypad {
    fn save(&self, s: &mut SaveState) {
        s.write_bool(self.strobe);
        s.write_u8(self.index);
        s.write_u8(self.status.bits());
    }

    fn load(&mut self, s: &mut SaveState) {
        self.strobe = s.read_bool();
        self.index = s.read_u8();
        self.update(s.read_u8());
    }
}
