mod registers;
mod screen;

use self::registers::PPU_Registers;
use self::screen::Screen;
use super::cpu::memory::{MappedMemory, Memory};

struct VRAM {
    nametables: [u8; 2 * 1024],
    palette: [u8; 32],
}

impl VRAM {
    pub fn new() -> Self {
        VRAM {
            nametables: [0; 2 * 1024],
            palette: [0; 32],
        }
    }
}

impl MappedMemory for VRAM {
    fn read_byte(&mut self, addr: u16, rom: &mut Box<dyn Memory>) -> u8 {
        if addr < 0x2000 {
            // pattern table
            rom.read_byte(addr)
        } else if addr < 0x3f00 {
            self.nametables[(addr & 2047) as usize]
        // TODO: nametable mirroring
        } else if addr < 0x4000 {
            self.palette[(addr & 31) as usize]
        } else {
            panic!("incorred VRAM read address: {}", addr)
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8, rom: &mut Box<dyn Memory>) {
        if addr < 0x2000 {
            // pattern table
            rom.write_byte(addr, val);
        } else if addr < 0x3f00 {
            self.nametables[(addr & 2047) as usize] = val;
        // TODO: nametable mirroring
        } else if addr < 0x4000 {
            self.palette[(addr & 31) as usize] = val;
        } else {
            panic!("incorred VRAM write address: {}", addr);
        }
    }
}

// https://wiki.nesdev.com/w/index.php/PPU_OAM
struct OAM {
    attributes: [u8; 64 * 4],
}

impl OAM {
    pub fn new() -> Self {
        OAM {
            attributes: [0; 64 * 4],
        }
    }
}

impl Memory for OAM {
    fn read_byte(&mut self, addr: u16) -> u8 {
        self.attributes[addr as usize]
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        self.attributes[addr as usize] = val;
    }
}

pub struct PPU {
    regs: PPU_Registers,
    vram: VRAM,
    oam: OAM,
    screen: Screen,
    cycle: usize,
    scanline: usize,
    data_buffer: u8,
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            regs: PPU_Registers::new(),
            vram: VRAM::new(),
            oam: OAM::new(),
            screen: Screen::new(),
            cycle: 0,
            scanline: 0,
            data_buffer: 0,
        }
    }
}

impl MappedMemory for PPU {
    fn read_byte(&mut self, addr: u16, rom: &mut Box<dyn Memory>) -> u8 {
        match addr & 7 {
            0 => self.regs.ctrl.val,
            1 => self.regs.mask.val,
            2 => self.regs.status.val,
            3 => self.oam.read_byte(self.regs.oam_addr as u16),
            4 => unimplemented!("OAM read"),
            5 => 0, // PPUSCROLL is write-only
            6 => 0, // PPUADDR is write-only
            7 => self.read_ppu_data(rom),
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8, rom: &mut Box<dyn Memory>) {
        match addr & 7 {
            0 => self.regs.ctrl.val = val,
            1 => self.regs.mask.val = val,
            2 => {} // read-only,
            3 => self.regs.oam_addr = val,
            4 => self.write_oam_data(val),
            5 => self.write_ppu_scroll(val),
            6 => self.write_ppu_addr(val),
            7 => self.write_ppu_data(val, rom),
            _ => unreachable!(),
        };
    }
}

impl PPU {
    fn read_ppu_data(&mut self, rom: &mut Box<dyn Memory>) -> u8 {
        let addr = self.regs.addr.addr;
        let val = self.vram.read_byte(addr, rom);
        // increment vram address
        self.regs.addr.addr += self.regs.ctrl.vram_addr_increment();

        if addr < 0x3f00 {
            // https://wiki.nesdev.com/w/index.php/PPU_registers#PPUDATA
            let tmp = self.data_buffer;
            self.data_buffer = val;
            tmp
        } else {
            val
        }
    }

    fn write_oam_data(&mut self, val: u8) {
        self.oam.write_byte(self.regs.oam_addr as u16, val);
        self.regs.oam_addr = self.regs.oam_addr.wrapping_add(1);
    }

    fn write_ppu_addr(&mut self, val: u8) {
        match self.regs.addr.is_high {
            true => {
                self.regs.addr.addr = (self.regs.addr.addr & 0x00ff) | ((val as u16) << 8);
            }
            false => {
                self.regs.addr.addr = (self.regs.addr.addr & 0xff00) | (val as u16);
            }
        };

        self.regs.addr.is_high = !self.regs.addr.is_high;
    }

    fn write_ppu_data(&mut self, val: u8, rom: &mut Box<dyn Memory>) {
        let addr = self.regs.addr.addr;
        self.vram.write_byte(addr, val, rom);
        self.regs.addr.addr += self.regs.ctrl.vram_addr_increment();
    }

    fn write_ppu_scroll(&mut self, val: u8) {
        match self.regs.scroll.is_x {
            true => {
                self.regs.scroll.x = val;
            }
            false => {
                self.regs.scroll.y = val;
            }
        };

        self.regs.scroll.is_x = !self.regs.scroll.is_x;
    }
}
