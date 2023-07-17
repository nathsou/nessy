use crate::{
    cpu::rom::{Cart, Mirroring},
    savestate::{self, SaveStateError},
};

use super::Mapper;

#[allow(clippy::upper_case_acronyms)]
pub struct MMC3 {
    registers: [u8; 8],
    reg: u8,
    prg_mode: u8,
    chr_mode: u8,
    prg_ram: [u8; 0x2000],
    prg_offsets: [u32; 4],
    chr_offsets: [u32; 8],
    irq_enabled: bool,
    irq_reload: u8,
    irq_counter: u8,
    irq_asserted: bool,
}

impl MMC3 {
    pub fn new(cart: &Cart) -> MMC3 {
        MMC3 {
            registers: [0; 8],
            reg: 0,
            prg_mode: 0,
            chr_mode: 0,
            prg_ram: [0; 0x2000],
            prg_offsets: [
                0,
                0x2000,
                (cart.prg_rom_size as u32 * 2 - 2) * 0x2000,
                (cart.prg_rom_size as u32 * 2 - 1) * 0x2000,
            ],
            chr_offsets: [0; 8],
            irq_enabled: false,
            irq_reload: 0,
            irq_counter: 0,
            irq_asserted: false,
        }
    }
}

impl Mapper for MMC3 {
    fn read(&mut self, cart: &mut Cart, addr: u16) -> u8 {
        match addr {
            // PPU
            0x0000..=0x1FFF => {
                let idx = (addr / 0x0400) as usize;
                let offset = self.chr_offsets[idx] as usize + (addr & 0x3FF) as usize;
                cart.bytes[cart.chr_rom_start + offset]
            }
            // CPU
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize],
            0x8000..=0xFFFF => {
                let idx = ((addr - 0x8000) / 0x2000) as usize;
                let offset = self.prg_offsets[idx] as usize + (addr & 0x1FFF) as usize;
                cart.bytes[cart.prg_rom_start + offset]
            }
            _ => {
                panic!("Invalid MMC3 read address: {:04X}", addr);
            }
        }
    }

    fn write(&mut self, cart: &mut Cart, addr: u16, val: u8) {
        match addr {
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize] = val,
            0x8000..=0x9FFF => {
                if addr & 1 == 0 {
                    self.reg = val & 0b111;
                    self.prg_mode = (val >> 6) & 1;
                    self.chr_mode = (val >> 7) & 1;
                } else {
                    let prg_pages = cart.prg_rom_size * 2;
                    self.registers[self.reg as usize] = match self.reg {
                        0..=5 => val,
                        _ => val % prg_pages,
                    };

                    if self.chr_mode == 0 {
                        self.chr_offsets[0] = (self.registers[0] & 0xFE) as u32 * 0x400;
                        self.chr_offsets[1] = (self.registers[0] | 1) as u32 * 0x400;
                        self.chr_offsets[2] = (self.registers[1] & 0xFE) as u32 * 0x400;
                        self.chr_offsets[3] = (self.registers[1] | 1) as u32 * 0x400;
                        self.chr_offsets[4] = (self.registers[2]) as u32 * 0x400;
                        self.chr_offsets[5] = (self.registers[3]) as u32 * 0x400;
                        self.chr_offsets[6] = (self.registers[4]) as u32 * 0x400;
                        self.chr_offsets[7] = (self.registers[5]) as u32 * 0x400;
                    } else {
                        self.chr_offsets[0] = (self.registers[2]) as u32 * 0x400;
                        self.chr_offsets[1] = (self.registers[3]) as u32 * 0x400;
                        self.chr_offsets[2] = (self.registers[4]) as u32 * 0x400;
                        self.chr_offsets[3] = (self.registers[5]) as u32 * 0x400;
                        self.chr_offsets[4] = (self.registers[0] & 0xFE) as u32 * 0x400;
                        self.chr_offsets[5] = (self.registers[0] | 1) as u32 * 0x400;
                        self.chr_offsets[6] = (self.registers[1] & 0xFE) as u32 * 0x400;
                        self.chr_offsets[7] = (self.registers[1] | 1) as u32 * 0x400;
                    }

                    if self.prg_mode == 0 {
                        self.prg_offsets[0] = self.registers[6] as u32 * 0x2000;
                        self.prg_offsets[2] = (prg_pages - 2) as u32 * 0x2000;
                    } else {
                        self.prg_offsets[0] = (prg_pages - 2) as u32 * 0x2000;
                        self.prg_offsets[2] = self.registers[6] as u32 * 0x2000;
                    }

                    self.prg_offsets[1] = self.registers[7] as u32 * 0x2000;
                    self.prg_offsets[3] = (prg_pages - 1) as u32 * 0x2000;
                }
            }
            0xA000..=0xBFFF => {
                if addr & 1 == 0 {
                    cart.mirroring = if val & 1 == 0 {
                        Mirroring::Vertical
                    } else {
                        Mirroring::Horizontal
                    };
                }
            }
            0xC000..=0xDFFF => {
                if addr & 1 == 0 {
                    self.irq_reload = val;
                } else {
                    self.irq_counter = 0;
                }
            }
            0xE000..=0xFFFF => {
                self.irq_enabled = addr & 1 == 1;

                if !self.irq_enabled {
                    self.irq_asserted = false;
                }
            }
            _ => {}
        }
    }

    fn is_asserting_irq(&mut self) -> bool {
        let result = self.irq_asserted;
        self.irq_asserted = false;

        result
    }

    fn step_scanline(&mut self) {
        if self.irq_counter == 0 {
            self.irq_counter = self.irq_reload;
        } else {
            self.irq_counter -= 1;
        }

        if self.irq_counter == 0 && self.irq_enabled {
            self.irq_asserted = true;
        }
    }
}

const MMC3_SECTION_NAME: &str = "MMC3";

impl savestate::Save for MMC3 {
    fn save(&self, parent: &mut savestate::Section) {
        let s = parent.create_child(MMC3_SECTION_NAME);

        s.data.write_u8_slice(&self.registers);
        s.data.write_u8(self.reg);
        s.data.write_u8(self.prg_mode);
        s.data.write_u8(self.chr_mode);
        s.data.write_u8_slice(&self.prg_ram);
        s.data.write_u32_slice(&self.prg_offsets);
        s.data.write_u32_slice(&self.chr_offsets);
        s.data.write_bool(self.irq_enabled);
        s.data.write_u8(self.irq_reload);
        s.data.write_u8(self.irq_counter);
        s.data.write_bool(self.irq_asserted);
    }

    fn load(&mut self, parent: &mut savestate::Section) -> Result<(), SaveStateError> {
        let s = parent.get(MMC3_SECTION_NAME)?;

        s.data.read_u8_slice(&mut self.registers)?;
        self.reg = s.data.read_u8()?;
        self.prg_mode = s.data.read_u8()?;
        self.chr_mode = s.data.read_u8()?;
        s.data.read_u8_slice(&mut self.prg_ram)?;
        s.data.read_u32_slice(&mut self.prg_offsets)?;
        s.data.read_u32_slice(&mut self.chr_offsets)?;
        self.irq_enabled = s.data.read_bool()?;
        self.irq_reload = s.data.read_u8()?;
        self.irq_counter = s.data.read_u8()?;
        self.irq_asserted = s.data.read_bool()?;

        Ok(())
    }
}
