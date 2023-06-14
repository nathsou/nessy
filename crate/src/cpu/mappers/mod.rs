use super::rom::ROM;

pub mod nrom;

pub trait Mapper {
    fn read_byte(&mut self, rom: &mut ROM, addr: u16) -> u8;
    fn write_byte(&mut self, rom: &mut ROM, addr: u16, val: u8);
}
