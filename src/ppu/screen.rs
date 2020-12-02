const WIDTH: u16 = 256;
const HEIGHT: u16 = 240;

pub struct Screen {
    pub pixels: [u8; WIDTH as usize * HEIGHT as usize * 3],
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            pixels: [0; WIDTH as usize * HEIGHT as usize * 3],
        }
    }

    pub fn set(&mut self, x: u16, y: u16, r: u8, g: u8, b: u8) {
        let addr = (y * WIDTH + x) as usize;
        self.pixels[addr] = r;
        self.pixels[addr + 1] = g;
        self.pixels[addr + 2] = b;
    }
}
