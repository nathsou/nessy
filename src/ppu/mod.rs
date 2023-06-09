mod registers;
pub mod screen;
use crate::cpu::rom::{Mirroring, ROM};
use std::rc::Rc;

use self::registers::{Registers, PPU_CTRL, PPU_MASK, PPU_STATUS};
use self::screen::Screen;

const PIXELS_PER_TILE: usize = 8;
const TILES_PER_NAMETABLE_BYTE: usize = 4;
const BYTES_PER_PALLETE: usize = 4;
const SPRITE_PALETTES_OFFSET: usize = 0x11;

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
    pub scanline: usize,
    data_buffer: u8,
    nmi_triggered: bool,
    sprite_zero_hit_this_frame: bool,
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
            sprite_zero_hit_this_frame: false,
        }
    }

    #[inline]
    pub fn is_vblank(&self) -> bool {
        self.regs.status.contains(PPU_STATUS::VBLANK_STARTED)
    }

    #[allow(clippy::too_many_arguments)]
    fn render_background_tile(
        &mut self,
        chr_bank_offset: usize,
        nth: usize,
        tile_col: usize,
        tile_row: usize,
        viewport: BoundingBox,
        shift_x: isize,
        shift_y: isize,
    ) {
        let tile = self.rom.get_tile(chr_bank_offset, nth);

        for y in 0..PIXELS_PER_TILE {
            let mut plane1 = tile[y];
            let mut plane2 = tile[y + 8];

            for x in (0..PIXELS_PER_TILE).rev() {
                let bit0 = plane1 & 1;
                let bit1 = plane2 & 1;
                let color_idx = (bit1 << 1) | bit0;

                plane1 >>= 1;
                plane2 >>= 1;

                let rgb = self.background_color_at(tile_col, tile_row, color_idx as usize);
                let pixel_x = tile_col * PIXELS_PER_TILE + x;
                let pixel_y = tile_row * PIXELS_PER_TILE + y;

                if pixel_x >= viewport.x_min
                    && pixel_x < viewport.x_max
                    && pixel_y >= viewport.y_min
                    && pixel_y < viewport.y_max
                {
                    self.screen.set(
                        (pixel_x as isize + shift_x) as usize,
                        (pixel_y as isize + shift_y) as usize,
                        rgb,
                    );
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_sprite_tile(
        &mut self,
        chr_bank_offset: usize,
        nth: usize,
        tile_x: usize,
        tile_y: usize,
        flip_horizontally: bool,
        flip_vertically: bool,
        palette: [Option<(u8, u8, u8)>; 4],
    ) {
        let tile = self.rom.get_tile(chr_bank_offset, nth);

        for y in 0..PIXELS_PER_TILE {
            let mut plane1 = tile[y];
            let mut plane2 = tile[y + 8];

            for x in (0..PIXELS_PER_TILE).rev() {
                let bit0 = plane1 & 1;
                let bit1 = plane2 & 1;
                let color_idx = (bit1 << 1) | bit0;

                plane1 >>= 1;
                plane2 >>= 1;

                if let Some(rgb) = palette[color_idx as usize] {
                    let (x, y) = match (flip_horizontally, flip_vertically) {
                        (false, false) => (x, y),
                        (true, false) => (7 - x, y),
                        (false, true) => (x, 7 - y),
                        (true, true) => (7 - x, 7 - y),
                    };

                    self.screen.set(tile_x + x, tile_y + y, rgb);
                }
            }
        }
    }

    fn background_color_at(&self, tile_x: usize, tile_y: usize, color_idx: usize) -> (u8, u8, u8) {
        let x = tile_x / TILES_PER_NAMETABLE_BYTE; // 4x4 tiles
        let y = tile_y / TILES_PER_NAMETABLE_BYTE;
        let nametable_idx = y * 8 + x; // 1 byte for color info of 4x4 tiles
        let color_byte = self.vram[0x03c0 + nametable_idx];

        let block_x = (tile_x % TILES_PER_NAMETABLE_BYTE) / 2;
        let block_y = (tile_y % TILES_PER_NAMETABLE_BYTE) / 2;

        let palette_offset = 1 + 4 * match (block_x, block_y) {
            (0, 0) => color_byte & 0b11,
            (1, 0) => (color_byte >> 2) & 0b11,
            (0, 1) => (color_byte >> 4) & 0b11,
            (1, 1) => (color_byte >> 6) & 0b11,
            _ => unreachable!(),
        } as usize;

        COLOR_PALETTE[match color_idx {
            0 => self.palette[0],
            1 => self.palette[palette_offset],
            2 => self.palette[palette_offset + 1],
            3 => self.palette[palette_offset + 2],
            _ => unreachable!(),
        } as usize]
    }

    // https://www.nesdev.org/wiki/PPU_palettes
    fn sprite_palette(&self, palette_idx: u8) -> [Option<(u8, u8, u8)>; 4] {
        let palette_offset = SPRITE_PALETTES_OFFSET + palette_idx as usize * BYTES_PER_PALLETE;

        [
            None, // transparent
            Some(COLOR_PALETTE[self.palette[palette_offset] as usize]),
            Some(COLOR_PALETTE[self.palette[palette_offset + 1] as usize]),
            Some(COLOR_PALETTE[self.palette[palette_offset + 2] as usize]),
        ]
    }

    fn render_background(
        &mut self,
        nametable_offset: usize,
        viewport: BoundingBox,
        shift_x: isize,
        shift_y: isize,
    ) {
        let bank_offset: usize = if !self.regs.ctrl.contains(PPU_CTRL::BACKROUND_PATTERN_ADDR) {
            0
        } else {
            0x1000
        };

        for i in 0..0x03c0 {
            let tile_idx = self.vram[nametable_offset + i] as usize;
            let tile_col = i % 32;
            let tile_row = i / 32;

            self.render_background_tile(
                bank_offset,
                tile_idx,
                tile_col,
                tile_row,
                viewport,
                shift_x,
                shift_y,
            );
        }
    }

    fn render_sprites(&mut self) {
        let chr_bank_offset: usize = if !self.regs.ctrl.contains(PPU_CTRL::SPRITE_PATTERN_ADDR) {
            0
        } else {
            0x1000
        };

        for i in (0..256).step_by(4).rev() {
            let sprite_y = self.attributes[i] as usize + 1;
            let sprite_tile_idx = self.attributes[i + 1] as usize;
            let sprite_attr = self.attributes[i + 2];
            let sprite_x = self.attributes[i + 3] as usize;

            let palette_idx = sprite_attr & 0b11;
            // let priority = sprite_attr & 0b0010_0000 != 0;
            let flip_horizontally = sprite_attr & 0b0100_0000 != 0;
            let flip_vertically = sprite_attr & 0b1000_0000 != 0;
            let palette = self.sprite_palette(palette_idx);

            self.render_sprite_tile(
                chr_bank_offset,
                sprite_tile_idx,
                sprite_x,
                sprite_y,
                flip_horizontally,
                flip_vertically,
                palette,
            );
        }
    }

    pub fn render_frame(&mut self) {
        let base_nametable_addr = self.regs.ctrl.base_nametable_addr();
        let scroll_x = self.regs.scroll.x as usize;
        let scroll_y = self.regs.scroll.y as usize;

        if self.regs.mask.contains(PPU_MASK::SHOW_BACKGROUND) {
            let (nametable1, nametable2): (usize, usize) = match self.rom.mirroring {
                Mirroring::Vertical => match base_nametable_addr {
                    0x2000 | 0x2800 => (0, 0x400),
                    0x2400 | 0x2c00 => (0x400, 0),
                    _ => unreachable!(),
                },
                _ => (0, 0),
            };

            self.render_background(
                nametable1,
                BoundingBox {
                    x_min: scroll_x,
                    x_max: Screen::WIDTH,
                    y_min: scroll_y,
                    y_max: Screen::HEIGHT,
                },
                -(scroll_x as isize),
                -(scroll_y as isize),
            );

            self.render_background(
                nametable2,
                BoundingBox {
                    x_min: 0,
                    x_max: scroll_x,
                    y_min: 0,
                    y_max: Screen::HEIGHT,
                },
                (256 - scroll_x) as isize,
                0,
            );
        }

        if self.regs.mask.contains(PPU_MASK::SHOW_SPRITES) {
            self.render_sprites();
        }
    }

    pub fn step(&mut self, cycles: usize) {
        // catch up with the CPU
        self.cycle += cycles;

        if self.cycle >= 341 {
            if self.is_sprite_zero_hit() {
                self.regs.status.set(PPU_STATUS::SPRITE_ZERO_HIT, true);
            }

            self.cycle -= 341;
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
                self.sprite_zero_hit_this_frame = false;
                self.regs.status.set(PPU_STATUS::VBLANK_STARTED, false);
                self.regs.status.set(PPU_STATUS::SPRITE_ZERO_HIT, false);
            }
        }
    }

    #[inline]
    fn is_sprite_zero_hit(&mut self) -> bool {
        // TODO: Improve accuracy
        // https://www.nesdev.org/wiki/PPU_OAM#Sprite_zero_hits
        // if self.sprite_zero_hit_this_frame
        //     || !self.regs.mask.contains(PPU_MASK::SHOW_SPRITES)
        //     || !self.regs.mask.contains(PPU_MASK::SHOW_BACKGROUND)
        // {
        //     return false;
        // }

        let y = self.attributes[0] as usize;
        let x = self.attributes[3] as usize;

        // let hit = self.scanline == y
        //     && self.cycle >= x
        //     && self.regs.mask.contains(PPU_MASK::SHOW_SPRITES)
        //     && self.regs.mask.contains(PPU_MASK::SHOW_BACKGROUND)
        //     && !self.sprite_zero_hit_this_frame;

        // if hit {
        //     self.sprite_zero_hit_this_frame = true;
        // }

        // hit

        y == self.scanline && x <= self.cycle && self.regs.mask.contains(PPU_MASK::SHOW_SPRITES)
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
        self.attributes[self.regs.oam_addr as usize]
    }

    pub fn write_oam_data_reg(&mut self, data: u8) {
        self.attributes[self.regs.oam_addr as usize] = data;
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

    pub fn write_oam_dma_reg(&mut self, page: [u8; 256]) {
        if self.regs.oam_addr == 0 {
            self.attributes.copy_from_slice(&page);
        } else {
            for byte in page.iter() {
                self.write_oam_data_reg(*byte);
            }
        }
    }
}

#[derive(Clone, Copy)]
struct BoundingBox {
    x_min: usize,
    x_max: usize,
    y_min: usize,
    y_max: usize,
}
