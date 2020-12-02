use super::super::Memory;
use super::super::ROM;

pub struct NROM {
    pub rom: ROM,
}

impl Memory for NROM {
    fn read_byte(&mut self, addr: u16) -> u8 {
        if self.rom.prg_rom_size == 2 {
            self.rom.bytes[16 + (addr & 0x7fff) as usize]
        } else {
            self.rom.bytes[16 + (addr & 0x3fff) as usize]
        }
    }

    fn write_byte(&mut self, _: u16, _: u8) {
        panic!("cannot write to Nrom")
    }
}
