mod registers;
use crate::cpu::rom::{Mirroring, ROM};

use self::registers::{Registers, SpriteSize, PPU_CTRL, PPU_MASK, PPU_STATUS};

const PIXELS_PER_TILE: usize = 8;
const TILES_PER_NAMETABLE_BYTE: usize = 4;
const BYTES_PER_PALLETE: usize = 4;
const SPRITE_PALETTES_OFFSET: usize = 0x11;
const WIDTH: usize = 256;
const HEIGHT: usize = 240;
const DEBUG_SCROLL: bool = false;
const SHOW_OPAQUE_MAP: bool = false;

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
    pub rom: ROM,
    pub regs: Registers,
    vram: [u8; 2 * 1024],
    palette: [u8; 32],
    attributes: [u8; 64 * 4],
    pub cycle: usize,
    pub scanline: usize,
    data_buffer: u8,
    nmi_triggered: bool,
    sprite_zero_hit_this_frame: bool,
    opaque_map: [bool; WIDTH * HEIGHT],
}

impl PPU {
    pub fn new(rom: ROM) -> Self {
        PPU {
            rom,
            regs: Registers::new(),
            vram: [0; 2 * 1024],
            palette: [0; 32],
            attributes: [0; 64 * 4],
            cycle: 0,
            scanline: 0,
            data_buffer: 0,
            nmi_triggered: false,
            sprite_zero_hit_this_frame: false,
            opaque_map: [false; WIDTH * HEIGHT],
        }
    }

    #[inline]
    pub fn is_vblank(&self) -> bool {
        self.regs.status.contains(PPU_STATUS::VBLANK_STARTED)
    }

    pub fn set_pixel(frame: &mut [u8], x: usize, y: usize, rgb: (u8, u8, u8)) {
        if (0..WIDTH).contains(&x) && (0..HEIGHT).contains(&y) {
            let offset = (y * WIDTH + x) * 4;
            frame[offset] = rgb.0;
            frame[offset + 1] = rgb.1;
            frame[offset + 2] = rgb.2;
            frame[offset + 3] = 255;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_background_tile(
        &mut self,
        frame: &mut [u8],
        chr_bank_offset: u16,
        nametable_offset: usize,
        nth: usize,
        tile_col: usize,
        tile_row: usize,
        viewport: BoundingBox,
        shift_x: isize,
        shift_y: isize,
    ) {
        let mut tile = [0u8; 16];
        self.rom
            .mapper
            .get_tile(&self.rom.cart, chr_bank_offset, nth, &mut tile);

        for y in 0..PIXELS_PER_TILE {
            let mut plane1 = tile[y];
            let mut plane2 = tile[y + 8];

            for x in (0..PIXELS_PER_TILE).rev() {
                let bit0 = plane1 & 1;
                let bit1 = plane2 & 1;
                let color_idx = (bit1 << 1) | bit0;

                plane1 >>= 1;
                plane2 >>= 1;

                let rgb = self.background_color_at(
                    nametable_offset,
                    tile_col,
                    tile_row,
                    color_idx as usize,
                );

                let pixel_x = tile_col * PIXELS_PER_TILE + x;
                let pixel_y = tile_row * PIXELS_PER_TILE + y;

                if pixel_x >= viewport.x_min
                    && pixel_x < viewport.x_max
                    && pixel_y >= viewport.y_min
                    && pixel_y < viewport.y_max
                {
                    let x = pixel_x as isize + shift_x;
                    let y = pixel_y as isize + shift_y;

                    if x >= 0 && y >= 0 {
                        let x = x as usize;
                        let y = y as usize;
                        let idx = y * WIDTH + x;

                        let color = if SHOW_OPAQUE_MAP {
                            if color_idx == 0 {
                                (0, 0, 0)
                            } else {
                                (255, 255, 255)
                            }
                        } else {
                            rgb
                        };

                        PPU::set_pixel(frame, x, y, color);
                        self.opaque_map[idx] = color_idx != 0;
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_sprite_tile(
        &mut self,
        frame: &mut [u8],
        chr_bank_offset: u16,
        nth: usize,
        tile_x: usize,
        tile_y: usize,
        behind_background: bool,
        flip_horizontally: bool,
        flip_vertically: bool,
        palette: [Option<(u8, u8, u8)>; 4],
    ) {
        let mut tile = [0u8; 16];
        self.rom
            .mapper
            .get_tile(&self.rom.cart, chr_bank_offset, nth, &mut tile);

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

                    let pixel_x = tile_x + x;
                    let pixel_y = tile_y + y;
                    let pixel_idx = pixel_y * WIDTH + pixel_x;
                    if pixel_idx < self.opaque_map.len() {
                        let is_bg_opaque = self.opaque_map[pixel_idx];
                        if !behind_background || !is_bg_opaque {
                            PPU::set_pixel(frame, pixel_x, pixel_y, rgb);
                        }
                    }
                }
            }
        }
    }

    fn background_color_at(
        &self,
        nametable_offset: usize,
        tile_x: usize,
        tile_y: usize,
        color_idx: usize,
    ) -> (u8, u8, u8) {
        let x = tile_x / TILES_PER_NAMETABLE_BYTE; // 4x4 tiles
        let y = tile_y / TILES_PER_NAMETABLE_BYTE;
        let nametable_idx = y * 8 + x; // 1 byte for color info of 4x4 tiles
        let color_byte = self.vram[nametable_offset + 0x3c0 + nametable_idx];

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
            _ => self.palette[palette_offset + color_idx - 1],
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
        frame: &mut [u8],
        nametable_offset: usize,
        viewport: BoundingBox,
        shift_x: isize,
        shift_y: isize,
    ) {
        let chr_bank_offset: u16 = if !self.regs.ctrl.contains(PPU_CTRL::BACKROUND_PATTERN_ADDR) {
            0
        } else {
            0x1000
        };

        for i in 0..0x03c0 {
            let tile_idx = self.vram[nametable_offset + i] as usize;
            let tile_col = i % 32;
            let tile_row = i / 32;

            self.render_background_tile(
                frame,
                chr_bank_offset,
                nametable_offset,
                tile_idx,
                tile_col,
                tile_row,
                viewport,
                shift_x,
                shift_y,
            );
        }
    }

    fn render_sprites(&mut self, frame: &mut [u8]) {
        let sprite_size = self.regs.ctrl.sprite_size();

        // Sprites with lower OAM indices are drawn in front
        for i in (0..WIDTH).step_by(4).rev() {
            let sprite_y = self.attributes[i] as usize + 1;
            let sprite_tile_idx = match sprite_size {
                SpriteSize::Sprite8x8 => self.attributes[i + 1] as usize,
                SpriteSize::Sprite8x16 => (self.attributes[i + 1]) as usize,
            };
            let sprite_attr = self.attributes[i + 2];
            let sprite_x = self.attributes[i + 3] as usize;

            let palette_idx = sprite_attr & 0b11;
            let behind_background = sprite_attr & 0b0010_0000 != 0;
            let flip_horizontally = sprite_attr & 0b0100_0000 != 0;
            let flip_vertically = sprite_attr & 0b1000_0000 != 0;
            let palette = self.sprite_palette(palette_idx);
            let chr_bank_offset: u16 = match sprite_size {
                SpriteSize::Sprite8x8 => {
                    if !self.regs.ctrl.contains(PPU_CTRL::SPRITE_PATTERN_ADDR) {
                        0
                    } else {
                        0x1000
                    }
                }
                SpriteSize::Sprite8x16 => (sprite_tile_idx as u16 & 1) * 0x1000,
            };

            let (top_tile_idx, bot_tile_idx) = {
                use SpriteSize::*;
                match (sprite_size, flip_vertically) {
                    (Sprite8x8, _) => (sprite_tile_idx, None),
                    (Sprite8x16, false) => (sprite_tile_idx, Some(sprite_tile_idx + 1)),
                    (Sprite8x16, true) => (sprite_tile_idx + 1, Some(sprite_tile_idx)),
                }
            };

            self.render_sprite_tile(
                frame,
                chr_bank_offset,
                top_tile_idx,
                sprite_x,
                sprite_y,
                behind_background,
                flip_horizontally,
                flip_vertically,
                palette,
            );

            if let Some(idx) = bot_tile_idx {
                self.render_sprite_tile(
                    frame,
                    chr_bank_offset,
                    idx,
                    sprite_x,
                    sprite_y + 8,
                    behind_background,
                    flip_horizontally,
                    flip_vertically,
                    palette,
                );
            }
        }
    }

    pub fn render_frame(&mut self, frame: &mut [u8]) {
        let base_nametable_addr = self.regs.ctrl.base_nametable_addr();
        let scroll_x = self.regs.scroll.x as usize;
        let scroll_y = self.regs.scroll.y as usize;

        if self.regs.mask.contains(PPU_MASK::SHOW_BACKGROUND) {
            let (nametable1, nametable2): (usize, usize) = match self.rom.cart.mirroring {
                Mirroring::Vertical => match base_nametable_addr {
                    0x2000 | 0x2800 => (0, 0x400),
                    0x2400 | 0x2c00 => (0x400, 0),
                    _ => unreachable!(),
                },
                Mirroring::Horizontal => match base_nametable_addr {
                    0x2000 | 0x2400 => (0, 0x400),
                    0x2800 | 0x2c00 => (0x400, 0),
                    _ => unreachable!(),
                },
                Mirroring::OneScreenLowerBank => (0, 0),
                Mirroring::OneScreenUpperBank => (0x400, 0x400),
                Mirroring::FourScreen => match base_nametable_addr {
                    0x2000 => (0x000, 0x400),
                    0x2400 => (0x400, 0x800),
                    0x2800 => (0x800, 0xc00),
                    0x2c00 => (0xc00, 0x800),
                    _ => unreachable!(),
                },
            };

            self.render_background(
                frame,
                nametable1,
                BoundingBox {
                    x_min: scroll_x,
                    x_max: WIDTH,
                    y_min: scroll_y,
                    y_max: HEIGHT,
                },
                -(scroll_x as isize),
                -(scroll_y as isize),
            );

            if scroll_x > 0 {
                self.render_background(
                    frame,
                    nametable2,
                    BoundingBox {
                        x_min: 0,
                        x_max: scroll_x,
                        y_min: 0,
                        y_max: HEIGHT,
                    },
                    (WIDTH - scroll_x) as isize,
                    0,
                );
            } else if scroll_y > 0 {
                self.render_background(
                    frame,
                    nametable2,
                    BoundingBox {
                        x_min: 0,
                        x_max: WIDTH,
                        y_min: 0,
                        y_max: scroll_y,
                    },
                    0,
                    (HEIGHT - scroll_y) as isize,
                );
            }

            if DEBUG_SCROLL {
                for x in 0..WIDTH {
                    PPU::set_pixel(frame, x, HEIGHT - scroll_y, (255, 0, 255));
                }

                for y in 0..HEIGHT {
                    PPU::set_pixel(frame, WIDTH - scroll_x, y, (0, 255, 0));
                }
            }
        }

        if self.regs.mask.contains(PPU_MASK::SHOW_SPRITES) {
            self.render_sprites(frame);
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
        let y = self.attributes[0] as usize;
        let x = self.attributes[3] as usize;

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
        match self.rom.cart.mirroring {
            Mirroring::Horizontal => match addr {
                0x2000..=0x23ff => addr - 0x2000,        // A
                0x2400..=0x27ff => addr - 0x2400,        // A
                0x2800..=0x2bff => addr - 0x2800 + 1024, // B
                0x2c00..=0x2fff => addr - 0x2c00 + 1024, // B
                _ => unreachable!(),
            },
            Mirroring::Vertical => match addr {
                0x2000..=0x23ff => addr - 0x2000,        // A
                0x2400..=0x27ff => addr - 0x2400 + 1024, // B
                0x2800..=0x2bff => addr - 0x2800,        // A
                0x2c00..=0x2fff => addr - 0x2c00 + 1024, // B
                _ => unreachable!(),
            },
            Mirroring::OneScreenLowerBank => match addr {
                0x2000..=0x23ff => addr - 0x2000, // A
                0x2400..=0x27ff => addr - 0x2400, // A
                0x2800..=0x2bff => addr - 0x2800, // A
                0x2c00..=0x2fff => addr - 0x2c00, // A
                _ => unreachable!(),
            },
            Mirroring::OneScreenUpperBank => match addr {
                0x2000..=0x23ff => addr - 0x2000 + 1024, // B
                0x2400..=0x27ff => addr - 0x2400 + 1024, // B
                0x2800..=0x2bff => addr - 0x2800 + 1024, // B
                0x2c00..=0x2fff => addr - 0x2c00 + 1024, // B
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
                self.data_buffer = self.rom.mapper.read_chr(&self.rom.cart, addr);
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
            0x0000..=0x1fff => self.rom.mapper.write_chr(&mut self.rom.cart, addr, data),
            0x2000..=0x2fff => {
                self.vram[self.vram_mirrored_addr(addr) as usize] = data;
            }
            0x3000..=0x3eff => unreachable!(),
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                let mut addr = addr as usize - 0x3f10;
                if addr >= 32 {
                    addr %= 32;
                }

                self.palette[addr] = data;
            }
            0x3f00..=0x3fff => {
                let mut addr = addr as usize - 0x3f00;
                if addr >= 32 {
                    addr %= 32;
                }

                self.palette[addr] = data;
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
