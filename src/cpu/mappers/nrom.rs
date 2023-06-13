use super::super::ROM;
use super::Mapper;

#[allow(clippy::upper_case_acronyms)]
pub struct NROM {}

impl NROM {
    pub fn new() -> Self {
        NROM {}
    }
}

impl NROM {
    #[inline]
    fn mirrored_addr(rom: &ROM, addr: u16) -> usize {
        let mut prg_rom_addr = addr as usize - 0x8000;

        if rom.prg_rom_size == 1 && prg_rom_addr >= 0x4000 {
            prg_rom_addr -= 0x4000;
        }

        rom.prg_rom_start + prg_rom_addr
    }
}

impl Mapper for NROM {
    fn read_byte(&mut self, rom: &mut ROM, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => rom.bytes[addr as usize],
            0x8000..=0xFFFF => {
                let addr = NROM::mirrored_addr(rom, addr);
                rom.bytes[addr]
            }
            _ => panic!("Invalid NROM read address: {:04X}", addr),
        }
    }

    fn write_byte(&mut self, rom: &mut ROM, addr: u16, val: u8) {
        match addr {
            0x6000..=0x7FFF => {
                rom.bytes[addr as usize] = val;
            }
            0x8000..=0xFFFF => {
                let addr = NROM::mirrored_addr(rom, addr);
                rom.bytes[addr] = val;
            }
            _ => panic!("Invalid NROM write address: {:04X}", addr),
        }
    }
}
