use sha2::{Digest, Sha256};

use super::mappers::mmc1::MMC1;
use super::mappers::mmc3::MMC3;
use super::mappers::nrom::NROM;
use super::mappers::unrom::UNROM;
use super::mappers::Mapper;

const PRG_ROM_PAGE_SIZE: usize = 16384;

pub struct Cart {
    pub bytes: Vec<u8>,
    pub hash: [u8; 32],
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
    pub mapper: Box<dyn Mapper + Send + Sync>,
}

#[derive(Debug)]
pub enum RomError {
    InvalidiNesHeader,
    InvalidSaveStateHeader,
    UnsupportedMapper(u8),
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

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = hasher.finalize().into();

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
        let cart = Cart {
            bytes,
            hash,
            prg_rom_size,
            chr_rom_size,
            mirroring,
            mapper_id,
            battery,
            trainer,
            prg_rom_start,
            chr_rom_start,
        };

        let mapper = ROM::get_mapper(mapper_id, &cart)?;

        Ok(ROM { mapper, cart })
    }

    fn get_mapper(mapper_id: u8, cart: &Cart) -> Result<Box<dyn Mapper + Send + Sync>, RomError> {
        match mapper_id {
            0 => Ok(Box::new(NROM::new())),
            1 => Ok(Box::new(MMC1::new())),
            2 => Ok(Box::new(UNROM::new())),
            4 => Ok(Box::new(MMC3::new(cart))),
            _ => Err(RomError::UnsupportedMapper(mapper_id)),
        }
    }
}
