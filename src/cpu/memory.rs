pub trait Memory {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, val: u8);

    fn read_word(&self, addr: u16) -> u16 {
        let high = self.read_byte(addr + 1) as u16;
        let low = self.read_byte(addr) as u16;

        (high << 8) | low
    }

    fn write_word(&mut self, addr: u16, val: u16) {
        self.write_byte(addr, (val & 0xff) as u8);
        self.write_byte(addr + 1, ((val >> 8) & 0xff) as u8);
    }
}

pub trait MappedMemory {
    fn read_byte(&self, addr: u16, other: &mut Box<dyn Memory>) -> u8;
    fn write_byte(&mut self, addr: u16, val: u8, other: &mut Box<dyn Memory>);

    fn read_word(&self, addr: u16, other: &mut Box<dyn Memory>) -> u16 {
        let high = self.read_byte(addr + 1, other) as u16;
        let low = self.read_byte(addr, other) as u16;

        (high << 8) | low
    }

    fn write_word(&mut self, addr: u16, val: u16, other: &mut Box<dyn Memory>) {
        self.write_byte(addr, (val & 0xff) as u8, other);
        self.write_byte(addr, ((val >> 8) & 0xff) as u8, other);
    }
}
