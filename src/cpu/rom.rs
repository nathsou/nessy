use super::mappers::nrom::NROM;
use super::memory::Memory;
use std::fs::File;
use std::io;
use std::io::prelude::Read;

pub struct ROM {
    pub bytes: Vec<u8>,
    pub prg_rom_size: u8, // 16kb units
    pub chr_rom_size: u8, // 8kb units
    pub mirroring_type: u8,
    pub mapper_type: u8,
    pub battery: bool,
    pub trainer: bool,
}

#[derive(Debug)]
pub enum RomError {
    InvalidiNesHeader,
    UnsupportedMapper(u8),
    IOError(io::Error),
}

impl ROM {
    pub fn new(bytes: Vec<u8>) -> Result<ROM, RomError> {
        if bytes[0] != 78 || bytes[1] != 69 || bytes[2] != 83 || bytes[3] != 26 {
            return Err(RomError::InvalidiNesHeader);
        }

        Ok(ROM {
            prg_rom_size: bytes[4],
            chr_rom_size: bytes[5],
            mirroring_type: (bytes[6] & 1) | (((bytes[6] >> 3) & 1) << 1),
            mapper_type: (bytes[6] >> 4) | (bytes[7] & 0xf0),
            battery: ((bytes[6] >> 1) & 1) != 0,
            trainer: ((bytes[6] >> 2) & 1) != 0,
            bytes,
        })
    }

    pub fn from(path: &str) -> Result<ROM, RomError> {
        match read_file(path) {
            Ok(bytes) => ROM::new(bytes),
            Err(io_err) => Err(RomError::IOError(io_err)),
        }
    }

    pub fn get_mapper(self) -> Result<Box<dyn Memory>, RomError> {
        match self.mapper_type {
            0 => Ok(Box::new(NROM { rom: self })),
            _ => Err(RomError::UnsupportedMapper(self.mapper_type)),
        }
    }
}

fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(32768);
    let mut file = File::open(path)?;
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}
