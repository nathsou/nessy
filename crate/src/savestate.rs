pub struct SaveState {
    data: Vec<u8>,
    read_index: usize,
}

impl SaveState {
    pub fn new() -> Self {
        SaveState {
            data: vec![],
            read_index: 0,
        }
    }

    pub fn from(data: Vec<u8>) -> Self {
        SaveState {
            data,
            read_index: 0,
        }
    }

    pub fn get_data(self) -> Vec<u8> {
        self.data
    }

    pub fn write_slice(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    pub fn write_bool(&mut self, data: bool) {
        self.write_u8(data.into());
    }

    pub fn write_u8(&mut self, data: u8) {
        self.data.push(data)
    }

    pub fn write_u16(&mut self, data: u16) {
        self.data.push((data >> 8) as u8);
        self.data.push((data & 0xff) as u8);
    }

    pub fn write_u32(&mut self, data: u32) {
        self.write_u16((data >> 16) as u16);
        self.write_u16((data & 0xffff) as u16);
    }

    pub fn write_u64(&mut self, data: u64) {
        self.write_u32((data >> 32) as u32);
        self.write_u32((data & 0xffff_ffff) as u32);
    }

    pub fn read_slice(&mut self, dst: &mut [u8]) {
        let slice = &self.data[self.read_index..self.read_index + dst.len()];
        dst.copy_from_slice(slice);
        self.read_index += dst.len();
    }

    pub fn read_bool(&mut self) -> bool {
        self.read_u8() != 0
    }

    pub fn read_u8(&mut self) -> u8 {
        let value = self.data[self.read_index];
        self.read_index += 1;
        value
    }

    pub fn read_u16(&mut self) -> u16 {
        let hi = self.read_u8() as u16;
        let lo = self.read_u8() as u16;
        hi << 8 | lo
    }

    pub fn read_u32(&mut self) -> u32 {
        let hi = self.read_u16() as u32;
        let lo = self.read_u16() as u32;
        hi << 16 | lo
    }

    pub fn read_u64(&mut self) -> u64 {
        let hi = self.read_u32() as u64;
        let lo = self.read_u32() as u64;
        hi << 32 | lo
    }

    pub fn write_all(&mut self, values: &[impl Save]) {
        for value in values {
            value.save(self);
        }
    }

    pub fn read_all(&mut self, values: &mut [impl Save]) {
        for value in values {
            value.load(self);
        }
    }
}

pub trait Save {
    fn save(&self, s: &mut SaveState);
    fn load(&mut self, s: &mut SaveState);
}
