use super::rom::Cart;

pub mod mmc1;
pub mod nrom;

pub trait Mapper {
    fn read_prg(&mut self, cart: &mut Cart, addr: u16) -> u8;
    fn write_prg(&mut self, cart: &mut Cart, addr: u16, val: u8);
    fn read_chr(&self, cart: &Cart, addr: u16) -> u8;
    fn write_chr(&mut self, cart: &mut Cart, addr: u16, val: u8);

    fn get_tile<'a>(
        &'a self,
        cart: &'a Cart,
        chr_bank_offset: u16,
        nth: usize,
        buffer: &mut [u8; 16],
    ) {
        let offset = chr_bank_offset + (nth * 16) as u16;

        for i in 0..16 {
            buffer[i] = self.read_chr(cart, offset + i as u16);
        }
    }
}
