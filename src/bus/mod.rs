use super::ppu::PPU;
use crate::cpu::{memory::Memory, rom::ROM};

#[allow(clippy::upper_case_acronyms)]
pub struct RAM {
    pub ram: [u8; 0x800],
}

impl RAM {
    #[inline]
    fn mirrored_addr(addr: u16) -> u16 {
        addr & 0b0000_0111_1111_1111
    }
}

impl Memory for RAM {
    fn read_byte(&self, addr: u16) -> u8 {
        self.ram[RAM::mirrored_addr(addr) as usize]
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        self.ram[RAM::mirrored_addr(addr) as usize] = val;
    }
}

pub struct Bus {
    pub ram: RAM,
    pub mapper: Box<dyn Memory>,
    pub ppu: PPU,
}

impl Bus {
    pub fn new(rom: ROM) -> Bus {
        Bus {
            ram: RAM { ram: [0; 0x800] },
            mapper: rom.get_mapper().unwrap(),
            ppu: PPU::new(),
        }
    }
}

// https://wiki.nesdev.com/w/index.php/CPU_memory_map
impl Memory for Bus {
    // fn read_byte(&self, addr: u16) -> u8 {
    //     self.mapper.read_byte(addr)
    // }

    // fn write_byte(&mut self, addr: u16, val: u8) {
    //     self.mapper.write_byte(addr, val);
    // }

    fn read_byte(&self, addr: u16) -> u8 {
        let res = if addr < 0x2000 {
            self.ram.read_byte(addr)
        } else if addr < 0x4000 {
            todo!("PPU");
            // self.ppu.read_byte(addr, &mut self.rom)
        } else if addr < 0x4018 {
            0 // APU
        } else {
            self.mapper.read_byte(addr)
        };
        
        println!("reading {:04X} = {:02X}", addr, res);

        res
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        let res = if addr < 0x2000 {
            self.ram.write_byte(addr, val);
        } else if addr < 0x4000 {
            self.ppu.write_byte(addr, val, &mut self.mapper);
        } else if addr < 0x4018 {
            // APU
        } else {
            self.mapper.write_byte(addr, val);
        };

        println!("writing {:04X} = {:02X}", addr, val);

        res
    }
}
