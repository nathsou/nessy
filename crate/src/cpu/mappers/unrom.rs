use crate::cpu::rom::Cart;

use super::Mapper;

#[allow(clippy::upper_case_acronyms)]
pub struct UNROM {
    prg_ram: [u8; 2048],
    chr_ram: [u8; 0x2000],
    bank: u8,
}

impl UNROM {
    pub fn new() -> Self {
        UNROM {
            prg_ram: [0; 2048],
            chr_ram: [0; 0x2000],
            bank: 0,
        }
    }
}

impl Mapper for UNROM {
    fn read(&mut self, cart: &mut Cart, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                if cart.chr_rom_size == 0 {
                    self.chr_ram[addr as usize]
                } else {
                    let addr = cart.chr_rom_start + (addr & 0x1fff) as usize;
                    cart.bytes[addr]
                }
            }
            0x6000..=0x7FFF => self.prg_ram[((addr - 0x6000) & 0x7FF) as usize],
            0x8000..=0xBFFF => {
                let addr =
                    cart.prg_rom_start + ((self.bank as usize) * 0x4000) + (addr & 0x3FFF) as usize;

                cart.bytes[addr]
            }
            0xC000..=0xFFFF => {
                let addr = cart.prg_rom_start
                    + (cart.prg_rom_size as usize - 1) * 0x4000
                    + (addr & 0x3FFF) as usize;

                cart.bytes[addr]
            }
            _ => 0, // _ => panic!("Invalid NROM read address: {:04X}", addr),
        }
    }

    fn write(&mut self, cart: &mut Cart, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                if cart.chr_rom_size == 0 {
                    self.chr_ram[addr as usize] = val;
                } else {
                    let addr = cart.chr_rom_start + (addr & 0x1fff) as usize;
                    cart.bytes[addr] = val;
                }
            }
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000) as usize] = val;
            }
            0x8000..=0xFFFF => {
                self.bank = val & 0b1111;
            }
            _ => {}
        }
    }
}
