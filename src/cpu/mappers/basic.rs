use super::super::Memory;
use super::super::ROM;

#[allow(clippy::upper_case_acronyms)]
pub struct Basic {
    pub rom: ROM,
}

impl Memory for Basic {
    fn read_byte(&self, addr: u16) -> u8 {
        self.rom.bytes[addr as usize]
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        self.rom.bytes[addr as usize] = val;
    }
}
