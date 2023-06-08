mod registers;
mod screen;
use crate::cpu::rom::{Mirroring, ROM};
use std::rc::Rc;

use self::registers::{Registers, PPU_CTRL, PPU_STATUS};
use self::screen::Screen;

#[rustfmt::skip]
pub static COLOR_PALETTE: [(u8, u8, u8); 64] = [
   (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E),
   (0xC7, 0x00, 0x28), (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00),
   (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05),
   (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF), (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA),
   (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00), (0xC4, 0x62, 0x00),
   (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21),
   (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF),
   (0xD4, 0x80, 0xFF), (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12),
   (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E), (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF),
   (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D), (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF),
   (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3), (0xFF, 0xD2, 0xB0),
   (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA),
   (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11),
];

#[allow(clippy::upper_case_acronyms)]
pub struct PPU {
    rom: Rc<ROM>,
    pub regs: Registers,
    vram: [u8; 2 * 1024],
    palette: [u8; 32],
    attributes: [u8; 64 * 4],
    pub screen: Screen,
    pub cycle: usize,
    pub scanline: u16,
    data_buffer: u8,
    nmi_triggered: bool,
}

impl PPU {
    pub fn new(rom: Rc<ROM>) -> Self {
        PPU {
            rom,
            regs: Registers::new(),
            vram: [0; 2 * 1024],
            palette: [0; 32],
            attributes: [0; 64 * 4],
            screen: Screen::new(),
            cycle: 0,
            scanline: 0,
            data_buffer: 0,
            nmi_triggered: false,
        }
    }

    #[inline]
    pub fn is_vblank(&self) -> bool {
        self.regs.status.contains(PPU_STATUS::VBLANK_STARTED)
    }

    pub fn render_tile(&mut self, bank: usize, nth: usize, x_offset: usize, y_offset: usize) {
        let bank_offset = bank * 0x1000;
        let tile_offset = nth * 16;
        let tile_start = self.rom.chr_rom_start + bank_offset + tile_offset;
        let tile_end = tile_start + 15;
        let tile = &self.rom.bytes[tile_start..=tile_end];

        for y in 0..8 {
            let mut plane1 = tile[y];
            let mut plane2 = tile[y + 8];

            for x in (0..8).rev() {
                let bit0 = plane1 & 1;
                let bit1 = plane2 & 1;
                let color = (bit0 << 1) | bit1;

                plane1 >>= 1;
                plane2 >>= 1;

                let rgb = match color {
                    0 => COLOR_PALETTE[0x01],
                    1 => COLOR_PALETTE[0x23],
                    2 => COLOR_PALETTE[0x27],
                    3 => COLOR_PALETTE[0x30],
                    _ => unreachable!(),
                };

                self.screen.set(x_offset + x, y_offset + y, rgb);
            }
        }
    }

    pub fn show_tile_bank(&mut self, bank: usize) {
        for x in 0..16 {
            for y in 0..16 {
                let nth = x + y * 16;
                self.render_tile(bank, nth, x * 8, y * 8);
            }
        }
    }

    pub fn render_frame(&mut self) {
        let base_nametable_addr = self.regs.ctrl.base_nametable_addr();
        let chr_bank = if self.regs.ctrl.contains(PPU_CTRL::BACKROUND_PATTERN_ADDR) {
            1usize
        } else {
            0
        };

        for i in 0..0x03c0 {
            let tile_idx = self.vram[i] as usize;
            let tile_x = i % 32;
            let tile_y = i / 32;
            self.render_tile(chr_bank, tile_idx, tile_x * 8, tile_y * 8);
        }
    }

    pub fn step(&mut self) {
        self.cycle += 1;

        if self.cycle == 341 {
            self.cycle = 0;
            self.scanline += 1;

            // VBlank
            if self.scanline == 241 {
                self.regs.status.set(PPU_STATUS::VBLANK_STARTED, true);
                self.regs.status.set(PPU_STATUS::SPRITE_ZERO_HIT, false);
                if self.regs.ctrl.contains(PPU_CTRL::GENERATE_NMI) {
                    self.nmi_triggered = true;
                }
            }

            if self.scanline == 262 {
                self.scanline = 0;
                self.regs.status.set(PPU_STATUS::VBLANK_STARTED, false);
                self.regs.status.set(PPU_STATUS::SPRITE_ZERO_HIT, false);
            }
        }
    }

    pub fn pull_nmi_status(&mut self) -> bool {
        let triggered = self.nmi_triggered;
        self.nmi_triggered = false;
        triggered
    }

    fn increment_vram_addr(&mut self) {
        self.regs
            .addr
            .increment(self.regs.ctrl.vram_addr_increment())
    }

    fn vram_mirrored_addr(&self, addr: u16) -> u16 {
        match self.rom.mirroring {
            Mirroring::Horizontal => match addr {
                0x2000..=0x23ff => addr - 0x2000,        // A
                0x2400..=0x27ff => addr - 0x2400,        // A
                0x2800..=0x2bff => 1024 + addr - 0x2800, // B
                0x2c00..=0x2fff => 1024 + addr - 0x2c00, // B
                _ => unreachable!(),
            },
            Mirroring::Vertical => match addr {
                0x2000..=0x23ff => addr - 0x2000,        // A
                0x2400..=0x27ff => 1024 + addr - 0x2400, // B
                0x2800..=0x2bff => addr - 0x2800,        // A
                0x2c00..=0x2fff => 1024 + addr - 0x2c00, // B
                _ => unreachable!(),
            },
            Mirroring::FourScreen => addr - 0x2000,
        }
    }

    pub fn write_ctrl_reg(&mut self, data: u8) {
        // the PPU immediately triggers a NMI when the VBlank flag transitions from 0 to 1 during VBlank
        let prev_nmi_status = self.regs.ctrl.contains(PPU_CTRL::GENERATE_NMI);
        self.regs.ctrl.update(data);
        let new_nmi_status = self.regs.ctrl.contains(PPU_CTRL::GENERATE_NMI);

        if self.is_vblank() && !prev_nmi_status && new_nmi_status {
            self.nmi_triggered = true;
        }
    }

    pub fn read_data_reg(&mut self) -> u8 {
        let addr = self.regs.addr.addr;
        self.increment_vram_addr();

        match addr {
            0x0000..=0x1fff => {
                let res = self.data_buffer;
                self.data_buffer = self.rom.read_chr(addr);
                res
            }
            0x2000..=0x2fff => {
                let res = self.data_buffer;
                self.data_buffer = self.vram[self.vram_mirrored_addr(addr) as usize];
                res
            }
            0x3000..=0x3eff => unreachable!(),
            0x3f00..=0x3fff => self.palette[addr as usize - 0x3f00],
            _ => unreachable!(),
        }
    }

    pub fn write_data_reg(&mut self, data: u8) {
        let addr = self.regs.addr.addr;

        match addr {
            0x0000..=0x1fff => panic!("cannot write to CHR ROM"),
            0x2000..=0x2fff => {
                self.vram[self.vram_mirrored_addr(addr) as usize] = data;
            }
            0x3000..=0x3eff => unreachable!(),
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                self.palette[addr as usize - 0x3f10] = data;
            }
            0x3f00..=0x3fff => {
                self.palette[addr as usize - 0x3f00] = data;
            }
            _ => {
                println!("ignoring write to {addr:04X}");
            }
        }

        self.increment_vram_addr();
    }

    pub fn read_oam_data_reg(&mut self) -> u8 {
        self.regs.oam_data[self.regs.oam_addr as usize]
    }

    pub fn write_oam_data_reg(&mut self, data: u8) {
        self.regs.oam_data[self.regs.oam_addr as usize] = data;
        self.regs.oam_addr = self.regs.oam_addr.wrapping_add(1);
    }

    pub fn read_status_reg(&mut self) -> u8 {
        let res = self.regs.status.bits();
        self.regs.status.remove(PPU_STATUS::VBLANK_STARTED);
        // clear the address latch used by PPUSCROLL and PPUADDR
        self.regs.scroll.is_x = true;
        self.regs.addr.is_high = true;
        res
    }

    #[inline]
    pub fn write_scroll_reg(&mut self, data: u8) {
        self.regs.scroll.write(data);
    }

    #[inline]
    pub fn write_addr_reg(&mut self, data: u8) {
        self.regs.addr.write(data);
    }
}
