pub mod registers;

use self::registers::{Ctrl, Mask, Registers, SpriteSize, Status};
use crate::{
    cpu::rom::{Mirroring, ROM},
    savestate::{self, SaveStateError},
};

const PIXELS_PER_TILE: usize = 8;
const BYTES_PER_PALLETE: usize = 4;
const TILES_PER_NAMETABLE_BYTE: usize = 4;
const TILES_PER_NAMETABLE: usize = 32 * 30;
const BYTES_PER_NAMETABLE: usize = 1024;
const SPRITE_PALETTES_OFFSET: usize = 0x11;
const WIDTH: usize = 256;
const HEIGHT: usize = 240;
const FAST_MODE: bool = true;

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

#[derive(Clone, Copy)]
struct SpriteData {
    x: u16,
    idx: u8,
    chr: [u8; 8],
    palette_idx: u8,
    behind_background: bool,
}

#[derive(Clone, Copy)]
struct CachedTile {
    chr: [u8; 64],
}

#[allow(clippy::upper_case_acronyms)]
pub struct PPU {
    pub rom: ROM,
    pub regs: Registers,
    open_bus: u8,
    vram: [u8; 2 * BYTES_PER_NAMETABLE],
    palette: [u8; 32],
    attributes: [u8; 64 * 4],
    pub cycle: u16,
    scanline: u16,
    frame: u64,
    data_buffer: u8,
    nmi_triggered: bool,
    nmi_edge_detector: bool,
    should_trigger_nmi: bool,
    pub frame_complete: bool,
    tile_data: u64,
    nametable_byte: u8,
    attribute_table_byte: u8,
    pattern_table_low_byte: u8,
    pattern_table_high_byte: u8,
    scanline_sprites: [SpriteData; 8],
    visible_sprites_count: u8,
    nt_cache: Box<[Option<CachedTile>; BYTES_PER_NAMETABLE * 2]>,
    updated_bg_tiles: Box<[bool; BYTES_PER_NAMETABLE * 2]>,
    background: Box<[u8; WIDTH * HEIGHT * 3 * 2]>,
}

impl PPU {
    pub fn new(rom: ROM) -> Self {
        let mut ppu = PPU {
            rom,
            regs: Registers::new(),
            open_bus: 0,
            vram: [0; 2 * 1024],
            palette: [0; 32],
            attributes: [0; 64 * 4],
            cycle: 0,
            scanline: 0,
            frame: 0,
            data_buffer: 0,
            nmi_triggered: false,
            should_trigger_nmi: false,
            nmi_edge_detector: false,
            frame_complete: false,
            // background data
            tile_data: 0,
            nametable_byte: 0,
            attribute_table_byte: 0,
            pattern_table_low_byte: 0,
            pattern_table_high_byte: 0,
            scanline_sprites: [SpriteData {
                x: 0,
                idx: 0,
                palette_idx: 0,
                behind_background: false,
                chr: [0; 8],
            }; 8],
            visible_sprites_count: 0,
            nt_cache: Box::new([None; BYTES_PER_NAMETABLE * 2]),
            updated_bg_tiles: Box::new([true; BYTES_PER_NAMETABLE * 2]),
            background: Box::new([0; WIDTH * HEIGHT * 3 * 2]),
        };

        ppu.reset();
        ppu
    }

    pub fn tick_inaccurate(&mut self) {
        if self.should_trigger_nmi
            && self.regs.ctrl.contains(Ctrl::GENERATE_NMI)
            && self.regs.status.contains(Status::VBLANK_STARTED)
        {
            self.nmi_triggered = true;
            self.should_trigger_nmi = false;
        }
    }

    fn tick(&mut self) {
        // TODO: handle NMI delay

        if self.should_trigger_nmi
            && self.regs.ctrl.contains(Ctrl::GENERATE_NMI)
            && self.regs.status.contains(Status::VBLANK_STARTED)
        {
            self.nmi_triggered = true;
            self.should_trigger_nmi = false;
        }

        if self.regs.rendering_enabled() && self.regs.f && self.scanline == 261 && self.cycle == 339
        {
            // skip cycle 339 of pre-render scanline
            self.cycle = 0;
            self.scanline = 0;
            self.regs.f = !self.regs.f;
            self.frame += 1;
            return;
        }

        self.cycle += 1;

        if self.cycle > 340 {
            if FAST_MODE && self.is_sprite_zero_hit() {
                self.regs.status.insert(Status::SPRITE_ZERO_HIT);
            }

            self.cycle = 0;
            self.scanline += 1;

            if self.scanline > 261 {
                self.scanline = 0;
                self.regs.f = !self.regs.f;
                self.frame += 1;
            }
        }

        if self.regs.rendering_enabled() && self.cycle == 260 && self.scanline < 240 {
            self.rom.mapper.step_scanline();
        }
    }

    pub fn step(&mut self, frame: &mut [u8]) {
        self.tick();

        let preline = self.scanline == 261;
        let visible_line = self.scanline < 240;
        let render_line = preline || visible_line;
        let pre_fetch_cycle = self.cycle >= 321 && self.cycle <= 336;
        let visible_cycle = self.cycle >= 1 && self.cycle <= 256;
        let fetch_cycle = pre_fetch_cycle || visible_cycle;

        // background logic
        if !FAST_MODE {
            if self.regs.show_background() {
                if visible_line && visible_cycle {
                    self.render_pixel(frame);
                }

                if render_line && fetch_cycle {
                    self.tile_data <<= 4;

                    match self.cycle & 7 {
                        1 => self.fetch_nametable_byte(),
                        3 => self.fetch_attribute_table_byte(),
                        // 5 => self.fetch_pattern_table_low_byte(),
                        // 7 => self.fetch_pattern_table_high_byte(),
                        7 => self.fetch_pattern_table_bytes(),
                        0 => self.store_background_tile_data(),
                        _ => {}
                    }
                }

                if preline && self.cycle >= 280 && self.cycle <= 304 {
                    self.regs.copy_y();
                }

                if render_line {
                    if fetch_cycle && self.cycle & 7 == 0 {
                        self.regs.increment_x();
                    }

                    if self.cycle == 256 {
                        self.regs.increment_y();
                    }

                    if self.cycle == 257 {
                        self.regs.copy_x();
                    }
                }
            }

            if self.regs.show_sprites() && self.cycle == 257 {
                if visible_line {
                    self.fetch_next_scanline_sprites();
                } else {
                    // clear secondary OAM
                    self.visible_sprites_count = 0;
                }
            }
        }

        if preline && self.cycle == 1 {
            self.end_vblank();
        }

        // VBlank
        if self.scanline == 241 && self.cycle == 1 {
            self.start_vblank();
        }
    }

    pub fn start_vblank(&mut self) {
        self.frame_complete = true;
        self.regs.status.insert(Status::VBLANK_STARTED);
        self.detect_nmi_edge();

        // if FAST_MODE {
        //     self.render_frame();
        // }

        // self.transfer_frame_buffer();
    }

    pub fn end_vblank(&mut self) {
        self.frame_complete = false;
        self.regs.status.remove(Status::VBLANK_STARTED);
        self.regs.status.remove(Status::SPRITE_ZERO_HIT);
        self.regs.status.remove(Status::SPRITE_OVERFLOW);
        self.detect_nmi_edge();
    }

    pub fn render_frame(&mut self, frame: &mut [u8]) {
        let base_nametable_addr = self.regs.ctrl.base_nametable_addr();
        let scroll_x = self.regs.scroll.x as usize;
        // let scroll_y = self.regs.scroll.y as usize;

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

        if self.regs.show_background() {
            frame.fill(0);
            self.render_background(nametable1, 0);

            if scroll_x > 0 {
                self.render_background(nametable2, WIDTH);
            }

            let mut offset = 0;
            let len = WIDTH * 3;
            for y in 0..HEIGHT {
                // for x in 0..WIDTH {
                //     let xs = x + scroll_x;
                //     let bg_idx = (y * 2 * WIDTH + xs) * 3;
                //     let buf_idx = (y * WIDTH + x) * 3;
                //     frame[buf_idx] = self.background[bg_idx];
                //     frame[buf_idx + 1] = self.background[bg_idx + 1];
                //     frame[buf_idx + 2] = self.background[bg_idx + 2];
                // }

                let s = (y * 2 * WIDTH + scroll_x) * 3;

                frame[offset..offset + len]
                    .copy_from_slice(&self.background.as_slice()[s..s + len]);
                offset += len;
            }

            // frame.copy_from_slice(&self.background.as_slice()[..WIDTH * HEIGHT * 3]);
            // self.copy_rect(0, (scroll_x, 0), (WIDTH + scroll_x, HEIGHT), frame);
            // self.copy_rect(
            //     WIDTH - scroll_x,
            //     (WIDTH + scroll_x, 0),
            //     (2 * WIDTH, HEIGHT),
            //     frame,
            // );
        }

        // for y in 0..HEIGHT {
        //     PPU::set_pixel(frame, scroll_x, y, (0, 0, 255));
        //     PPU::set_pixel(frame, (scroll_x + WIDTH) % (2 * WIDTH), y, (0, 0, 255));
        // }

        if self.regs.show_sprites() {
            self.render_sprites(frame);
        }
    }

    fn render_background(&mut self, nt_offset: usize, offset_x: usize) {
        let chr_bank_offset = self.regs.ctrl.background_chr_offset();

        for i in 0..0x03c0 {
            let tile_col = i & 31; // i % 32
            let tile_row = i / 32;

            self.render_background_tile(
                chr_bank_offset,
                nt_offset,
                i,
                tile_col,
                tile_row,
                offset_x,
            );
        }
    }

    fn get_nametable_tile(
        &mut self,
        nt_offset: usize,
        nth: usize,
        chr_bank_offset: u16,
    ) -> [u8; 64] {
        let tile_index = nt_offset + nth;

        match self.nt_cache[tile_index] {
            Some(tile) => tile.chr,
            _ => {
                let mut tile = [0u8; 16];
                self.rom
                    .mapper
                    .get_tile(&mut self.rom.cart, chr_bank_offset, nth, &mut tile);

                let mut chr = [0u8; 64];

                for y in 0..PIXELS_PER_TILE {
                    let mut plane1 = tile[y];
                    let mut plane2 = tile[y + 8];

                    for x in (0..PIXELS_PER_TILE).rev() {
                        let bit0 = plane1 & 1;
                        let bit1 = plane2 & 1;
                        let color_idx = (bit1 << 1) | bit0;

                        plane1 >>= 1;
                        plane2 >>= 1;

                        chr[y * PIXELS_PER_TILE + x] = color_idx;
                    }
                }

                self.nt_cache[tile_index] = Some(CachedTile { chr });
                chr
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_background_tile(
        &mut self,
        chr_bank_offset: u16,
        nt_offset: usize,
        nth: usize,
        tile_col: usize,
        tile_row: usize,
        offset_x: usize,
    ) {
        let tile_idx = nt_offset + nth;

        if !self.updated_bg_tiles[tile_idx] {
            return;
        }

        let tile =
            self.get_nametable_tile(nt_offset, self.vram[tile_idx] as usize, chr_bank_offset);

        for y in 0..PIXELS_PER_TILE {
            for x in 0..PIXELS_PER_TILE {
                let color_idx = tile[y * PIXELS_PER_TILE + x];
                let rgb =
                    self.background_color_at(nt_offset, tile_col, tile_row, color_idx as usize);

                let pixel_x = offset_x + tile_col * PIXELS_PER_TILE + x;
                let pixel_y = tile_row * PIXELS_PER_TILE + y;
                PPU::set_pixel(self.background.as_mut_slice(), pixel_x, pixel_y, rgb);
            }
        }

        self.updated_bg_tiles[tile_idx] = false;
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
                    if !self.regs.ctrl.contains(Ctrl::SPRITE_PATTERN_ADDR) {
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
                chr_bank_offset,
                top_tile_idx,
                sprite_x,
                sprite_y,
                behind_background,
                flip_horizontally,
                flip_vertically,
                palette,
                frame,
            );

            if let Some(idx) = bot_tile_idx {
                self.render_sprite_tile(
                    chr_bank_offset,
                    idx,
                    sprite_x,
                    sprite_y + 8,
                    behind_background,
                    flip_horizontally,
                    flip_vertically,
                    palette,
                    frame,
                );
            }
        }
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

    #[allow(clippy::too_many_arguments)]
    fn render_sprite_tile(
        &mut self,
        chr_bank_offset: u16,
        nth: usize,
        tile_x: usize,
        tile_y: usize,
        behind_background: bool,
        flip_horizontally: bool,
        flip_vertically: bool,
        palette: [Option<(u8, u8, u8)>; 4],
        frame: &mut [u8],
    ) {
        let mut tile = [0u8; 16];
        self.rom
            .mapper
            .get_tile(&mut self.rom.cart, chr_bank_offset, nth, &mut tile);

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
                    let is_bg_opaque = false;
                    if !behind_background || !is_bg_opaque {
                        let offset = (pixel_y * WIDTH + pixel_x) * 3;
                        if offset < frame.len() {
                            frame[offset] = rgb.0;
                            frame[offset + 1] = rgb.1;
                            frame[offset + 2] = rgb.2;
                            // PPU::set_pixel(frame, pixel_x, pixel_y, rgb);
                        }
                    }
                }
            }
        }
    }

    #[inline]
    fn is_sprite_zero_hit(&mut self) -> bool {
        let y = self.attributes[0] as usize;
        let x = self.attributes[3] as usize;

        y == self.scanline as usize
            && x <= self.cycle as usize
            && self.regs.mask.contains(Mask::SHOW_SPRITES)
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

    // #[inline]
    // fn transfer_frame_buffer(&mut self) {
    //     self.frame_buffer_complete
    //         .copy_from_slice(self.frame_buffer.as_slice());
    // }

    fn detect_nmi_edge(&mut self) {
        let nmi = self.regs.ctrl.contains(Ctrl::GENERATE_NMI)
            && self.regs.status.contains(Status::VBLANK_STARTED);

        if !self.nmi_edge_detector && nmi {
            self.should_trigger_nmi = true;
        }

        self.nmi_edge_detector = nmi;
    }

    fn fetch_nametable_byte(&mut self) {
        // See Tile and attribute fetching
        // https://www.nesdev.org/wiki/PPU_scrolling
        let offset = 0x2000 | (self.regs.v & 0x0FFF);
        self.nametable_byte = self.read_nametable(offset);
    }

    fn fetch_attribute_table_byte(&mut self) {
        let v = self.regs.v;
        let address = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0b111);
        let shift = ((v >> 4) & 4) | (v & 2);
        self.attribute_table_byte = (self.read_nametable(address) >> shift) & 0b11;
    }

    fn fetch_pattern_table_bytes(&mut self) {
        let table = self.regs.ctrl.background_chr_offset();
        let tile = self.nametable_byte as u16;
        let fine_y = self.regs.fine_y() as u16;
        let offset = table + tile * 16 + fine_y;

        self.pattern_table_low_byte = self.read_chr(offset);
        self.pattern_table_high_byte = self.read_chr(offset + 8);
    }

    fn reset(&mut self) {
        self.cycle = 340;
        self.scanline = 240;
        self.frame = 0;
        self.regs.write_ctrl(0);
        self.regs.write_mask(0);
    }

    fn store_background_tile_data(&mut self) {
        let mut data: u32 = 0;
        let attr = self.attribute_table_byte << 2;

        for _ in 0..8 {
            let p1 = (self.pattern_table_low_byte & (1 << 7)) >> 7;
            let p2 = (self.pattern_table_high_byte & (1 << 7)) >> 6;
            let pattern = p2 | p1;
            self.pattern_table_low_byte <<= 1;
            self.pattern_table_high_byte <<= 1;
            data <<= 4;
            data |= (attr | pattern) as u32;
        }

        self.tile_data |= data as u64;
    }

    fn get_background_pixel(&mut self) -> Option<(u8, u8, u8)> {
        if self.regs.show_background() {
            let color_idx = ((self.tile_data >> 32) >> ((7 - self.regs.x) * 4)) & 0xF;
            if color_idx & 3 == 0 {
                None
            } else {
                Some(color_idx as usize)
            }
        } else {
            None
        }
        .map(|idx| COLOR_PALETTE[(self.palette[idx] & 63) as usize])
    }

    fn get_sprite_pixel(&mut self) -> Option<((u8, u8, u8), bool, u8)> {
        if self.regs.show_sprites() {
            let x = self.cycle - 1;

            for i in 0..(self.visible_sprites_count as usize) {
                let sprite = self.scanline_sprites[i];
                if x >= sprite.x && x < sprite.x + 8 {
                    let idx = sprite.chr[(x - sprite.x) as usize];
                    if let Some(color) = self.sprite_color(sprite.palette_idx, idx) {
                        return Some((color, sprite.behind_background, sprite.idx));
                    }
                }
            }
        }

        None
    }

    fn fetch_next_scanline_sprites(&mut self) {
        let mut count = 0;
        let sprite_size = self.regs.ctrl.sprite_size();
        let height = sprite_size.height() as u16;

        for i in 0..64 {
            let offset = i * 4;
            let y = self.attributes[offset] as u16;

            if self.scanline >= y && self.scanline < y + height {
                let row = self.scanline - y;
                let tile_idx = self.attributes[offset + 1] as u16;
                let attr = self.attributes[offset + 2];
                let palette_idx = attr & 0b11;
                let behind_background = attr & 0b0010_0000 != 0;
                let flip_horizontally = attr & 0b0100_0000 != 0;
                let flip_vertically = attr & 0b1000_0000 != 0;
                let x = self.attributes[offset + 3];

                let (chr_bank, row, tile_idx) = match sprite_size {
                    SpriteSize::Sprite8x8 => {
                        let chr_bank = self.regs.ctrl.sprite_chr_offset();
                        let row = if flip_vertically { 7 - row } else { row };
                        (chr_bank, row, tile_idx)
                    }
                    SpriteSize::Sprite8x16 => {
                        let chr_bank = (tile_idx & 1) * 0x1000;
                        let mut tile_idx = tile_idx & 0xFE;
                        let mut row = if flip_vertically { 15 - row } else { row };

                        if row > 7 {
                            row -= 8;
                            tile_idx += 1;
                        }

                        (chr_bank, row, tile_idx)
                    }
                };

                let tile_offset = chr_bank + tile_idx * 16 + row;

                if count < 8 {
                    let chr_low = self.read_chr(tile_offset);
                    let chr_high = self.read_chr(tile_offset + 8);
                    let mut chr = [0u8; 8];

                    #[allow(clippy::needless_range_loop)]
                    for i in 0..8 {
                        let mask = 1 << if flip_horizontally { i } else { 7 - i };
                        let p1: u8 = (chr_low & mask != 0).into();
                        let p2: u8 = (chr_high & mask != 0).into();
                        let pattern = (p2 << 1) | p1;
                        chr[i] = pattern;
                    }

                    self.scanline_sprites[count] = SpriteData {
                        x: x as u16,
                        idx: i as u8,
                        palette_idx,
                        behind_background,
                        chr,
                    };

                    count += 1;
                } else {
                    // TODO: implement sprite overflow hardware bug
                    self.regs.status.insert(Status::SPRITE_OVERFLOW);
                    break;
                }
            }
        }

        self.visible_sprites_count = count as u8;
    }

    fn render_pixel(&mut self, frame: &mut [u8]) {
        let x = self.cycle - 1;
        let y = self.scanline;
        let mut bg = self.get_background_pixel();
        let mut sprite = self.get_sprite_pixel();

        if x < 8 {
            if !self.regs.show_leftmost_background() {
                bg = None;
            }

            if !self.regs.show_leftmost_sprites() {
                sprite = None;
            }
        }

        let color = match (bg, sprite) {
            (None, None) => COLOR_PALETTE[(self.palette[0] & 63) as usize],
            (None, Some((sp, _, _))) => sp,
            (Some(bg), None) => bg,
            (Some(bg), Some((sp, behind, _))) => {
                if behind {
                    bg
                } else {
                    sp
                }
            }
        };

        if let Some((_, _, idx)) = sprite {
            let sprite_zero_hit = idx == 0
                && x < 255
                && bg.is_some()
                && !self.regs.status.contains(Status::SPRITE_ZERO_HIT);

            if sprite_zero_hit {
                self.regs.status.insert(Status::SPRITE_ZERO_HIT);
            }
        }

        PPU::set_pixel(frame, x as usize, y as usize, color);
    }

    fn set_pixel(target: &mut [u8], x: usize, y: usize, (r, g, b): (u8, u8, u8)) {
        let offset = (y * 2 * WIDTH + x) * 3;

        if x < 2 * WIDTH && y < HEIGHT && offset < target.len() {
            target[offset] = r;
            target[offset + 1] = g;
            target[offset + 2] = b;
        }
    }

    #[inline]
    pub fn sprite_zero_coords(&self) -> (u8, u8) {
        (self.attributes[3], self.attributes[0])
    }

    // https://www.nesdev.org/wiki/PPU_palettes
    fn sprite_color(&self, palette_idx: u8, color_idx: u8) -> Option<(u8, u8, u8)> {
        let palette_offset = SPRITE_PALETTES_OFFSET + palette_idx as usize * BYTES_PER_PALLETE;

        match color_idx {
            0 => None,
            1 => Some(COLOR_PALETTE[(self.palette[palette_offset] & 63) as usize]),
            2 => Some(COLOR_PALETTE[(self.palette[palette_offset + 1] & 63) as usize]),
            3 => Some(COLOR_PALETTE[(self.palette[palette_offset + 2] & 63) as usize]),
            _ => unreachable!(),
        }
    }

    pub fn is_asserting_nmi(&mut self) -> bool {
        let triggered = self.nmi_triggered;
        self.nmi_triggered = false;
        triggered
    }

    fn nametable_mirrored_addr(&self, addr: u16) -> u16 {
        let addr = addr & 0x2FFF;
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
        // let bg_bank = self.regs.ctrl.bits() & 0b11;
        // if self.regs.show_background() && data & 0b11 != bg_bank {
        //     match bg_bank {
        //         0 => self.updated_bg_tiles[BYTES_PER_NAMETABLE..].fill(true),
        //         1 => self.updated_bg_tiles[..BYTES_PER_NAMETABLE].fill(true),
        //         _ => {}
        //     }
        // }

        if data & Ctrl::BACKROUND_PATTERN_ADDR.bits()
            != self.regs.ctrl.bits() & Ctrl::BACKROUND_PATTERN_ADDR.bits()
        {
            self.nt_cache.fill(None);
        }

        self.regs.write_ctrl(data);
        // the PPU immediately triggers a NMI when the VBlank flag transitions from 0 to 1 during VBlank
        self.detect_nmi_edge();
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        self.rom.mapper.read(&mut self.rom.cart, addr)
    }

    fn read_nametable(&self, addr: u16) -> u8 {
        let addr = self.nametable_mirrored_addr(addr);
        self.vram[addr as usize]
    }

    pub fn read_data_reg(&mut self) -> u8 {
        let addr = self.regs.v;

        let res = match addr {
            0x0000..=0x1fff => {
                let res = self.data_buffer;
                self.data_buffer = self.read_chr(addr);
                res
            }
            0x2000..=0x3eff => {
                let res = self.data_buffer;
                self.data_buffer = self.read_nametable(addr);
                res
            }
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => self.palette[(addr as usize - 0x3f10) & 31],
            0x3f00..=0x3fff => self.palette[(addr as usize - 0x3f00) & 31],
            _ => {
                // panic!("invalid ppu read address: {:04x}", addr);
                0
            }
        };

        self.regs.increment_vram_addr();
        res
    }

    pub fn write_data_reg(&mut self, data: u8) {
        let addr = self.regs.v;

        match addr {
            0x0000..=0x1fff => self.rom.mapper.write(&mut self.rom.cart, addr, data),
            0x2000..=0x2fff => {
                let mirrored_addr = self.nametable_mirrored_addr(addr) as usize;
                if self.vram[mirrored_addr] != data {
                    self.vram[mirrored_addr] = data;

                    if mirrored_addr & 0x3ff < TILES_PER_NAMETABLE {
                        // nametable
                        self.updated_bg_tiles[mirrored_addr] = true;
                    } else {
                        // attributes
                        let attrib_idx = mirrored_addr & 63;
                        let meta_tile_x = attrib_idx & 31;
                        let meta_tile_y = attrib_idx / 32;
                        let base_nt_addr =
                            (mirrored_addr / BYTES_PER_NAMETABLE) * BYTES_PER_NAMETABLE;

                        for x in 0..4 {
                            for y in 0..4 {
                                let tile_x = meta_tile_x * 4 + x;
                                let tile_y = meta_tile_y * 4 + y;
                                let tile_idx = base_nt_addr + tile_y * 32 + tile_x;
                                self.updated_bg_tiles[tile_idx] = true;
                            }
                        }
                    }
                }
            }
            // 0x3000..=0x3eff => unreachable!(),
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                let addr = ((addr - 0x3f10) & 31) as usize;

                if data != self.palette[addr] {
                    self.palette[addr] = data;
                    self.updated_bg_tiles.fill(true);
                }
            }
            0x3f00..=0x3fff => {
                let addr = ((addr - 0x3f00) & 31) as usize;

                if data != self.palette[addr] {
                    self.palette[addr] = data;
                    self.updated_bg_tiles.fill(true);
                }
            }
            _ => {
                // ignoring write to {addr:04X}
            }
        }

        self.regs.increment_vram_addr();
    }

    pub fn read_oam_data_reg(&mut self) -> u8 {
        self.attributes[self.regs.oam_addr as usize]
    }

    pub fn write_oam_data_reg(&mut self, data: u8) {
        self.attributes[self.regs.oam_addr as usize] = data;
        self.regs.oam_addr = self.regs.oam_addr.wrapping_add(1);
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

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0x2002 => {
                let res = self.regs.read_status(self.open_bus);
                self.detect_nmi_edge();
                res
            }
            0x2004 => self.read_oam_data_reg(),
            0x2007 => self.read_data_reg(),
            _ => 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        // https://www.nesdev.org/wiki/Open_bus_behavior#PPU_open_bus
        self.open_bus = data;

        match addr {
            0x2000 => self.write_ctrl_reg(data),
            0x2001 => self.regs.write_mask(data),
            0x2002 => {
                // panic!("PPU status register is read-only");
            }
            0x2003 => self.regs.write_oam_address(data),
            0x2004 => self.write_oam_data_reg(data),
            0x2005 => self.regs.write_scroll(data),
            0x2006 => self.regs.write_address(data),
            0x2007 => self.write_data_reg(data),
            _ => unreachable!("invalid PPU register address"),
        }
    }
}

impl savestate::Save for SpriteData {
    fn save(&self, s: &mut savestate::Section) {
        s.data.write_u16(self.x);
        s.data.write_u8(self.idx);
        s.data.write_u8(self.palette_idx);
        s.data.write_bool(self.behind_background);
    }

    fn load(&mut self, s: &mut savestate::Section) -> Result<(), SaveStateError> {
        self.x = s.data.read_u16()?;
        self.idx = s.data.read_u8()?;
        self.palette_idx = s.data.read_u8()?;
        self.behind_background = s.data.read_bool()?;

        Ok(())
    }
}

const PPU_SECTION_NAME: &str = "ppu";

impl savestate::Save for PPU {
    fn save(&self, parent: &mut savestate::Section) {
        let s = parent.create_child(PPU_SECTION_NAME);

        s.data.write_u8(self.open_bus);
        s.data.write_u8_slice(&self.vram);
        s.data.write_u8_slice(&self.palette);
        s.data.write_u8_slice(&self.attributes);
        s.data.write_u16(self.cycle);
        s.data.write_u16(self.scanline);
        s.data.write_u64(self.frame);
        s.data.write_u8(self.data_buffer);
        s.data.write_bool(self.nmi_triggered);
        s.data.write_bool(self.nmi_edge_detector);
        s.data.write_bool(self.should_trigger_nmi);
        s.data.write_bool(self.frame_complete);
        s.data.write_u64(self.tile_data);
        s.data.write_u8(self.nametable_byte);
        s.data.write_u8(self.attribute_table_byte);
        s.data.write_u8(self.pattern_table_low_byte);
        s.data.write_u8(self.pattern_table_high_byte);
        s.data.write_u8(self.visible_sprites_count);
        s.write_all(&self.scanline_sprites);

        self.regs.save(s);
        self.rom.mapper.save(s);
    }

    fn load(&mut self, parent: &mut savestate::Section) -> Result<(), SaveStateError> {
        let s = parent.get(PPU_SECTION_NAME)?;

        self.open_bus = s.data.read_u8()?;
        s.data.read_u8_slice(&mut self.vram)?;
        s.data.read_u8_slice(&mut self.palette)?;
        s.data.read_u8_slice(&mut self.attributes)?;
        self.cycle = s.data.read_u16()?;
        self.scanline = s.data.read_u16()?;
        self.frame = s.data.read_u64()?;
        self.data_buffer = s.data.read_u8()?;
        self.nmi_triggered = s.data.read_bool()?;
        self.nmi_edge_detector = s.data.read_bool()?;
        self.should_trigger_nmi = s.data.read_bool()?;
        self.frame_complete = s.data.read_bool()?;
        self.tile_data = s.data.read_u64()?;
        self.nametable_byte = s.data.read_u8()?;
        self.attribute_table_byte = s.data.read_u8()?;
        self.pattern_table_low_byte = s.data.read_u8()?;
        self.pattern_table_high_byte = s.data.read_u8()?;
        self.visible_sprites_count = s.data.read_u8()?;
        s.read_all(&mut self.scanline_sprites)?;

        self.regs.load(s)?;
        self.rom.mapper.load(s)?;

        Ok(())
    }
}
