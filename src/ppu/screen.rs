const WIDTH: usize = 256;
const HEIGHT: usize = 240;

pub struct Screen {
    pub pixels: [u8; WIDTH * HEIGHT * 3],
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            pixels: [0; WIDTH * HEIGHT * 3],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, color: (u8, u8, u8)) {
        let addr = (y * WIDTH + x) * 3;
        self.pixels[addr] = color.0;
        self.pixels[addr + 1] = color.1;
        self.pixels[addr + 2] = color.2;
    }
}
