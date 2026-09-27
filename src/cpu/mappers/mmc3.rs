use super::Mapper;
use crate::{
    cpu::rom::{Cart, Mirroring},
    savestate::{self, SaveStateError},
};

/// Sharp MMC3B/C: a zero reload value can assert an IRQ on every qualified edge.
#[allow(clippy::upper_case_acronyms)]
pub struct MMC3 {
    registers: [u8; 8],
    reg: u8,
    prg_mode: u8,
    chr_mode: u8,
    prg_ram: [u8; 0x2000],
    chr_ram: [u8; 0x2000],
    ram_control: u8,
    irq_enabled: bool,
    irq_reload: u8,
    irq_counter: u8,
    reload_pending: bool,
    irq_asserted: bool,
    a12: bool,
    a12_low_since: u64,
}
impl MMC3 {
    pub fn new(_: &Cart) -> Self {
        Self {
            registers: [0; 8],
            reg: 0,
            prg_mode: 0,
            chr_mode: 0,
            prg_ram: [0; 0x2000],
            chr_ram: [0; 0x2000],
            ram_control: 0x80,
            irq_enabled: false,
            irq_reload: 0,
            irq_counter: 0,
            reload_pending: false,
            irq_asserted: false,
            a12: false,
            a12_low_since: 0,
        }
    }
    fn chr_offset(&self, addr: u16) -> usize {
        let slot = ((addr as usize >> 10) ^ (self.chr_mode as usize * 4)) & 7;
        let bank = match slot {
            0 => self.registers[0] & 0xfe,
            1 => self.registers[0] | 1,
            2 => self.registers[1] & 0xfe,
            3 => self.registers[1] | 1,
            _ => self.registers[slot - 2],
        };
        bank as usize * 0x400 + (addr as usize & 0x3ff)
    }
    fn clock_irq(&mut self) {
        if self.irq_counter == 0 || self.reload_pending {
            self.irq_counter = self.irq_reload;
        } else {
            self.irq_counter -= 1;
        }
        self.reload_pending = false;
        if self.irq_counter == 0 && self.irq_enabled {
            self.irq_asserted = true;
        }
    }
}
impl Mapper for MMC3 {
    fn cpu_read(&mut self, cart: &mut Cart, addr: u16) -> Option<u8> {
        if (0x6000..=0x7fff).contains(&addr) && self.ram_control & 0x80 == 0 {
            None
        } else {
            Some(self.read(cart, addr))
        }
    }
    fn read(&mut self, cart: &mut Cart, addr: u16) -> u8 {
        match addr {
            0..=0x1fff => {
                let offset = self.chr_offset(addr);
                if cart.chr_rom_size == 0 {
                    self.chr_ram[offset & 0x1fff]
                } else {
                    cart.bytes[cart.chr_rom_start + offset % (cart.chr_rom_size as usize * 0x2000)]
                }
            }
            0x6000..=0x7fff if self.ram_control & 0x80 != 0 => self.prg_ram[addr as usize & 0x1fff],
            0x8000..=0xffff => {
                let pages = cart.prg_rom_size as usize * 2;
                let slot = (addr as usize - 0x8000) / 0x2000;
                let bank = match (slot, self.prg_mode) {
                    (0, 0) | (2, 1) => (self.registers[6] & 0x3f) as usize,
                    (1, _) => (self.registers[7] & 0x3f) as usize,
                    (3, _) => pages - 1,
                    _ => pages - 2,
                } % pages;
                cart.bytes[cart.prg_rom_start + bank * 0x2000 + (addr as usize & 0x1fff)]
            }
            _ => 0,
        }
    }
    fn write(&mut self, cart: &mut Cart, addr: u16, val: u8) {
        match addr {
            0..=0x1fff if cart.chr_rom_size == 0 => {
                let offset = self.chr_offset(addr) & 0x1fff;
                self.chr_ram[offset] = val;
            }
            0x6000..=0x7fff if self.ram_control & 0xc0 == 0x80 => {
                self.prg_ram[addr as usize & 0x1fff] = val
            }
            0x8000..=0x9fff if addr & 1 == 0 => {
                self.reg = val & 7;
                self.prg_mode = (val >> 6) & 1;
                self.chr_mode = val >> 7;
            }
            0x8000..=0x9fff => self.registers[self.reg as usize] = val,
            0xa000..=0xbfff if addr & 1 == 1 => self.ram_control = val,
            0xa000..=0xbfff if cart.mirroring != Mirroring::FourScreen => {
                cart.mirroring = if val & 1 == 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                }
            }
            0xc000..=0xdfff if addr & 1 == 0 => self.irq_reload = val,
            0xc000..=0xdfff => self.reload_pending = true,
            0xe000..=0xffff => {
                self.irq_enabled = addr & 1 != 0;
                if !self.irq_enabled {
                    self.irq_asserted = false;
                }
            }
            _ => {}
        }
    }
    fn is_asserting_irq(&mut self) -> bool {
        self.irq_asserted
    }
    fn step_scanline(&mut self) {
        self.clock_irq();
    }
    fn ppu_address(&mut self, addr: u16, cycle: u64) {
        let a12 = addr & 0x1000 != 0;
        if !a12 && self.a12 {
            self.a12_low_since = cycle;
        }
        if a12 && !self.a12 && cycle.wrapping_sub(self.a12_low_since) >= 10 {
            self.clock_irq();
        }
        self.a12 = a12;
    }
}
impl savestate::Save for MMC3 {
    fn save(&self, parent: &mut savestate::Section) {
        let s = parent.create_child("MMC3");
        s.data.write_u8_slice(&self.registers);
        for v in [
            self.reg,
            self.prg_mode,
            self.chr_mode,
            self.ram_control,
            self.irq_reload,
            self.irq_counter,
        ] {
            s.data.write_u8(v);
        }
        s.data.write_u8_slice(&self.prg_ram);
        s.data.write_u8_slice(&self.chr_ram);
        for v in [
            self.irq_enabled,
            self.reload_pending,
            self.irq_asserted,
            self.a12,
        ] {
            s.data.write_bool(v);
        }
        s.data.write_u64(self.a12_low_since);
    }
    fn load(&mut self, parent: &mut savestate::Section) -> Result<(), SaveStateError> {
        let s = parent.get("MMC3")?;
        s.data.read_u8_slice(&mut self.registers)?;
        self.reg = s.data.read_u8()?;
        self.prg_mode = s.data.read_u8()?;
        self.chr_mode = s.data.read_u8()?;
        if self.reg > 7 || self.prg_mode > 1 || self.chr_mode > 1 {
            return Err(SaveStateError::InvalidData);
        }
        self.ram_control = s.data.read_u8()?;
        self.irq_reload = s.data.read_u8()?;
        self.irq_counter = s.data.read_u8()?;
        s.data.read_u8_slice(&mut self.prg_ram)?;
        s.data.read_u8_slice(&mut self.chr_ram)?;
        self.irq_enabled = s.data.read_bool()?;
        self.reload_pending = s.data.read_bool()?;
        self.irq_asserted = s.data.read_bool()?;
        self.a12 = s.data.read_bool()?;
        self.a12_low_since = s.data.read_u64()?;
        Ok(())
    }
}
