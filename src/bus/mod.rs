use self::controller::Joypad;
use super::apu::APU;
use super::ppu::PPU;
use crate::{
    cpu::{memory::Memory, rom::ROM},
    savestate::{self, SaveStateError},
};
pub mod controller;

#[allow(clippy::upper_case_acronyms)]
pub struct RAM([u8; 0x800]);

impl RAM {
    fn mirrored_addr(addr: u16) -> u16 {
        addr & 0b0000_0111_1111_1111
    }
}

impl Memory for RAM {
    fn read_byte(&mut self, addr: u16) -> u8 {
        self.0[RAM::mirrored_addr(addr) as usize]
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        self.0[RAM::mirrored_addr(addr) as usize] = val;
    }
}

pub enum Interrupt {
    None,
    Nmi,
    Irq,
}

pub struct Bus {
    pub ram: RAM,
    pub ppu: PPU,
    pub apu: APU,
    pub joypad1: Joypad,
    pub joypad2: Joypad,
    pub dma_transfer: bool,
}

impl Bus {
    pub fn new(rom: ROM, sample_rate: f64) -> Bus {
        Bus {
            ram: RAM([0; 0x800]),
            ppu: PPU::new(rom),
            apu: APU::new(sample_rate),
            joypad1: Joypad::new(),
            joypad2: Joypad::new(),
            dma_transfer: false,
        }
    }

    pub fn pull_interrupt(&mut self) -> Interrupt {
        if self.ppu.is_asserting_nmi() {
            Interrupt::Nmi
        } else {
            let is_mapper_irq = self.ppu.rom.mapper.is_asserting_irq();
            let is_apu_irq = self.apu.is_asserting_irq();

            if is_mapper_irq || is_apu_irq {
                Interrupt::Irq
            } else {
                Interrupt::None
            }
        }
    }

    pub fn advance(&mut self, cpu_cycles: u32) {
        let ppu_cycles = cpu_cycles * 3;

        for _ in 0..ppu_cycles {
            self.ppu.step();
        }

        for _ in 0..cpu_cycles {
            self.apu.step();

            if let Some(addr) = self.apu.pull_memory_read_request() {
                let val = self.read_byte(addr);
                self.apu.push_memory_read_response(val);
            }
        }
    }
}

// https://wiki.nesdev.com/w/index.php/CPU_memory_map
impl Memory for Bus {
    fn read_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1fff => self.ram.read_byte(addr),
            // 0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
            //     panic!("PPU address {addr:x} is write-only");
            // }
            0x2000..=0x2007 => self.ppu.read_register(addr),
            0x2008..=0x3fff => self.ppu.read_register(0x2000 + (addr & 7)),
            0x4016 => self.joypad1.read(),
            0x4000..=0x4017 => self.apu.read(addr),
            0x4018..=0x401F => {
                // APU and I/O functionality that is normally disabled.
                0
            }
            0x4020..=0xffff => self.ppu.rom.mapper.read(&mut self.ppu.rom.cart, addr),
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1fff => self.ram.write_byte(addr, val),
            0x2000..=0x2007 => self.ppu.write_register(addr, val),
            0x2008..=0x3fff => self.ppu.write_register(0x2000 + (addr & 7), val),
            0x4014 => {
                let mut page = [0u8; 256];
                let high_byte = (val as u16) << 8;

                for low_byte in 0..256u16 {
                    page[low_byte as usize] = self.read_byte(high_byte | low_byte);
                }

                self.ppu.write_oam_dma_reg(page);
                self.dma_transfer = true;
            }
            0x4016 => self.joypad1.write(val),
            0x4000..=0x4017 => self.apu.write(addr, val),
            0x4018..=0x401F => (), // APU and I/O functionality that is normally disabled.
            0x4020..=0xffff => self.ppu.rom.mapper.write(&mut self.ppu.rom.cart, addr, val),
        }
    }
}

const BUS_SECTION_NAME: &str = "bus";
const JOYPADS_SECTION_NAME: &str = "joypads";

impl savestate::Save for Bus {
    fn save(&self, parent: &mut savestate::Section) {
        let s = parent.create_child(BUS_SECTION_NAME);

        s.data.write_u8_slice(&self.ram.0);
        s.data.write_bool(self.dma_transfer);

        self.ppu.save(s);

        let s = parent.create_child(JOYPADS_SECTION_NAME);
        self.joypad1.save(s);
        self.joypad2.save(s);
    }

    fn load(&mut self, parent: &mut savestate::Section) -> Result<(), SaveStateError> {
        let s = parent.get(BUS_SECTION_NAME)?;

        s.data.read_u8_slice(&mut self.ram.0)?;
        self.dma_transfer = s.data.read_bool()?;

        self.ppu.load(s)?;

        let s = parent.get(JOYPADS_SECTION_NAME)?;
        self.joypad1.load(s)?;
        self.joypad2.load(s)?;

        Ok(())
    }
}
