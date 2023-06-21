mod registers;

use self::registers::{Ctrl, Mask, Registers, SpriteSize, Status};
use crate::cpu::rom::{Mirroring, ROM};

const PIXELS_PER_TILE: usize = 8;
const BYTES_PER_PALLETE: usize = 4;
const SPRITE_PALETTES_OFFSET: usize = 0x11;
const WIDTH: usize = 256;
const HEIGHT: usize = 240;

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
    pub frame_complete: bool,
    tile_data: u64,
    nametable_byte: u8,
    attribute_table_byte: u8,
    pattern_table_low_byte: u8,
    pattern_table_high_byte: u8,
}

impl PPU {
    pub fn new(rom: ROM) -> Self {
        let mut ppu = PPU {
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
            frame_complete: false,
            tile_data: 0,
            nametable_byte: 0,
            attribute_table_byte: 0,
            pattern_table_low_byte: 0,
            pattern_table_high_byte: 0,
        };

        ppu.reset();
        ppu
    }

    fn tick(&mut self) {
        // TODO: handle NMI delay

        if self.regs.rendering_enabled() && self.regs.f && self.scanline == 261 && self.cycle == 339
        {
            // skip cycle 339 of pre-render scanline
            self.cycle = 0;
            self.scanline = 0;
            self.regs.f = !self.regs.f;
            return;
        }

        self.cycle += 1;

        if self.cycle > 340 {
            if self.is_sprite_zero_hit() {
                self.regs.status.set(Status::SPRITE_ZERO_HIT, true);
            }

            self.cycle = 0;
            self.scanline += 1;

            if self.scanline > 261 {
                self.scanline = 0;
                self.regs.f = !self.regs.f;
            }
        }
    }

    pub fn step(&mut self, frame: &mut [u8]) {
        self.tick();

        let rendering_enabled = self.regs.rendering_enabled();
        let preline = self.scanline == 261;
        let visible_line = self.scanline < 240;
        let render_line = preline || visible_line;
        let pre_fetch_cycle = self.cycle >= 321 && self.cycle <= 336;
        let visible_cycle = self.cycle >= 1 && self.cycle <= 256;
        let fetch_cycle = pre_fetch_cycle || visible_cycle;

        // background logic
        if rendering_enabled {
            if visible_line && visible_cycle {
                self.render_pixel(frame);
            }

            if render_line && fetch_cycle {
                self.tile_data <<= 4;

                match self.cycle & 7 {
                    1 => self.fetch_nametable_byte(),
                    3 => self.fetch_attribute_table_byte(),
                    5 => self.fetch_pattern_table_low_byte(),
                    7 => self.fetch_pattern_table_high_byte(),
                    0 => self.store_tile_data(),
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

        // VBlank
        if self.scanline == 241 && self.cycle == 1 {
            self.frame_complete = true;
            self.regs.status.set(Status::VBLANK_STARTED, true);
            self.regs.status.set(Status::SPRITE_ZERO_HIT, false);

            if self.regs.ctrl.contains(Ctrl::GENERATE_NMI) {
                self.nmi_triggered = true;
            }
        }

        if preline && self.cycle == 1 {
            self.sprite_zero_hit_this_frame = false;
            self.regs.status.set(Status::VBLANK_STARTED, false);
            self.regs.status.set(Status::SPRITE_ZERO_HIT, false);
        }
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

    fn fetch_pattern_table_low_byte(&mut self) {
        let table = self.regs.ctrl.background_chr_offset();
        let tile = self.nametable_byte as u16;
        let fine_y = self.regs.fine_y() as u16;
        let offset = table + tile * 16 + fine_y;

        self.pattern_table_low_byte = self.read_chr(offset);
    }

    fn fetch_pattern_table_high_byte(&mut self) {
        let table = self.regs.ctrl.background_chr_offset();
        let tile = self.nametable_byte as u16;
        let fine_y = self.regs.fine_y() as u16;
        let offset = table + tile * 16 + fine_y;

        self.pattern_table_high_byte = self.read_chr(offset + 8);
    }

    pub fn reset(&mut self) {
        self.cycle = 340;
        self.scanline = 240;
        self.regs.write_ctrl(0);
        self.regs.write_mask(0);
        self.regs.oam_addr = 0;
    }

    fn store_tile_data(&mut self) {
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

    fn get_background_pixel(&mut self) -> usize {
        if self.regs.show_background() {
            let color_idx = ((self.tile_data >> 32) >> ((7 - self.regs.x) * 4)) & 0xF;
            if color_idx & 3 == 0 {
                0
            } else {
                color_idx as usize
            }
        } else {
            0
        }
    }

    fn render_pixel(&mut self, frame: &mut [u8]) {
        let x = self.cycle - 1;
        let y = self.scanline;
        let color_idx = self.get_background_pixel();
        let color = COLOR_PALETTE[self.palette[color_idx] as usize];

        Self::set_pixel(frame, x, y, color);
    }

    #[inline]
    pub fn vblank_started(&self) -> bool {
        self.regs.status.contains(Status::VBLANK_STARTED)
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
    fn render_sprite_tile(
        &mut self,
        frame: &mut [u8],
        chr_bank_offset: u16,
        nth: usize,
        tile_x: usize,
        tile_y: usize,
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
                    PPU::set_pixel(frame, pixel_x, pixel_y, rgb);
                }
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

    pub fn render_sprites(&mut self, frame: &mut [u8]) {
        if !self.regs.show_sprites() {
            return;
        }

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
            // let behind_background = sprite_attr & 0b0010_0000 != 0;
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
                frame,
                chr_bank_offset,
                top_tile_idx,
                sprite_x,
                sprite_y,
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
                    flip_horizontally,
                    flip_vertically,
                    palette,
                );
            }
        }
    }

    #[inline]
    fn is_sprite_zero_hit(&mut self) -> bool {
        let y = self.attributes[0] as usize;
        let x = self.attributes[3] as usize;

        y == self.scanline && x <= self.cycle && self.regs.mask.contains(Mask::SHOW_SPRITES)
    }

    pub fn pull_nmi_status(&mut self) -> bool {
        let triggered = self.nmi_triggered;
        self.nmi_triggered = false;
        triggered
    }

    fn nametable_mirrored_addr(&self, addr: u16) -> u16 {
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
        let prev_nmi_status = self.regs.ctrl.contains(Ctrl::GENERATE_NMI);
        self.regs.write_ctrl(data);
        let new_nmi_status = self.regs.ctrl.contains(Ctrl::GENERATE_NMI);

        if self.vblank_started() && !prev_nmi_status && new_nmi_status {
            self.nmi_triggered = true;
        }
    }

    #[inline]
    fn read_chr(&self, addr: u16) -> u8 {
        self.rom.mapper.read_chr(&self.rom.cart, addr)
    }

    #[inline]
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
            0x2000..=0x2fff => {
                let res = self.data_buffer;
                self.data_buffer = self.read_nametable(addr);
                res
            }
            0x3000..=0x3eff => unreachable!(),
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => self.palette[(addr as usize - 0x3f10) & 31],
            0x3f00..=0x3fff => self.palette[(addr as usize - 0x3f00) & 31],
            _ => unreachable!(),
        };

        self.regs.increment_vram_addr();
        res
    }

    pub fn write_data_reg(&mut self, data: u8) {
        let addr = self.regs.v;

        match addr {
            0x0000..=0x1fff => self.rom.mapper.write_chr(&mut self.rom.cart, addr, data),
            0x2000..=0x2fff => {
                self.vram[self.nametable_mirrored_addr(addr) as usize] = data;
            }
            0x3000..=0x3eff => unreachable!(),
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                self.palette[((addr - 0x3f10) & 31) as usize] = data;
            }
            0x3f00..=0x3fff => {
                self.palette[((addr - 0x3f00) & 31) as usize] = data;
            }
            _ => {
                println!("ignoring write to {addr:04X}");
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
            0x2002 => self.regs.read_status(),
            0x2004 => self.read_oam_data_reg(),
            0x2007 => self.read_data_reg(),
            _ => 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        match addr {
            0x2000 => self.write_ctrl_reg(data),
            0x2001 => self.regs.write_mask(data),
            0x2002 => panic!("PPU status register is read-only"),
            0x2003 => self.regs.write_oam_address(data),
            0x2004 => self.write_oam_data_reg(data),
            0x2005 => self.regs.write_scroll(data),
            0x2006 => self.regs.write_address(data),
            0x2007 => self.write_data_reg(data),
            _ => unreachable!("invalid PPU register address"),
        }
    }
}
