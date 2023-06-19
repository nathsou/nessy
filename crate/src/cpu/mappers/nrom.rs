use crate::cpu::rom::Cart;

use super::Mapper;

#[allow(clippy::upper_case_acronyms)]
pub struct NROM {
    ram: [u8; 2048],
}

impl NROM {
    pub fn new() -> Self {
        NROM { ram: [0; 2048] }
    }
}

#[inline]
fn mirrored_addr(cart: &Cart, addr: u16) -> usize {
    let mut prg_rom_addr = addr as usize - 0x8000;

    if cart.prg_rom_size == 1 && prg_rom_addr >= 0x4000 {
        prg_rom_addr -= 0x4000;
    }

    cart.prg_rom_start + prg_rom_addr
}

impl Mapper for NROM {
    fn read_prg(&mut self, cart: &mut Cart, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.ram[((addr - 0x6000) & 0x7FF) as usize],
            0x8000..=0xFFFF => {
                let addr = mirrored_addr(cart, addr);
                cart.bytes[addr]
            }
            _ => 0, // _ => panic!("Invalid NROM read address: {:04X}", addr),
        }
    }

    fn write_prg(&mut self, _: &mut Cart, addr: u16, val: u8) {
        match addr {
            0x6000..=0x7FFF => {
                self.ram[(addr - 0x6000) as usize] = val;
            }
            _ => panic!("Invalid NROM write address: {:04X}", addr),
        }
    }

    fn read_chr(&self, cart: &Cart, addr: u16) -> u8 {
        let addr = cart.chr_rom_start + (addr & 0x1fff) as usize;
        cart.bytes[addr]
    }

    fn write_chr(&mut self, _: &mut Cart, _: u16, _: u8) {
        panic!("Attempted to write to CHR ROM on NROM mapper");
    }
}
