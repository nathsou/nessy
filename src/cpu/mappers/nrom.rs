use super::super::Memory;
use super::super::ROM;

#[allow(clippy::upper_case_acronyms)]
pub struct NROM {
    pub rom: ROM,
    prg_offset: u16,
}

impl NROM {
    pub fn new(rom: ROM) -> NROM {
        NROM {
            prg_offset: 16 + if rom.trainer { 512 } else { 0 },
            rom,
        }
    }
}

impl Memory for NROM {
    fn read_byte(&self, addr: u16) -> u8 {
        let mut pgr_rom_addr = addr - 0x8000;

        if self.rom.prg_rom_size == 1 && pgr_rom_addr >= 0x4000 {
            pgr_rom_addr -= 0x4000;
        }

        self.rom.bytes[(self.prg_offset + pgr_rom_addr) as usize]
    }

    fn write_byte(&mut self, _: u16, _: u8) {
        panic!("cannot write to Nrom")
    }
}
