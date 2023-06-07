use super::super::Memory;
use super::super::ROM;
use std::rc::Rc;

#[allow(clippy::upper_case_acronyms)]
pub struct NROM {
    pub rom: Rc<ROM>,
}

impl NROM {
    pub fn new(rom: Rc<ROM>) -> Self {
        NROM { rom }
    }
}

impl Memory for NROM {
    fn read_byte(&mut self, addr: u16) -> u8 {
        let mut prg_rom_addr = addr as usize - 0x8000;

        if self.rom.prg_rom_size == 1 && prg_rom_addr >= 0x4000 {
            prg_rom_addr -= 0x4000;
        }

        self.rom.bytes[self.rom.prg_rom_start + prg_rom_addr]
    }

    fn write_byte(&mut self, _: u16, _: u8) {
        panic!("cannot write to Nrom")
    }
}
