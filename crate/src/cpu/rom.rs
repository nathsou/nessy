use super::mappers::nrom::NROM;
use super::mappers::Mapper;
use std::io;

const PRG_ROM_PAGE_SIZE: usize = 16384;

#[allow(clippy::upper_case_acronyms)]
pub struct ROM {
    pub bytes: Vec<u8>,
    pub prg_rom_size: u8, // 16kb units
    pub chr_rom_size: u8, // 8kb units
    pub mirroring: Mirroring,
    pub mapper: u8,
    pub battery: bool,
    pub trainer: bool,
    pub prg_rom_start: usize,
    pub chr_rom_start: usize,
}

#[derive(Debug)]
pub enum RomError {
    InvalidiNesHeader,
    UnsupportedMapper(u8),
    IOError(io::Error),
}

#[derive(Debug, PartialEq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
}

impl ROM {
    pub fn new(bytes: Vec<u8>) -> Result<ROM, RomError> {
        if bytes[0] != 78 || bytes[1] != 69 || bytes[2] != 83 || bytes[3] != 26 {
            return Err(RomError::InvalidiNesHeader);
        }

        let ines_ver = (bytes[7] >> 2) & 0b11;
        if ines_ver != 0 {
            // only support iNES version 1
            return Err(RomError::InvalidiNesHeader);
        }

        let four_screen = bytes[6] & 0b1000 != 0;
        let vertical_mirroring = bytes[6] & 0b1 != 0;
        let mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let battery = bytes[6] & 0b10 != 0;
        let trainer = bytes[6] & 0b100 != 0;
        let mapper = (bytes[7] & 0b1111_0000) | (bytes[6] >> 4);
        let prg_rom_size = bytes[4];
        let chr_rom_size = bytes[5];

        let prg_rom_start = 16 + if trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + (prg_rom_size as usize) * PRG_ROM_PAGE_SIZE;

        Ok(ROM {
            prg_rom_size,
            chr_rom_size,
            mirroring,
            mapper,
            battery,
            trainer,
            bytes,
            prg_rom_start,
            chr_rom_start,
        })
    }

    pub fn get_mapper(rom: &mut ROM) -> Result<Box<dyn Mapper>, RomError> {
        match rom.mapper {
            0 => Ok(Box::new(NROM::new())),
            _ => Err(RomError::UnsupportedMapper(rom.mapper)),
        }
    }

    pub fn read_chr(&self, addr: u16) -> u8 {
        self.bytes[self.chr_rom_start + addr as usize]
    }

    pub fn get_tile(&self, chr_bank_offset: usize, nth: usize) -> &[u8] {
        let tile_offset = nth * 16;
        let tile_start = self.chr_rom_start + chr_bank_offset + tile_offset;
        let tile_end = tile_start + 15;
        &self.bytes[tile_start..=tile_end]
    }
}
