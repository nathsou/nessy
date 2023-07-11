const APU_BUFFER_SIZE: usize = 1024 * 8;
const APU_BUFFER_MASK: u16 = APU_BUFFER_SIZE as u16 - 1;
const CPU_FREQ: f64 = 1789772.5;

enum FrameMode {
    FourStep,
    FiveStep,
}

#[allow(clippy::upper_case_acronyms)]
pub struct APU {
    cycles_per_sample: f64,
    samples_per_frame: f64,
    buffer: [f32; APU_BUFFER_SIZE],
    buffer_write_index: u16,
    buffer_read_index: u16,
    cycle: u32,
    frame_counter: u32,
    frame_interrupt: bool,
    frame_mode: FrameMode,
    pulse1: PulseChannel,
    pulse2: PulseChannel,
    current_sample: Option<f32>,
    samples_pushed: u32,
}

#[rustfmt::skip]
const PULSE_LOOKUP: [f32; 32] = [
    0.0, 0.011609139, 0.02293948, 0.034000948,
    0.044803, 0.05535466, 0.06566453, 0.07574082,
    0.0855914, 0.09522375, 0.10464504, 0.11386215,
    0.12288164, 0.1317098, 0.14035264, 0.14881596,
    0.15710525, 0.16522588, 0.17318292, 0.18098126,
    0.18862559, 0.19612046, 0.20347017, 0.21067894,
    0.21775076, 0.2246895, 0.23149887, 0.23818247,
    0.24474378, 0.25118607, 0.25751257, 0.26372638,
];

#[rustfmt::skip]
const LENGTH_LOOKUP: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6,
    160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

impl APU {
    pub fn new(sound_card_sample_rate: f64) -> APU {
        APU {
            cycles_per_sample: CPU_FREQ / sound_card_sample_rate,
            samples_per_frame: sound_card_sample_rate / 60.0,
            buffer: [0.0; APU_BUFFER_SIZE],
            buffer_write_index: 0,
            buffer_read_index: 0,
            cycle: 0,
            frame_counter: 0,
            frame_interrupt: false,
            frame_mode: FrameMode::FourStep,
            pulse1: PulseChannel::new(),
            pulse2: PulseChannel::new(),
            current_sample: None,
            samples_pushed: 0,
        }
    }

    #[inline]
    fn get_sample(&self) -> f32 {
        // https://www.nesdev.org/wiki/APU_Mixer
        let p1 = self.pulse1.output();
        let p2 = self.pulse2.output();
        let pulse_out = PULSE_LOOKUP[(p1 + p2) as usize];
        pulse_out
    }

    fn push_sample(&mut self) {
        self.samples_pushed += 1;
        let sample = self.get_sample();
        self.current_sample = Some(sample);
        self.buffer[self.buffer_write_index as usize] = sample;
        self.buffer_write_index = (self.buffer_write_index + 1) & APU_BUFFER_MASK;
    }

    #[inline]
    pub fn pull_sample(&mut self) -> Option<f32> {
        self.current_sample.take()
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // Pulse 1
            0x4000 => self.pulse1.write_control(val),
            0x4001 => {}
            0x4002 => self.pulse1.write_reload_low(val),
            0x4003 => self.pulse1.write_reload_high(val),

            // Pulse 2
            0x4004 => self.pulse2.write_control(val),
            0x4005 => {}
            0x4006 => self.pulse2.write_reload_low(val),
            0x4007 => self.pulse2.write_reload_high(val),
            // Triangle
            0x4008 => {}
            0x4009 => {}
            0x400A => {}
            0x400B => {}
            // Noise
            0x400C => {}
            0x400E => {}
            0x400F => {}
            // DMC
            0x4010 => {}
            0x4011 => {}
            0x4012 => {}
            0x4013 => {}
            // Control
            0x4015 => {
                self.pulse1.enabled = val & 1 != 0;
                self.pulse2.enabled = val & 2 != 0;

                if !self.pulse1.enabled {
                    self.pulse1.length_counter = 0;
                }

                if !self.pulse2.enabled {
                    self.pulse2.length_counter = 0;
                }
            }
            // Frame Counter
            0x4017 => {
                self.frame_mode = if val & 0b1000_0000 == 0 {
                    FrameMode::FourStep
                } else {
                    FrameMode::FiveStep
                };

                if val & 0b0100_0000 != 0 {
                    self.frame_interrupt = false;
                }
            }
            _ => {}
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x4015 => {
                self.frame_interrupt = false;
                0
            }
            _ => 0,
        }
    }

    #[inline]
    fn get_sample_count(&self) -> u32 {
        (self.cycle as f64 / self.cycles_per_sample) as u32
    }

    pub fn step(&mut self) {
        self.cycle += 1;
        let next_sample_count = self.get_sample_count();

        if self.cycle & 1 == 0 {
            self.pulse1.step();
            self.pulse2.step();
            self.frame_counter += 1;

            let mut quarter_frame = false;
            let mut half_frame = false;

            match self.frame_counter {
                3729 => quarter_frame = true,
                7457 => {
                    quarter_frame = true;
                    half_frame = true;
                }
                11186 => quarter_frame = true,
                14915 => {
                    if matches!(self.frame_mode, FrameMode::FourStep) {
                        quarter_frame = true;
                        half_frame = true;
                        self.frame_counter = 0;
                    }
                }
                18641 => {
                    // this only happens in 5 step mode
                    quarter_frame = true;
                    half_frame = true;
                    self.frame_interrupt = true;
                    self.frame_counter = 0;
                }
                _ => {}
            };

            if quarter_frame {
                self.pulse1.step_envelope();
                self.pulse2.step_envelope();
            }

            if half_frame {
                self.pulse1.step_length_counter();
                self.pulse2.step_length_counter();
            }
        }

        if self.samples_pushed != next_sample_count {
            self.push_sample();
        }
    }

    pub fn remaining_buffered_samples(&self) -> u16 {
        if self.buffer_write_index >= self.buffer_read_index {
            self.buffer_write_index - self.buffer_read_index
        } else {
            APU_BUFFER_SIZE as u16 - self.buffer_read_index + self.buffer_write_index
        }
    }

    pub fn remaining_samples_in_frame(&self) -> usize {
        (self.samples_per_frame - (self.samples_pushed as f64 % self.samples_per_frame)) as usize
    }

    pub fn fill(&mut self, buffer: &mut [f32]) {
        let client_buffer_size = buffer.len();

        for i in 0..client_buffer_size {
            buffer[i] = if i < self.buffer_write_index as usize {
                self.buffer[i]
            } else {
                // js::log(&format!("Buffer underflow!"));
                0.0
            };
        }

        for i in client_buffer_size..APU_BUFFER_SIZE {
            self.buffer[i - client_buffer_size] = self.buffer[i];
        }

        self.buffer_write_index = if self.buffer_write_index > client_buffer_size as u16 {
            self.buffer_write_index - client_buffer_size as u16
        } else {
            0
        };
    }
}

const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

struct PulseChannel {
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
