use crate::{
    cpu::rom::Cart,
    savestate::{self, SaveStateError},
};

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
    fn read(&mut self, cart: &mut Cart, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let addr = cart.chr_rom_start + addr as usize;
                cart.bytes[addr]
            }
            0x6000..=0x7FFF => self.ram[((addr - 0x6000) & 0x7FF) as usize],
            0x8000..=0xFFFF => {
                let addr = mirrored_addr(cart, addr);
                cart.bytes[addr]
            }
            _ => 0, // _ => panic!("Invalid NROM read address: {:04X}", addr),
        }
    }

    fn write(&mut self, _: &mut Cart, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                // panic!("Attempted to write to CHR ROM on NROM mapper");
            }
            0x6000..=0x7FFF => {
                self.ram[(addr - 0x6000) as usize] = val;
            }
            _ => {
                // panic!("Invalid NROM write address: {:04X}", addr)
            }
        }
    }
}

const NROM_SECTION_NAME: &str = "NROM";

impl savestate::Save for NROM {
    fn save(&self, parent: &mut savestate::Section) {
        let s = parent.create_child(NROM_SECTION_NAME);

        s.data.write_u8_slice(&self.ram);
    }

    fn load(&mut self, parent: &mut savestate::Section) -> Result<(), SaveStateError> {
        let s = parent.get(NROM_SECTION_NAME)?;

        s.data.read_u8_slice(&mut self.ram)?;

        Ok(())
    }
}
