use super::mappers::mmc1::MMC1;
use super::mappers::nrom::NROM;
use super::mappers::Mapper;
use std::io;

const PRG_ROM_PAGE_SIZE: usize = 16384;

pub struct Cart {
    pub bytes: Vec<u8>,
    pub prg_rom_size: u8, // 16kb units
    pub chr_rom_size: u8, // 8kb units
    pub mirroring: Mirroring,
    pub mapper_id: u8,
    pub battery: bool,
    pub trainer: bool,
    pub prg_rom_start: usize,
    pub chr_rom_start: usize,
}

#[allow(clippy::upper_case_acronyms)]
pub struct ROM {
    pub cart: Cart,
    pub mapper: Box<dyn Mapper>,
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
    OneScreenLowerBank,
    OneScreenUpperBank,
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
        let mapper_id = (bytes[7] & 0b1111_0000) | (bytes[6] >> 4);
        let prg_rom_size = bytes[4];
        let chr_rom_size = bytes[5];

        let prg_rom_start = 16 + if trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + (prg_rom_size as usize) * PRG_ROM_PAGE_SIZE;

        Ok(ROM {
            cart: Cart {
                prg_rom_size,
                chr_rom_size,
                mirroring,
                mapper_id,
                battery,
                trainer,
                bytes,
                prg_rom_start,
                chr_rom_start,
            },
            mapper: ROM::get_mapper(mapper_id)?,
        })
    }

    fn get_mapper(mapper_id: u8) -> Result<Box<dyn Mapper>, RomError> {
        match mapper_id {
            0 => Ok(Box::new(NROM::new())),
            1 => Ok(Box::new(MMC1::new())),
            _ => Err(RomError::UnsupportedMapper(mapper_id)),
        }
    }
}
