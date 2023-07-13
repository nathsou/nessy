use self::{noise::NoiseChannel, pulse::PulseChannel, triangle::TriangleChannel};

mod common;
mod noise;
mod pulse;
mod triangle;

const APU_BUFFER_SIZE: usize = 8 * 1024;
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
    triangle: TriangleChannel,
    noise: NoiseChannel,
    current_sample: Option<f32>,
    samples_pushed: u32,
}

#[rustfmt::skip]
const PULSE_MIXER_LOOKUP: [f32; 32] = [
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
const TRIANGLE_MIXER_LOOKUP: [f32; 204] = [
    0.0, 0.006699824, 0.01334502, 0.019936256, 0.02647418, 0.032959443, 0.039392676, 0.0457745, 
    0.052105535, 0.05838638, 0.064617634, 0.07079987, 0.07693369, 0.08301962, 0.08905826, 0.095050134, 
    0.100995794, 0.10689577, 0.11275058, 0.118560754, 0.12432679, 0.13004918, 0.13572845, 0.14136505, 
    0.1469595, 0.15251222, 0.1580237, 0.1634944, 0.16892476, 0.17431524, 0.17966628, 0.1849783, 
    0.19025174, 0.19548698, 0.20068447, 0.20584463, 0.21096781, 0.21605444, 0.22110492, 0.2261196, 
    0.23109888, 0.23604311, 0.24095272, 0.245828, 0.25066936, 0.2554771, 0.26025164, 0.26499328, 
    0.26970237, 0.27437922, 0.27902418, 0.28363758, 0.28821972, 0.29277095, 0.29729152, 0.3017818, 
    0.3062421, 0.31067267, 0.31507385, 0.31944588, 0.32378912, 0.32810378, 0.3323902, 0.3366486, 
    0.3408793, 0.34508255, 0.34925863, 0.35340777, 0.35753027, 0.36162636, 0.36569634, 0.36974037, 
    0.37375876, 0.37775174, 0.38171956, 0.38566244, 0.38958064, 0.39347437, 0.39734384, 0.4011893, 
    0.405011, 0.40880907, 0.41258383, 0.41633546, 0.42006415, 0.42377013, 0.4274536, 0.43111476, 
    0.43475384, 0.43837097, 0.44196644, 0.4455404, 0.449093, 0.45262453, 0.45613506, 0.4596249, 
    0.46309412, 0.46654293, 0.46997157, 0.47338015, 0.47676894, 0.48013794, 0.48348752, 0.4868177, 
    0.49012873, 0.4934207, 0.49669388, 0.49994832, 0.50318426, 0.50640184, 0.5096012, 0.51278245, 
    0.51594585, 0.5190914, 0.5222195, 0.52533007, 0.52842325, 0.5314993, 0.53455836, 0.5376005, 
    0.54062593, 0.5436348, 0.54662704, 0.54960304, 0.55256283, 0.55550647, 0.5584343, 0.56134623, 
    0.5642425, 0.56712323, 0.5699885, 0.5728384, 0.5756732, 0.57849294, 0.5812977, 0.5840876, 
    0.5868628, 0.58962345, 0.59236956, 0.59510136, 0.5978189, 0.6005223, 0.6032116, 0.605887, 
    0.60854864, 0.6111966, 0.6138308, 0.61645156, 0.619059, 0.62165314, 0.624234, 0.62680185, 
    0.6293567, 0.63189864, 0.6344277, 0.6369442, 0.63944805, 0.64193934, 0.64441824, 0.64688486, 
    0.6493392, 0.6517814, 0.6542115, 0.65662974, 0.65903604, 0.6614306, 0.6638134, 0.66618466, 
    0.66854435, 0.6708926, 0.67322946, 0.67555505, 0.67786944, 0.68017274, 0.68246496, 0.6847462, 
    0.6870166, 0.6892762, 0.69152504, 0.6937633, 0.6959909, 0.69820803, 0.7004148, 0.7026111, 
    0.7047972, 0.7069731, 0.7091388, 0.7112945, 0.7134401, 0.7155759, 0.7177018, 0.7198179, 
    0.72192425, 0.72402096, 0.726108, 0.72818565, 0.7302538, 0.73231256, 0.73436195, 0.7364021, 
    0.7384331, 0.7404549, 0.7424676, 0.7444713, 
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
            triangle: TriangleChannel::new(),
            noise: NoiseChannel::new(),
            current_sample: None,
            samples_pushed: 0,
        }
    }

    #[inline]
    fn get_sample(&self) -> f32 {
        // https://www.nesdev.org/wiki/APU_Mixer
        let p1 = self.pulse1.output();
        let p2 = self.pulse2.output();
        let t = self.triangle.output();
        let n = self.noise.output();

        let pulse_out = PULSE_MIXER_LOOKUP[(p1 + p2) as usize];
        let tnd_out = TRIANGLE_MIXER_LOOKUP[(3 * t + 2 * n) as usize];

        pulse_out + tnd_out
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
            0x4000..=0x4003 => self.pulse1.write(addr, val),
            0x4004..=0x4007 => self.pulse2.write(addr, val),
            0x4008..=0x400B => self.triangle.write(addr, val),
            0x400C..=0x400F => self.noise.write(addr, val),
            0x4010..=0x4013 => {} // DMC

            // Control
            0x4015 => {
                self.pulse1.set_enabled(val & 1 != 0);
                self.pulse2.set_enabled(val & 2 != 0);
                self.triangle.set_enabled(val & 4 != 0);
                self.noise.set_enabled(val & 8 != 0);
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

        self.triangle.step_timer();

        if self.cycle & 1 == 0 {
            self.pulse1.step_timer();
            self.pulse2.step_timer();
            self.noise.step_timer();
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
                self.triangle.step_linear_counter();
                self.noise.step_envelope();
            }

            if half_frame {
                self.pulse1.step_length_counter();
                self.pulse2.step_length_counter();
                self.triangle.step_length_counter();
                self.noise.step_length_counter();
                self.pulse1.step_sweep();
                self.pulse2.step_sweep();
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
