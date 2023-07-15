use bitflags::bitflags;

use crate::savestate::{self, Save, SaveState, SaveStateError};

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

    pub fn reset(&mut self) {
        self.strobe = false;
        self.index = 0;
        self.status = JoypadStatus::empty();
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

impl savestate::Save for Joypad {
    fn save(&self, parent: &mut savestate::Section) {
        parent.data.write_bool(self.strobe);
        parent.data.write_u8(self.index);
        parent.data.write_u8(self.status.bits());
    }

    fn load(&mut self, parent: &mut savestate::Section) -> Result<(), SaveStateError> {
        self.strobe = parent.data.read_bool()?;
        self.index = parent.data.read_u8()?;
        self.update(parent.data.read_u8()?);

        Ok(())
    }
}
