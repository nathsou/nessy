const APU_BUFFER_SIZE: usize = 8192;
const CPU_FREQ: f64 = 1789773.0;

#[allow(clippy::upper_case_acronyms)]
pub struct APU {
    sample_rate: f64,
    buffer: [f32; APU_BUFFER_SIZE],
    buffer_index: u16,
    cycle: u32,
    frame_counter: u32,
    pulse1: PulseChannel,
    pulse2: PulseChannel,
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

impl APU {
    pub fn new(sound_card_sample_rate: f64) -> APU {
        APU {
            sample_rate: CPU_FREQ / sound_card_sample_rate,
            buffer: [0.0; APU_BUFFER_SIZE],
            buffer_index: 0,
            cycle: 0,
            frame_counter: 0,
            pulse1: PulseChannel::new(),
            pulse2: PulseChannel::new(),
        }
    }

    #[inline]
    fn get_sample(&self) -> f32 {
        // https://www.nesdev.org/wiki/APU_Mixer
        let p1 = self.pulse1.output();
        let p2 = self.pulse2.output();
        let pulse_out = PULSE_LOOKUP[(p1 + p2) as usize];
        pulse_out * 15.0
    }

    fn push_sample(&mut self) {
        if self.buffer_index < APU_BUFFER_SIZE as u16 {
            self.buffer[self.buffer_index as usize] = self.get_sample();
            self.buffer_index += 1;
        } else {
            // js::log(&format!("Buffer overflow!"));
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // Pulse 1
            0x4000 => self.pulse1.duty_mode = (val >> 6) & 0b11,
            0x4001 => {}
            0x4002 => self.pulse1.write_reload_low(val),
            0x4003 => self.pulse1.write_reload_high(val),
            // Pulse 2
            0x4004 => self.pulse2.duty_mode = (val >> 6) & 0b11,
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
            }
            // Frame Counter
            0x4017 => {}
            _ => {}
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        0
    }

    pub fn step(&mut self) {
        let cycle = self.cycle;
        let new_sample = (cycle as f64 / self.sample_rate) as u32
            != ((cycle + 1) as f64 / self.sample_rate) as u32;

        self.cycle += 1;

        if self.cycle & 1 == 0 {
            self.pulse1.step();
            self.pulse2.step();
        }

        if new_sample {
            self.push_sample();
        }
    }

    pub fn fill(&mut self, buffer: &mut [f32]) {
        let client_buffer_size = buffer.len();

        for i in 0..client_buffer_size {
            buffer[i] = if i < self.buffer_index as usize {
                self.buffer[i]
            } else {
                // js::log(&format!("Buffer underflow!"));
                0.0
            };
        }

        for i in client_buffer_size..APU_BUFFER_SIZE {
            self.buffer[i - client_buffer_size] = self.buffer[i];
        }

        self.buffer_index = if self.buffer_index > client_buffer_size as u16 {
            self.buffer_index - client_buffer_size as u16
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
    pub reload: u16,
}

impl PulseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            duty_mode: 0,
            duty_cycle: 0,
            timer: 0,
            reload: 0,
        }
    }

    pub fn step(&mut self) {
        if self.timer == 0 {
            self.timer = self.reload;
            self.duty_cycle = (self.duty_cycle + 1) & 7;
        } else {
            self.timer -= 1;
        }
    }

    #[inline]
    pub fn write_reload_low(&mut self, val: u8) {
        self.reload = (self.reload & 0xFF00) | (val as u16);
    }

    #[inline]
    pub fn write_reload_high(&mut self, val: u8) {
        self.reload = (self.reload & 0x00FF) | (((val & 7) as u16) << 8);
        self.duty_cycle = 0;
    }

    pub fn output(&self) -> u8 {
        if !self.enabled {
            return 0;
        }

        DUTY_TABLE[self.duty_mode as usize][self.duty_cycle as usize]
    }
}
