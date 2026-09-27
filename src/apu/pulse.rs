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
                if self.enabled {
                    self.length_counter.set(val >> 3);
                }
                self.envelope.start = true;
            }
            _ => {}
        }
    }

    fn sweep_target_period(&self) -> u16 {
        let change_amount = self.timer.period >> self.sweep_shift;

        if self.sweep_negate {
            self.timer
                .period
                .wrapping_sub(change_amount)
                .wrapping_sub(matches!(self.id, PulseChannelId::Pulse1) as u16)
        } else {
            self.timer.period + change_amount
        }
    }

    pub fn step_sweep(&mut self) {
        let target_period = self.sweep_target_period();
        self.sweep_mute = self.timer.period < 8 || target_period > 0x7FF;

        if self.sweep_divider == 0
            && self.sweep_enabled
            && self.sweep_shift != 0
            && !self.sweep_mute
        {
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
            || self.timer.period < 8
            || (!self.sweep_negate && self.sweep_target_period() > 0x7ff)
            || self.length_counter.is_zero()
            || DUTY_TABLE[self.duty_mode as usize][self.duty_cycle as usize] == 0
        {
            return 0;
        }

        self.envelope.output()
    }
}

crate::savestate::state_fields!(
    PulseChannel,
    id,
    enabled,
    duty_mode,
    duty_cycle,
    sweep_enabled,
    sweep_period,
    sweep_negate,
    sweep_shift,
    sweep_reload,
    sweep_divider,
    sweep_mute,
    length_counter,
    envelope,
    timer
);

impl crate::savestate::StateValue for PulseChannelId {
    fn put(&self, d: &mut crate::savestate::ByteBuffer) {
        d.write_u8(matches!(self, Self::Pulse2) as u8);
    }
    fn get(
        &mut self,
        d: &mut crate::savestate::ByteBuffer,
    ) -> Result<(), crate::savestate::SaveStateError> {
        *self = match d.read_u8()? {
            0 => Self::Pulse1,
            1 => Self::Pulse2,
            _ => return Err(crate::savestate::SaveStateError::InvalidData),
        };
        Ok(())
    }
}
impl PulseChannel {
    pub(super) fn valid(&self) -> bool {
        self.duty_mode < 4
            && self.duty_cycle < 8
            && self.sweep_shift < 8
            && self.timer.period <= 0x7ff
            && self.timer.counter <= 0x7ff
            && self.envelope.valid()
    }
}
