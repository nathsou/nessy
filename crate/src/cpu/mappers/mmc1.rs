use crate::cpu::rom::{Cart, Mirroring};

use super::Mapper;

#[allow(clippy::upper_case_acronyms)]
pub struct MMC1 {
    prg_ram: [u8; 0x2000],
    chr_ram: [u8; 0x2000],
    shift_reg: u8,
    control: u8,
    prg_mode: u8,
    chr_mode: u8,
    chr_bank0: u8,
    chr_bank1: u8,
    prg_bank: u8,
}

impl MMC1 {
    pub fn new() -> Self {
        MMC1 {
            prg_ram: [0; 0x2000],
            chr_ram: [0; 0x2000],
            shift_reg: 0b10000,
            control: 0,
            prg_mode: 0,
            chr_mode: 0,
            chr_bank0: 0,
            chr_bank1: 0,
            prg_bank: 0,
        }
    }
}

impl Mapper for MMC1 {
    fn read_prg(&mut self, cart: &mut Cart, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize],
            0x8000..=0xBFFF => {
                let bank = match self.prg_mode {
                    0 | 1 => self.prg_bank & 0xFE,
                    2 => 0,
                    3 => self.prg_bank,
                    _ => unreachable!(),
                };

                let offset = addr as usize - 0x8000;
                let addr = cart.prg_rom_start + (bank as usize * 0x4000) + offset;
                cart.bytes[addr]
            }
            0xC000..=0xFFFF => {
                let bank = match self.prg_mode {
                    0 | 1 => (self.prg_bank & 0xFE) | 1,
                    2 => self.prg_bank,
                    3 => cart.prg_rom_size - 1,
                    _ => unreachable!(),
                };

                let offset = (addr as usize - 0x8000) & 0x3fff;
                let addr = cart.prg_rom_start + (bank as usize * 0x4000) + offset;
                cart.bytes[addr]
            }
            _ => panic!("Invalid MMC1 read address: {:04X}", addr),
        }
    }

    fn write_prg(&mut self, cart: &mut Cart, addr: u16, val: u8) {
        match addr {
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000) as usize] = val;
            }
            0x8000..=0xFFFF => {
                if val & (1 << 7) != 0 {
                    // reset the shift register
                    self.shift_reg = 0b10000;
                    self.write_control(cart, self.control | 0x0C);
                } else {
                    let done = self.shift_reg & 1 == 1;
                    self.shift_reg = ((self.shift_reg >> 1) | ((val & 1) << 4)) & 0b11111;

                    if done {
                        match addr {
                            0x8000..=0x9FFF => {
                                self.write_control(cart, self.shift_reg);
                            }
                            0xA000..=0xBFFF => {
                                self.chr_bank0 = self.shift_reg & 0b11111;
                            }
                            0xC000..=0xDFFF => {
                                self.chr_bank1 = self.shift_reg & 0b11111;
                            }
                            0xE000..=0xFFFF => {
                                self.prg_bank = self.shift_reg & 0b1111;
                            }
                            _ => unreachable!(),
                        }

                        self.shift_reg = 0b10000;
                    }
                }
            }
            _ => panic!("Invalid NROM write address: {:04X}", addr),
        }
    }

    fn read_chr(&self, cart: &Cart, addr: u16) -> u8 {
        if cart.chr_rom_size == 0 {
            self.chr_ram[addr as usize]
        } else {
            let offset = self.chr_rom_offset(cart, addr);
            cart.bytes[cart.chr_rom_start + offset]
        }
    }

    fn write_chr(&mut self, cart: &mut Cart, addr: u16, val: u8) {
        if cart.chr_rom_size == 0 {
            self.chr_ram[addr as usize] = val;
        } else {
            let offset = self.chr_rom_offset(cart, addr);
            cart.bytes[cart.chr_rom_start + offset] = val;
        }
    }
}

impl MMC1 {
    fn chr_rom_offset(&self, _: &Cart, addr: u16) -> usize {
        if self.chr_mode == 0 {
            // switch 8 KB at a time
            self.chr_bank0 as usize * 0x2000 + (addr as usize & 0x1FFF)
        } else {
            // switch two separate 4 KB banks
            (addr as usize & 0xFFF)
                + match addr {
                    0x0000..=0x0FFF => (self.chr_bank0 as usize) * 0x1000,
                    0x1000..=0x1FFF => (self.chr_bank1 as usize) * 0x1000,
                    _ => unreachable!(),
                }
        }
    }

    fn write_control(&mut self, cart: &mut Cart, val: u8) {
        self.control = val;
        // CPPMM
        self.prg_mode = (val >> 2) & 0b11;
        self.chr_mode = (val >> 4) & 1;
        cart.mirroring = match val & 0b11 {
            0 => Mirroring::OneScreenLowerBank,
            1 => Mirroring::OneScreenUpperBank,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => unreachable!(),
        };
    }
}