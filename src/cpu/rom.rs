use super::mappers::basic::Basic;
use super::mappers::nrom::NROM;
use super::memory::Memory;
use std::fs::File;
use std::io;
use std::io::prelude::Read;

#[allow(clippy::upper_case_acronyms)]
pub struct ROM {
    pub bytes: Vec<u8>,
    pub prg_rom_size: u8, // 16kb units
    pub chr_rom_size: u8, // 8kb units
    pub mirroring: Mirroring,
    pub mapper: u8,
    pub battery: bool,
    pub trainer: bool,
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

        Ok(ROM {
            prg_rom_size: bytes[4],
            chr_rom_size: bytes[5],
            mirroring,
            mapper: (bytes[7] & 0b1111_0000) | (bytes[6] >> 4),
            battery: bytes[6] & 0b10 != 0,
            trainer: bytes[6] & 0b100 != 0,
            bytes,
        })
    }

    pub fn load(path: &str) -> Result<ROM, RomError> {
        match read_file(path) {
            Ok(bytes) => ROM::new(bytes),
            Err(io_err) => Err(RomError::IOError(io_err)),
        }
    }

    pub fn from_program(prog: &[u8], start_addr: u16) -> ROM {
        let mut bytes = vec![0; 0x10000];
        bytes[(start_addr as usize)..((start_addr as usize) + prog.len())].copy_from_slice(prog);
        bytes[0xfffc] = (start_addr & 0xff) as u8;
        bytes[0xfffd] = ((start_addr >> 8) & 0xff) as u8;

        ROM {
            bytes,
            prg_rom_size: 1,
            chr_rom_size: 0,
            mirroring: Mirroring::Horizontal,
            mapper: 100,
            battery: false,
            trainer: false,
        }
    }

    pub fn get_mapper(self) -> Result<Box<dyn Memory>, RomError> {
        match self.mapper {
            0 => Ok(Box::new(NROM::new(self))),
            100 => Ok(Box::new(Basic { rom: self })),
            _ => Err(RomError::UnsupportedMapper(self.mapper)),
        }
    }
}

fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(32768);
    let mut file = File::open(path)?;
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}
