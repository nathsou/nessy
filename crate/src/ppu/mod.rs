mod registers;

use self::registers::{Ctrl, Registers, SpriteSize, Status};
use crate::cpu::rom::{Mirroring, ROM};

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
    x: usize,
    idx: u8,
    chr: [u8; 8],
    palette_idx: u8,
    behind_background: bool,
}

#[allow(clippy::upper_case_acronyms)]
pub struct PPU {
    pub rom: ROM,
    regs: Registers,
    open_bus: u8,
    vram: [u8; 2 * 1024],
    palette: [u8; 32],
    attributes: [u8; 64 * 4],
    pub cycle: usize,
    scanline: usize,
    frame: usize,
    data_buffer: u8,
    nmi_triggered: bool,
    pub frame_complete: bool,
    tile_data: u64,
    nametable_byte: u8,
    attribute_table_byte: u8,
    pattern_table_low_byte: u8,
    pattern_table_high_byte: u8,
    scanline_sprites: [SpriteData; 8],
    visible_sprites_count: usize,
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
            self.frame += 1;
            return;
        }

        self.cycle += 1;

        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;

            if self.scanline > 261 {
                self.scanline = 0;
                self.regs.f = !self.regs.f;
                self.frame += 1;
            }
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

        // VBlank
        if self.scanline == 241 && self.cycle == 1 {
            self.frame_complete = true;
            self.regs.status.insert(Status::VBLANK_STARTED);

            if self.regs.ctrl.contains(Ctrl::GENERATE_NMI) {
                self.nmi_triggered = true;
            }
        }

        if preline && self.cycle == 1 {
            self.regs.status.remove(Status::VBLANK_STARTED);
            self.regs.status.remove(Status::SPRITE_ZERO_HIT);
            self.regs.status.remove(Status::SPRITE_OVERFLOW);
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

    fn fetch_pattern_table_bytes(&mut self) {
        let table = self.regs.ctrl.background_chr_offset();
        let tile = self.nametable_byte as u16;
        let fine_y = self.regs.fine_y() as u16;
        let offset = table + tile * 16 + fine_y;

        self.pattern_table_low_byte = self.read_chr(offset);
        self.pattern_table_high_byte = self.read_chr(offset + 8);
    }

    pub fn reset(&mut self) {
        self.cycle = 340;
        self.scanline = 240;
        self.regs.write_ctrl(0);
        self.regs.write_mask(0);
        self.regs.oam_addr = 0;
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
        .map(|idx| COLOR_PALETTE[self.palette[idx] as usize])
    }

    fn get_sprite_pixel(&mut self) -> Option<((u8, u8, u8), bool, u8)> {
        if self.regs.show_sprites() {
            let x = self.cycle - 1;

            for i in 0..self.visible_sprites_count {
                let sprite = self.scanline_sprites[i];
                if x >= sprite.x && x < sprite.x + 8 {
                    let idx = sprite.chr[x - sprite.x];
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
        let height = sprite_size.height() as usize;

        for i in 0..64 {
            let offset = i * 4;
            let y = self.attributes[offset] as usize;

            if self.scanline >= y && self.scanline < y + height {
                let row = self.scanline - y;
                let tile_idx = self.attributes[offset + 1] as u16;
                let attr = self.attributes[offset + 2];
                let palette_idx = attr & 0b11;
                let behind_background = attr & 0b0010_0000 != 0;
                let flip_horizontally = attr & 0b0100_0000 != 0;
                let flip_vertically = attr & 0b1000_0000 != 0;
                let x = self.attributes[offset + 3] as usize;

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

                let tile_offset = chr_bank + tile_idx * 16 + row as u16;

                if count < 8 {
                    let chr_low = self.read_chr(tile_offset);
                    let chr_high = self.read_chr(tile_offset + 8);
                    let mut chr = [0u8; 8];

                    for i in 0..8 {
                        let mask = 1 << if flip_horizontally { i } else { 7 - i };
                        let p1: u8 = (chr_low & mask != 0).into();
                        let p2: u8 = (chr_high & mask != 0).into();
                        let pattern = (p2 << 1) | p1;
                        chr[i] = pattern;
                    }

                    self.scanline_sprites[count] = SpriteData {
                        x,
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

        self.visible_sprites_count = count;
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
            (None, None) => COLOR_PALETTE[self.palette[0] as usize],
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

        Self::set_pixel(frame, x, y, color);
    }

    #[inline]
    pub fn vblank_started(&self) -> bool {
        self.regs.status.contains(Status::VBLANK_STARTED)
    }

    pub fn set_pixel(frame: &mut [u8], x: usize, y: usize, (r, g, b): (u8, u8, u8)) {
        if x < WIDTH && y < HEIGHT {
            let offset = (y * WIDTH + x) * 4;
            frame[offset] = r;
            frame[offset + 1] = g;
            frame[offset + 2] = b;
            frame[offset + 3] = 255;
        }
    }

    // https://www.nesdev.org/wiki/PPU_palettes
    fn sprite_color(&self, palette_idx: u8, color_idx: u8) -> Option<(u8, u8, u8)> {
        let palette_offset = SPRITE_PALETTES_OFFSET + palette_idx as usize * BYTES_PER_PALLETE;

        match color_idx {
            0 => None,
            1 => Some(COLOR_PALETTE[self.palette[palette_offset] as usize]),
            2 => Some(COLOR_PALETTE[self.palette[palette_offset + 1] as usize]),
            3 => Some(COLOR_PALETTE[self.palette[palette_offset + 2] as usize]),
            _ => unreachable!(),
        }
    }

    pub fn pull_nmi_status(&mut self) -> bool {
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
            0x2000..=0x3eff => {
                let res = self.data_buffer;
                self.data_buffer = self.read_nametable(addr);
                res
            }
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
            0x2002 => self.regs.read_status(self.open_bus),
            0x2004 => self.read_oam_data_reg(),
            0x2007 => self.read_data_reg(),
            _ => 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        // https://www.nesdev.org/wiki/Open_bus_behavior#PPU_open_bus
        println!(
            "write_register {addr:04X} {data:02X}",
            addr = addr,
            data = data
        );
        self.open_bus = data;

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

    pub fn trace(&self) -> String {
        format!(
            "c{}s{}f{}v{:X}t{:X}x{:X}w{:X}f{}-c{:X}m{:X}s{:X}r{:X}",
            self.cycle,
            self.scanline,
            self.frame,
            self.regs.v,
            self.regs.t,
            self.regs.x,
            if self.regs.w { 1 } else { 0 },
            if self.regs.f { 1 } else { 0 },
            self.regs.ctrl.bits(),
            self.regs.mask.bits(),
            self.regs.status.bits(),
            self.open_bus,
        )
    }
}
