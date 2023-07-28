use super::common::Timer;

const DELTA_MODULATION_RATES: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

#[derive(Default)]
pub struct DeltaModulationChannel {
    enabled: bool,
    pub interrupt_flag: bool,
    loop_flag: bool,
    output_level: u8,
    sample_addr: u16,
    sample_len: u16,
    current_addr: u16,
    bytes_remaining: u16,
    shift_register: u8,
    silence_flag: bool,
    output_bits_remaining: u8,
    irq_enabled: bool,
    pub cpu_stall: u32,
    pub memory_read_request: Option<u16>,
    timer: Timer,
}

impl DeltaModulationChannel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x4010 => {
                self.irq_enabled = val & 0b1000_0000 != 0;
                self.loop_flag = val & 0b0100_0000 != 0;
                let rate_index = (val & 0b1111) as usize;
                self.timer.period = DELTA_MODULATION_RATES[rate_index];
            }
            0x4011 => {
                self.output_level = val & 0b0111_1111;
            }
            0x4012 => {
                self.sample_addr = 0xC000 | ((val as u16) << 6);
            }
            0x4013 => {
                self.sample_len = ((val as u16) << 4) | 1;
            }
            _ => {}
        }
    }

    pub fn step_timer(&mut self) {
        if self.enabled {
            self.step_reader();

            if self.memory_read_request.is_none() && self.timer.step() {
                self.step_shifter();
            }
        }
    }

    pub fn set_memory_read_response(&mut self, val: u8) {
        self.shift_register = val;

        if self.timer.step() {
            self.step_shifter();
        }
    }

    fn restart(&mut self) {
        self.current_addr = self.sample_addr;
        self.bytes_remaining = self.sample_len;
    }

    fn step_reader(&mut self) {
        if self.output_bits_remaining == 0 && self.bytes_remaining > 0 {
            // TODO: the stall duration varies depending on the timing of the read
            self.cpu_stall += 4;

            // the bus will update the shift register with the read value right after the apu.step() call
            // ideally we would have: self.shift_register = bus.read_word(self.current_addr);
            self.memory_read_request = Some(self.current_addr);

            self.output_bits_remaining = 8;
            self.current_addr = match self.current_addr {
                0xFFFF => 0x8000,
                _ => self.current_addr + 1,
            };

            self.bytes_remaining -= 1;

            if self.bytes_remaining == 0 && self.loop_flag {
                self.restart();
            } else if self.bytes_remaining == 0 && self.interrupt_flag {
                self.interrupt_flag = true;
            }
        }
    }

    fn step_shifter(&mut self) {
        if self.output_bits_remaining != 0 {
            if !self.silence_flag {
                match self.shift_register & 1 {
                    1 => {
                        if self.output_level <= 125 {
                            self.output_level += 2
                        }
                    }
                    _ => {
                        if self.output_level >= 2 {
                            self.output_level -= 2
                        }
                    }
                };
            }

            self.shift_register >>= 1;
            self.output_bits_remaining -= 1;
        }
    }

    pub fn is_active(&self) -> bool {
        self.bytes_remaining > 0
    }

    pub fn clear_interrupt_flag(&mut self) {
        self.interrupt_flag = false;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        if !enabled {
            self.bytes_remaining = 0
        } else if self.bytes_remaining == 0 {
            self.restart();
        }
    }

    pub fn output(&self) -> u8 {
        self.output_level
    }
}
