use crate::savestate::{self};

use super::rom::Cart;

pub mod mmc1;
pub mod mmc3;
pub mod nrom;
pub mod unrom;

pub trait Mapper: savestate::Save {
    fn read(&mut self, cart: &mut Cart, addr: u16) -> u8;
    fn write(&mut self, cart: &mut Cart, addr: u16, val: u8);

    fn step_scanline(&mut self) {}

    fn is_asserting_irq(&mut self) -> bool {
        false
    }

    fn get_tile<'a>(
        &'a mut self,
        cart: &'a mut Cart,
        chr_bank_offset: u16,
        nth: usize,
        buffer: &mut [u8; 16],
    ) {
        let offset = chr_bank_offset + (nth * 16) as u16;

        #[allow(clippy::needless_range_loop)]
        for i in 0..16 {
            buffer[i] = self.read(cart, offset + i as u16);
        }
    }
}
