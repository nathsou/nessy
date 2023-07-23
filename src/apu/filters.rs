use std::f32::consts::PI;

// first order IIR filter: y[n] = B0 * x[n] + B1 * x[n-1] - A1 * y[n-1]
pub struct Filter {
    b0: f32,
    b1: f32,
    a1: f32,
    prev_x: f32,
    prev_y: f32,
}

impl Filter {
    pub fn new_low_pass(sample_rate: f32, cutoff: f32) -> Filter {
        let c = sample_rate / (cutoff * PI);
        let a0 = 1.0 / (1.0 + c);

        Filter {
            b0: a0,
            b1: a0,
            a1: (1.0 - c) * a0,
            prev_x: 0.0,
            prev_y: 0.0,
        }
    }

    pub fn new_high_pass(sample_rate: f32, cutoff: f32) -> Filter {
        let c = sample_rate / (cutoff * PI);
        let a0 = 1.0 / (1.0 + c);

        Filter {
            b0: c * a0,
            b1: -c * a0,
            a1: (1.0 - c) * a0,
            prev_x: 0.0,
            prev_y: 0.0,
        }
    }

    pub fn filter(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.prev_x - self.a1 * self.prev_y;
        self.prev_x = x;
        self.prev_y = y;
        y
    }
}
