use super::ppu::PPU;
use crate::cpu::{memory::Memory, rom::ROM};
use std::rc::Rc;

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
    fn read_byte(&mut self, addr: u16) -> u8 {
        self.ram[RAM::mirrored_addr(addr) as usize]
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        self.ram[RAM::mirrored_addr(addr) as usize] = val;
    }
}

pub enum Interrupt {
    None,
    NMI,
    IRQ,
}

pub struct Bus {
    pub ram: RAM,
    pub rom: Rc<ROM>,
    pub mapper: Box<dyn Memory>,
    pub ppu: PPU,
}

impl Bus {
    pub fn new(rom: ROM) -> Bus {
        let rc = Rc::new(rom);

        Bus {
            ram: RAM { ram: [0; 0x800] },
            ppu: PPU::new(rc.clone()),
            mapper: ROM::get_mapper(rc.clone()).unwrap(),
            rom: rc,
        }
    }

    pub fn pull_interrupt_status(&mut self) -> Interrupt {
        if self.ppu.pull_nmi_status() {
            Interrupt::NMI
        } else {
            Interrupt::None
        }
    }

    pub fn advance_ppu(&mut self, cpu_cycles: usize) {
        let ppu_cycles = cpu_cycles * 3;
        for _ in 0..ppu_cycles {
            self.ppu.step();
        }
    }
}

// https://wiki.nesdev.com/w/index.php/CPU_memory_map
impl Memory for Bus {
    fn read_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1fff => self.ram.read_byte(addr),
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("PPU address {addr:x} is write-only");
            }
            0x2002 => self.ppu.read_status_reg(),
            0x2004 => self.ppu.read_oam_data_reg(),
            0x2007 => self.ppu.read_data_reg(),
            0x2008..=0x3fff => self.read_byte(addr & 0b0010_0000_0000_0111),
            0x4000..=0x4017 => {
                // APU
                0
            }
            0x4020..=0xffff => self.mapper.read_byte(addr),
            _ => {
                println!("ignoring read at address {addr:x}");
                0
            }
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1fff => self.ram.write_byte(addr, val),
            0x2000 => self.ppu.write_ctrl_reg(val),
            0x2001 => self.ppu.regs.mask.val = val,
            0x2002 => panic!("PPU status register is read-only"),
            0x2003 => self.ppu.regs.oam_addr = val,
            0x2004 => self.ppu.write_oam_data_reg(val),
            0x2005 => self.ppu.write_scroll_reg(val),
            0x2006 => self.ppu.write_addr_reg(val),
            0x2007 => self.ppu.write_data_reg(val),
            0x2008..=0x3fff => self.write_byte(addr & 0b0010_0000_0000_0111, val),
            0x4000..=0x4017 => (), // APU
            0x4020..=0xffff => self.mapper.write_byte(addr, val),
            _ => println!("ignoring write at address {addr:x}"),
        }
    }
}
