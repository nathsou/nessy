mod registers;

use self::registers::{Ctrl, Registers, SpriteSize, Status};
use crate::{
    cpu::rom::{Mirroring, ROM},
    savestate::{self, SaveStateError},
};

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

#[derive(Clone, Copy)]
struct SpriteData {
    x: u16,
    idx: u8,
    chr: [u8; 8],
    pattern_addr: u16,
    flip: bool,
    low: u8,
    palette_idx: u8,
    behind_background: bool,
}

#[allow(clippy::upper_case_acronyms)]
pub struct PPU {
    pub rom: ROM,
    regs: Registers,
    open_bus: u8,
    vram: [u8; 4 * 1024],
    palette: [u8; 32],
    attributes: [u8; 64 * 4],
    secondary_oam: [u8; 32],
    secondary_indices: [u8; 8],
    eval_n: u8,
    eval_m: u8,
    eval_count: u8,
    eval_data: u8,
    pub cycle: u16,
    clock: u64,
    skip_rendering: bool,
    suppress_vblank: bool,
    scanline: u16,
    frame: u64,
    data_buffer: u8,
    nmi_triggered: bool,
    nmi_edge_detector: bool,
    should_trigger_nmi: u8,
    pub frame_complete: bool,
    tile_data: u64,
    nametable_byte: u8,
    attribute_table_byte: u8,
    pattern_table_low_byte: u8,
    pattern_table_high_byte: u8,
    scanline_sprites: [SpriteData; 8],
    visible_sprites_count: u8,
    frame_buffer: [u8; WIDTH * HEIGHT * 3],
    frame_buffer_complete: Box<[u8; WIDTH * HEIGHT * 3]>, // avoid stack overflow in WASM
}

impl PPU {
    pub fn new(rom: ROM) -> Self {
        let mut ppu = PPU {
            rom,
            regs: Registers::new(),
            open_bus: 0,
            vram: [0; 4 * 1024],
            palette: [0; 32],
            attributes: [0; 64 * 4],
            secondary_oam: [0xff; 32],
            secondary_indices: [0; 8],
            eval_n: 0,
            eval_m: 0,
            eval_count: 0,
            eval_data: 0xff,
            cycle: 0,
            clock: 0,
            skip_rendering: false,
            suppress_vblank: false,
            scanline: 0,
            frame: 0,
            data_buffer: 0,
            nmi_triggered: false,
            should_trigger_nmi: 0,
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
                pattern_addr: 0,
                flip: false,
                low: 0,
            }; 8],
            visible_sprites_count: 0,
            frame_buffer: [0; WIDTH * HEIGHT * 3],
            frame_buffer_complete: Box::new([0; WIDTH * HEIGHT * 3]),
        };

        ppu.reset();
        ppu
    }

    fn tick(&mut self) {
        self.clock = self.clock.wrapping_add(1);

        if self.should_trigger_nmi > 0
            && self.regs.ctrl.contains(Ctrl::GENERATE_NMI)
            && self.regs.status.contains(Status::VBLANK_STARTED)
        {
            self.should_trigger_nmi -= 1;
            if self.should_trigger_nmi == 0 {
                self.nmi_triggered = true;
            }
        }

        if self.skip_rendering && self.regs.f && self.scanline == 261 && self.cycle == 339 {
            // skip cycle 339 of pre-render scanline
            self.cycle = 0;
            self.scanline = 0;
            self.regs.f = !self.regs.f;
            self.frame = self.frame.wrapping_add(1);
            return;
        }

        if self.cycle == 337 {
            self.skip_rendering = self.regs.rendering_enabled();
        }
        self.cycle += 1;

        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;

            if self.scanline > 261 {
                self.scanline = 0;
                self.regs.f = !self.regs.f;
                self.frame = self.frame.wrapping_add(1);
            }
        }
    }

    pub fn step(&mut self) {
        self.tick();

        let preline = self.scanline == 261;
        let visible_line = self.scanline < 240;
        let render_line = preline || visible_line;
        let pre_fetch_cycle = self.cycle >= 321 && self.cycle <= 336;
        let visible_cycle = self.cycle >= 1 && self.cycle <= 256;
        let fetch_cycle = pre_fetch_cycle || visible_cycle;

        if visible_line && visible_cycle {
            if self.regs.rendering_enabled() {
                self.evaluate_sprite();
            }
            self.render_pixel();
        }
        // Both fetch pipelines keep running when either rendering bit is enabled.
        if self.regs.rendering_enabled() {
            if render_line && fetch_cycle {
                self.tile_data <<= 4;

                match self.cycle & 7 {
                    1 => self.fetch_nametable_byte(),
                    3 => self.fetch_attribute_table_byte(),
                    5 => self.fetch_pattern_table_byte(false),
                    7 => self.fetch_pattern_table_byte(true),
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

        if self.regs.rendering_enabled() && render_line && (257..=320).contains(&self.cycle) {
            if self.cycle == 257 {
                if visible_line {
                    self.fetch_next_scanline_sprites();
                } else {
                    self.visible_sprites_count = 0;
                }
            }
            self.fetch_sprite_byte();
        }
        if self.regs.rendering_enabled() && render_line && (self.cycle == 337 || self.cycle == 339)
        {
            self.fetch_nametable_byte();
        }
        // The address is driven on the first dot of each two-dot fetch.
        if self.regs.rendering_enabled() && render_line {
            if (256..320).contains(&self.cycle) {
                let phase = (self.cycle - 256) & 7;
                let slot = ((self.cycle - 256) / 8) as usize;
                let addr = if phase == 4 || phase == 6 {
                    let base = if slot < self.visible_sprites_count as usize {
                        self.scanline_sprites[slot].pattern_addr
                    } else {
                        match self.regs.ctrl.sprite_size() {
                            SpriteSize::Sprite8x8 => self.regs.ctrl.sprite_chr_offset() + 0xff0,
                            SpriteSize::Sprite8x16 => 0x1ff0,
                        }
                    };
                    base + if phase == 6 { 8 } else { 0 }
                } else {
                    0x2000 | (self.regs.v & 0xfff)
                };
                if phase & 1 == 0 {
                    self.observe_address(addr);
                }
            } else if self.cycle <= 336 {
                let phase = self.cycle & 7;
                let addr = if phase == 4 || phase == 6 {
                    self.regs.ctrl.background_chr_offset()
                        + self.nametable_byte as u16 * 16
                        + self.regs.fine_y() as u16
                        + if phase == 6 { 8 } else { 0 }
                } else {
                    0x2000 | (self.regs.v & 0xfff)
                };
                if phase & 1 == 0 {
                    self.observe_address(addr);
                }
            } else if self.cycle == 338 {
                self.observe_address(0x2000 | (self.regs.v & 0xfff));
            }
        }

        // VBlank
        if self.scanline == 241 && self.cycle == 1 {
            self.frame_complete = true;
            if !self.suppress_vblank {
                self.regs.status.insert(Status::VBLANK_STARTED);
            }
            self.suppress_vblank = false;
            self.detect_nmi_edge();
            self.transfer_frame_buffer();
        }

        if preline && self.cycle == 1 {
            self.regs.status.remove(Status::VBLANK_STARTED);
            self.regs.status.remove(Status::SPRITE_ZERO_HIT);
            self.regs.status.remove(Status::SPRITE_OVERFLOW);
            self.detect_nmi_edge();
        }
    }

    fn transfer_frame_buffer(&mut self) {
        self.frame_buffer_complete
            .copy_from_slice(&self.frame_buffer);
    }

    fn detect_nmi_edge(&mut self) {
        let nmi = self.regs.ctrl.contains(Ctrl::GENERATE_NMI)
            && self.regs.status.contains(Status::VBLANK_STARTED);

        if !self.nmi_edge_detector && nmi {
            self.should_trigger_nmi = 2;
        }

        if !nmi {
            self.should_trigger_nmi = 0;
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

    fn fetch_pattern_table_byte(&mut self, high: bool) {
        let addr = self.regs.ctrl.background_chr_offset()
            + self.nametable_byte as u16 * 16
            + self.regs.fine_y() as u16
            + if high { 8 } else { 0 };
        let value = self.read_chr(addr);
        if high {
            self.pattern_table_high_byte = value;
        } else {
            self.pattern_table_low_byte = value;
        }
    }
    fn fetch_sprite_byte(&mut self) {
        let slot = ((self.cycle - 257) / 8) as usize;
        let phase = (self.cycle - 257) & 7;
        if phase == 0 || phase == 2 {
            self.read_nametable(0x2000 | (self.regs.v & 0xfff));
        }
        if phase != 4 && phase != 6 {
            return;
        }
        let addr = if slot < self.visible_sprites_count as usize {
            self.scanline_sprites[slot].pattern_addr
        } else {
            match self.regs.ctrl.sprite_size() {
                SpriteSize::Sprite8x8 => self.regs.ctrl.sprite_chr_offset() + 0xff0,
                SpriteSize::Sprite8x16 => 0x1ff0,
            }
        };
        let value = self.read_chr(addr + if phase == 6 { 8 } else { 0 });
        if slot < self.visible_sprites_count as usize {
            let sprite = &mut self.scanline_sprites[slot];
            if phase == 4 {
                sprite.low = value;
            } else {
                for i in 0..8 {
                    let bit = if sprite.flip { i } else { 7 - i };
                    sprite.chr[i] = ((sprite.low >> bit) & 1) | (((value >> bit) & 1) << 1);
                }
            }
        }
    }
    fn observe_address(&mut self, addr: u16) {
        self.rom.mapper.ppu_address(addr & 0x3fff, self.clock);
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
        .map(|idx| self.color(self.palette[idx]))
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

    fn evaluate_sprite(&mut self) {
        if self.cycle <= 64 {
            if self.cycle & 1 == 0 {
                self.secondary_oam[(self.cycle / 2 - 1) as usize] = 0xff;
            }
            self.eval_n = 0;
            self.eval_m = 0;
            self.eval_count = 0;
            return;
        }
        if self.eval_n >= 64 {
            return;
        }
        if self.cycle & 1 != 0 {
            self.eval_data = self.attributes[(self.eval_n as usize * 4) + self.eval_m as usize];
            return;
        }
        let y = self.eval_data as u16;
        let in_range =
            self.scanline >= y && self.scanline < y + self.regs.ctrl.sprite_size().height() as u16;
        if self.eval_count < 8 {
            if self.eval_m == 0 && !in_range {
                self.eval_n += 1;
                return;
            }
            let offset = self.eval_count as usize * 4 + self.eval_m as usize;
            self.secondary_oam[offset] = self.eval_data;
            if self.eval_m == 0 {
                self.secondary_indices[self.eval_count as usize] = self.eval_n;
            }
            self.eval_m += 1;
            if self.eval_m == 4 {
                self.eval_m = 0;
                self.eval_n += 1;
                self.eval_count += 1;
            }
        } else {
            if in_range {
                self.regs.status.insert(Status::SPRITE_OVERFLOW);
            }
            // Once secondary OAM is full the broken incrementer scans diagonally:
            // tile/attribute/X bytes of subsequent sprites can be interpreted as Y.
            self.eval_n += 1;
            self.eval_m = (self.eval_m + 1) & 3;
        }
    }

    fn fetch_next_scanline_sprites(&mut self) {
        let mut count = 0;
        let sprite_size = self.regs.ctrl.sprite_size();
        let height = sprite_size.height() as u16;

        for i in 0..self.eval_count as usize {
            let offset = i * 4;
            let y = self.secondary_oam[offset] as u16;

            if self.scanline >= y && self.scanline < y + height {
                let row = self.scanline - y;
                let tile_idx = self.secondary_oam[offset + 1] as u16;
                let attr = self.secondary_oam[offset + 2];
                let palette_idx = attr & 0b11;
                let behind_background = attr & 0b0010_0000 != 0;
                let flip_horizontally = attr & 0b0100_0000 != 0;
                let flip_vertically = attr & 0b1000_0000 != 0;
                let x = self.secondary_oam[offset + 3];

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
                    self.scanline_sprites[count] = SpriteData {
                        x: x as u16,
                        idx: self.secondary_indices[i],
                        palette_idx,
                        behind_background,
                        chr: [0; 8],
                        pattern_addr: tile_offset,
                        flip: flip_horizontally,
                        low: 0,
                    };

                    count += 1;
                }
            }
        }

        self.visible_sprites_count = count as u8;
    }

    fn render_pixel(&mut self) {
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
            (None, None) => self.color(
                self.palette[if !self.regs.rendering_enabled() && self.regs.v & 0x3f00 == 0x3f00 {
                    Self::palette_index(self.regs.v)
                } else {
                    0
                }],
            ),
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

        self.set_pixel(x as usize, y as usize, color);
    }

    fn color(&self, index: u8) -> (u8, u8, u8) {
        let mask = self.regs.mask.bits();
        let (mut r, mut g, mut b) =
            COLOR_PALETTE[(index & if mask & 1 != 0 { 0x30 } else { 0x3f }) as usize];
        // Each NTSC emphasis bit attenuates the other two color components.
        if mask & 0x20 != 0 {
            g = (g as u16 * 3 / 4) as u8;
            b = (b as u16 * 3 / 4) as u8;
        }
        if mask & 0x40 != 0 {
            r = (r as u16 * 3 / 4) as u8;
            b = (b as u16 * 3 / 4) as u8;
        }
        if mask & 0x80 != 0 {
            r = (r as u16 * 3 / 4) as u8;
            g = (g as u16 * 3 / 4) as u8;
        }
        (r, g, b)
    }

    fn set_pixel(&mut self, x: usize, y: usize, (r, g, b): (u8, u8, u8)) {
        if x < WIDTH && y < HEIGHT {
            let offset = (y * WIDTH + x) * 3;
            self.frame_buffer[offset] = r;
            self.frame_buffer[offset + 1] = g;
            self.frame_buffer[offset + 2] = b;
        }
    }

    // https://www.nesdev.org/wiki/PPU_palettes
    fn sprite_color(&self, palette_idx: u8, color_idx: u8) -> Option<(u8, u8, u8)> {
        let palette_offset = SPRITE_PALETTES_OFFSET + palette_idx as usize * BYTES_PER_PALLETE;

        match color_idx {
            0 => None,
            1 => Some(self.color(self.palette[palette_offset])),
            2 => Some(self.color(self.palette[palette_offset + 1])),
            3 => Some(self.color(self.palette[palette_offset + 2])),
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
        self.regs.write_ctrl(data);
        // the PPU immediately triggers a NMI when the VBlank flag transitions from 0 to 1 during VBlank
        self.detect_nmi_edge();
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        self.observe_address(addr);
        self.rom.mapper.read(&mut self.rom.cart, addr)
    }

    fn read_nametable(&mut self, addr: u16) -> u8 {
        self.observe_address(addr);
        let addr = self.nametable_mirrored_addr(addr);
        self.vram[addr as usize]
    }

    fn palette_index(addr: u16) -> usize {
        let idx = addr as usize & 31;
        if idx & 0x13 == 0x10 {
            idx & 0x0f
        } else {
            idx
        }
    }
    fn increment_data_address(&mut self) {
        if self.regs.rendering_enabled() && (self.scanline < 240 || self.scanline == 261) {
            self.regs.increment_x();
            self.regs.increment_y();
        } else {
            self.regs.increment_vram_addr();
        }
        self.observe_address(self.regs.v);
    }
    pub fn read_data_reg(&mut self) -> u8 {
        let addr = self.regs.v & 0x3fff;
        let old = self.data_buffer;
        let result = if addr >= 0x3f00 {
            self.data_buffer = self.read_nametable(addr - 0x1000);
            let mask = if self.regs.mask.bits() & 1 != 0 {
                0x30
            } else {
                0x3f
            };
            (self.palette[Self::palette_index(addr)] & mask) | (self.open_bus & 0xc0)
        } else {
            self.data_buffer = if addr < 0x2000 {
                self.read_chr(addr)
            } else {
                self.read_nametable(addr)
            };
            old
        };
        self.increment_data_address();
        result
    }
    pub fn write_data_reg(&mut self, data: u8) {
        let addr = self.regs.v & 0x3fff;
        self.observe_address(addr);
        match addr {
            0..=0x1fff => self.rom.mapper.write(&mut self.rom.cart, addr, data),
            0x2000..=0x3eff => {
                let idx = self.nametable_mirrored_addr(addr) as usize;
                self.vram[idx] = data;
            }
            _ => self.palette[Self::palette_index(addr)] = data & 0x3f,
        }
        self.increment_data_address();
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
        let value = match addr {
            0x2002 => {
                if self.scanline == 241 && self.cycle == 0 {
                    self.suppress_vblank = true;
                }
                if self.scanline == 241 && self.cycle <= 2 {
                    self.nmi_triggered = false;
                    self.should_trigger_nmi = 0;
                }
                let res = self.regs.read_status(self.open_bus);
                self.detect_nmi_edge();
                res
            }
            0x2004 => self.read_oam_data_reg(),
            0x2007 => self.read_data_reg(),
            _ => self.open_bus,
        };
        self.open_bus = value;
        value
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        // https://www.nesdev.org/wiki/Open_bus_behavior#PPU_open_bus
        self.open_bus = data;

        match addr {
            0x2000 => self.write_ctrl_reg(data),
            0x2001 => self.regs.write_mask(data),
            0x2002 => {}
            0x2003 => self.regs.write_oam_address(data),
            0x2004 => self.write_oam_data_reg(data),
            0x2005 => self.regs.write_scroll(data),
            0x2006 => {
                self.regs.write_address(data);
                if !self.regs.w {
                    self.observe_address(self.regs.v);
                }
            }
            0x2007 => self.write_data_reg(data),
            _ => unreachable!("invalid PPU register address"),
        }
    }

    pub fn get_frame(&self) -> &[u8] {
        self.frame_buffer_complete.as_slice()
    }
}

impl savestate::Save for SpriteData {
    fn save(&self, s: &mut savestate::Section) {
        s.data.write_u16(self.x);
        s.data.write_u8(self.idx);
        s.data.write_u8(self.palette_idx);
        s.data.write_bool(self.behind_background);
        s.data.write_u8_slice(&self.chr);
        s.data.write_u16(self.pattern_addr);
        s.data.write_bool(self.flip);
        s.data.write_u8(self.low);
    }

    fn load(&mut self, s: &mut savestate::Section) -> Result<(), SaveStateError> {
        self.x = s.data.read_u16()?;
        self.idx = s.data.read_u8()?;
        self.palette_idx = s.data.read_u8()?;
        self.behind_background = s.data.read_bool()?;
        s.data.read_u8_slice(&mut self.chr)?;
        self.pattern_addr = s.data.read_u16()?;
        self.flip = s.data.read_bool()?;
        self.low = s.data.read_u8()?;
        if self.x > 255
            || self.pattern_addr > 0x1ff7
            || self.palette_idx > 3
            || self.chr.iter().any(|v| *v > 3)
        {
            return Err(SaveStateError::InvalidData);
        }

        Ok(())
    }
}

const PPU_SECTION_NAME: &str = "ppu";

impl savestate::Save for PPU {
    fn save(&self, parent: &mut savestate::Section) {
        let s = parent.create_child(PPU_SECTION_NAME);

        s.data.write_u8(self.open_bus);
        s.data.write_u64(self.clock);
        s.data.write_bool(self.skip_rendering);
        s.data.write_bool(self.suppress_vblank);
        s.data.write_u8(self.rom.cart.mirroring as u8);
        s.data.write_u8_slice(&self.frame_buffer);
        s.data.write_u8_slice(self.frame_buffer_complete.as_slice());
        s.data.write_u8_slice(&self.vram);
        s.data.write_u8_slice(&self.palette);
        s.data.write_u8_slice(&self.attributes);
        s.data.write_u8_slice(&self.secondary_oam);
        s.data.write_u8_slice(&self.secondary_indices);
        for v in [self.eval_n, self.eval_m, self.eval_count, self.eval_data] {
            s.data.write_u8(v);
        }
        s.data.write_u16(self.cycle);
        s.data.write_u16(self.scanline);
        s.data.write_u64(self.frame);
        s.data.write_u8(self.data_buffer);
        s.data.write_bool(self.nmi_triggered);
        s.data.write_bool(self.nmi_edge_detector);
        s.data.write_u8(self.should_trigger_nmi);
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
        self.clock = s.data.read_u64()?;
        self.skip_rendering = s.data.read_bool()?;
        self.suppress_vblank = s.data.read_bool()?;
        self.rom.cart.mirroring = match s.data.read_u8()? {
            0 => Mirroring::Horizontal,
            1 => Mirroring::Vertical,
            2 => Mirroring::OneScreenLowerBank,
            3 => Mirroring::OneScreenUpperBank,
            4 => Mirroring::FourScreen,
            _ => return Err(SaveStateError::InvalidData),
        };
        s.data.read_u8_slice(&mut self.frame_buffer)?;
        s.data
            .read_u8_slice(self.frame_buffer_complete.as_mut_slice())?;
        s.data.read_u8_slice(&mut self.vram)?;
        s.data.read_u8_slice(&mut self.palette)?;
        s.data.read_u8_slice(&mut self.attributes)?;
        s.data.read_u8_slice(&mut self.secondary_oam)?;
        s.data.read_u8_slice(&mut self.secondary_indices)?;
        self.eval_n = s.data.read_u8()?;
        self.eval_m = s.data.read_u8()?;
        self.eval_count = s.data.read_u8()?;
        self.eval_data = s.data.read_u8()?;
        if self.eval_n > 64 || self.eval_m > 3 || self.eval_count > 8 {
            return Err(SaveStateError::InvalidData);
        }
        self.cycle = s.data.read_u16()?;
        self.scanline = s.data.read_u16()?;
        self.frame = s.data.read_u64()?;
        self.data_buffer = s.data.read_u8()?;
        self.nmi_triggered = s.data.read_bool()?;
        self.nmi_edge_detector = s.data.read_bool()?;
        self.should_trigger_nmi = s.data.read_u8()?;
        self.frame_complete = s.data.read_bool()?;
        self.tile_data = s.data.read_u64()?;
        self.nametable_byte = s.data.read_u8()?;
        self.attribute_table_byte = s.data.read_u8()?;
        self.pattern_table_low_byte = s.data.read_u8()?;
        self.pattern_table_high_byte = s.data.read_u8()?;
        self.visible_sprites_count = s.data.read_u8()?;
        if self.should_trigger_nmi > 2
            || self.attribute_table_byte > 3
            || self.visible_sprites_count > 8
            || self.scanline > 261
            || self.cycle > 340
        {
            return Err(SaveStateError::InvalidData);
        }
        s.read_all(&mut self.scanline_sprites)?;

        self.regs.load(s)?;
        self.rom.mapper.load(s)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sprite_ppu() -> PPU {
        let mut bytes = vec![0; 16 + 0x4000 + 0x2000];
        bytes[..4].copy_from_slice(b"NES\x1a");
        bytes[4] = 1;
        bytes[5] = 1;
        let mut ppu = PPU::new(ROM::new(bytes).unwrap());
        ppu.scanline = 0;
        ppu.cycle = 0;
        ppu.attributes.fill(0xff);
        for i in 0..8 {
            ppu.attributes[i * 4] = 0;
        }
        ppu.write_register(0x2001, 0x18);
        ppu
    }

    #[test]
    fn ninth_sprite_sets_overflow_on_its_evaluation_cycle() {
        let mut ppu = sprite_ppu();
        ppu.attributes[8 * 4] = 0;
        for _ in 0..129 {
            ppu.step();
        }
        assert!(!ppu.regs.status.contains(Status::SPRITE_OVERFLOW));
        ppu.step();
        assert!(ppu.regs.status.contains(Status::SPRITE_OVERFLOW));
    }

    #[test]
    fn overflow_bug_interprets_tenth_sprites_tile_as_y() {
        let mut ppu = sprite_ppu();
        ppu.attributes[9 * 4 + 1] = 0;
        for _ in 0..131 {
            ppu.step();
        }
        assert!(!ppu.regs.status.contains(Status::SPRITE_OVERFLOW));
        ppu.step();
        assert!(ppu.regs.status.contains(Status::SPRITE_OVERFLOW));
        assert_eq!(ppu.eval_count, 8);
    }
}
