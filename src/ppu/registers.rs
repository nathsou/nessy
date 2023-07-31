use bitflags::bitflags;

use crate::savestate::{self, SaveStateError};

// https://wiki.nesdev.com/w/index.php/PPU_registers
pub struct Registers {
    pub v: u16,  // current vram address
    pub t: u16,  // temp vram address
    pub x: u8,   // fine x scroll
    pub w: bool, // write toggle
    pub f: bool, // even/odd frame flag

    pub ctrl: Ctrl,
    pub mask: Mask,
    pub status: Status,
    pub oam_addr: u8,
    pub scroll: Scroll,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            v: 0,
            t: 0,
            x: 0,
            w: false,
            f: false,
            ctrl: Ctrl::empty(),
            mask: Mask::empty(),
            status: Status::empty(),
            oam_addr: 0,
            scroll: Scroll {
                x: 0,
                y: 0,
                is_x: true,
            },
        }
    }

    pub fn fine_y(&self) -> u8 {
        ((self.v >> 12) & 0b111) as u8
    }
}

// 7  bit  0
// ---- ----
// VPHB SINN
// |||| ||||
// |||| ||++- Base nametable address
// |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
// |||| |+--- VRAM address increment per CPU read/write of PPUDATA
// |||| |     (0: add 1, going across; 1: add 32, going down)
// |||| +---- Sprite pattern table address for 8x8 sprites
// ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
// |||+------ Background pattern table address (0: $0000; 1: $1000)
// ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
// |+-------- PPU master/slave select
// |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
// +--------- Generate an NMI at the start of the
//            vertical blanking interval (0: off; 1: on)

bitflags! {
    #[derive(Clone, Copy)]
    pub struct Ctrl: u8 {
        const NAMETABLE1              = 0b0000_0001;
        const NAMETABLE2              = 0b0000_0010;
        const VRAM_ADD_INCREMENT      = 0b0000_0100;
        const SPRITE_PATTERN_ADDR     = 0b0000_1000;
        const BACKROUND_PATTERN_ADDR  = 0b0001_0000;
        const SPRITE_SIZE             = 0b0010_0000;
        const MASTER_SLAVE_SELECT     = 0b0100_0000;
        const GENERATE_NMI            = 0b1000_0000;
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SpriteSize {
    Sprite8x8,
    Sprite8x16,
}

impl SpriteSize {
    pub fn height(&self) -> u8 {
        use SpriteSize::*;

        match self {
            Sprite8x8 => 8,
            Sprite8x16 => 16,
        }
    }
}

impl Ctrl {
    pub fn background_chr_offset(&self) -> u16 {
        if !self.contains(Ctrl::BACKROUND_PATTERN_ADDR) {
            0
        } else {
            0x1000
        }
    }

    #[inline]
    pub fn base_nametable_addr(&self) -> usize {
        match self.bits() & 0b11 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2c00,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn sprite_chr_offset(&self) -> u16 {
        if !self.contains(Ctrl::SPRITE_PATTERN_ADDR) {
            0
        } else {
            0x1000
        }
    }

    pub fn vram_addr_increment(self) -> u16 {
        if self.contains(Ctrl::VRAM_ADD_INCREMENT) {
            32
        } else {
            1
        }
    }

    pub fn sprite_size(&self) -> SpriteSize {
        if self.contains(Ctrl::SPRITE_SIZE) {
            SpriteSize::Sprite8x16
        } else {
            SpriteSize::Sprite8x8
        }
    }
}

impl Registers {
    pub fn write_ctrl(&mut self, data: u8) {
        *self.ctrl.0.bits_mut() = data;
        // t: ...GH.. ........ <- d: ......GH
        self.t = (self.t & 0xF3FF) | (((data as u16) & 0b11) << 10);
    }
}

// 7  bit  0
// ---- ----
// BGRs bMmG
// |||| ||||
// |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
// |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
// |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
// |||| +---- 1: Show background
// |||+------ 1: Show sprites
// ||+------- Emphasize red (green on PAL/Dendy)
// |+-------- Emphasize green (red on PAL/Dendy)
// +--------- Emphasize blue
bitflags! {
    pub struct Mask: u8 {
        const GREYSCALE             = 0b0000_0001;
        const SHOW_BACKGROUND_LEFT  = 0b0000_0010;
        const SHOW_SPRITES_LEFT     = 0b0000_0100;
        const SHOW_BACKGROUND       = 0b0000_1000;
        const SHOW_SPRITES          = 0b0001_0000;
        const EMPHASIZE_RED         = 0b0010_0000;
        const EMPHASIZE_GREEN       = 0b0100_0000;
        const EMPHASIZE_BLUE        = 0b1000_0000;
    }
}

// TODO: support greyscale etc..
impl Registers {
    pub fn write_mask(&mut self, data: u8) {
        *self.mask.0.bits_mut() = data;
    }

    pub fn show_background(&self) -> bool {
        self.mask.contains(Mask::SHOW_BACKGROUND)
    }

    pub fn show_sprites(&self) -> bool {
        self.mask.contains(Mask::SHOW_SPRITES)
    }

    pub fn rendering_enabled(&self) -> bool {
        self.show_background() || self.show_sprites()
    }

    pub fn show_leftmost_background(&self) -> bool {
        self.mask.contains(Mask::SHOW_BACKGROUND_LEFT)
    }

    pub fn show_leftmost_sprites(&self) -> bool {
        self.mask.contains(Mask::SHOW_SPRITES_LEFT)
    }
}

// 7  bit  0
// ---- ----
// VSO. ....
// |||| ||||
// |||+-++++- PPU open bus. Returns stale PPU bus contents.
// ||+------- Sprite overflow. The intent was for this flag to be set
// ||         whenever more than eight sprites appear on a scanline, but a
// ||         hardware bug causes the actual behavior to be more complicated
// ||         and generate false positives as well as false negatives; see
// ||         PPU sprite evaluation. This flag is set during sprite
// ||         evaluation and cleared at dot 1 (the second dot) of the
// ||         pre-render line.
// |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
// |          a nonzero background pixel; cleared at dot 1 of the pre-render
// |          line.  Used for raster timing.
// +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
//            Set at dot 1 of line 241 (the line *after* the post-render
//            line); cleared after reading $2002 and at dot 1 of the
//            pre-render line.

bitflags! {
    pub struct Status: u8 {
        const UNUSED1          = 0b00000001;
        const UNUSED2         = 0b00000010;
        const UNUSED3         = 0b00000100;
        const UNUSED4         = 0b00001000;
        const UNUSED5         = 0b00010000;
        const SPRITE_OVERFLOW  = 0b00100000;
        const SPRITE_ZERO_HIT  = 0b01000000;
        const VBLANK_STARTED   = 0b10000000;
    }
}

impl Registers {
    pub fn read_status(&mut self, open_bus: u8) -> u8 {
        let res = (self.status.bits() & 0b1110_0000) | (open_bus & 0b0001_1111);
        self.status.remove(Status::VBLANK_STARTED);
        self.w = false;
        res
    }
}

impl Registers {
    pub fn write_oam_address(&mut self, val: u8) {
        self.oam_addr = val;
    }
}

impl Registers {
    pub fn write_scroll(&mut self, data: u8) {
        if !self.w {
            // t: ....... ...ABCDE <- d: ABCDE...
            // x:              FGH <- d: .....FGH
            // w:                  <- 1
            self.t = (self.t & 0xFFE0) | ((data as u16) >> 3);
            self.x = data & 0b111;
            self.w = true;

            self.scroll.is_x = false;
            self.scroll.x = data;
        } else {
            // t: FGH..AB CDE..... <- d: ABCDEFGH
            // w:                  <- 0
            self.t = (self.t & 0x8FFF) | (((data as u16) & 0b111) << 12);
            self.t = (self.t & 0xFC1F) | (((data as u16) & 0b11111000) << 2);
            self.w = false;

            self.scroll.is_x = true;
            self.scroll.y = data;
        }
    }

    pub fn increment_x(&mut self) {
        if self.v & 0x001F == 31 {
            // if coarse X == 31
            self.v &= !0x001F; // coarse X = 0
            self.v ^= 0x0400; // switch horizontal nametable
        } else {
            self.v += 1; // increment coarse X
        }
    }

    pub fn increment_y(&mut self) {
        // if fine Y < 7
        if self.v & 0x7000 != 0x7000 {
            self.v += 0x1000; // increment fine Y
        } else {
            self.v &= !0x7000; // fine Y = 0
            let mut y = (self.v & 0x03E0) >> 5; // let y = coarse Y
            if y == 29 {
                y = 0; // coarse Y = 0
                self.v ^= 0x0800; // switch vertical nametable
            } else if y == 31 {
                y = 0; // coarse Y = 0, nametable not switched
            } else {
                y += 1; // increment coarse Y
            }

            self.v = (self.v & !0x03E0) | (y << 5); // put coarse Y back into v
        }
    }

    pub fn copy_x(&mut self) {
        // copy all bits related to horizontal position from t to v:
        // v: ....A.. ...BCDEF <- t: ....A.. ...BCDEF
        self.v = (self.v & 0xFBE0) | (self.t & 0x041F);
    }

    pub fn copy_y(&mut self) {
        // copy the vertical bits from t to v
        // v: GHIA.BC DEF..... <- t: GHIA.BC DEF.....
        self.v = (self.v & 0x841F) | (self.t & 0x7BE0);
    }
}

impl Registers {
    pub fn write_address(&mut self, data: u8) {
        if !self.w {
            // t: .CDEFGH ........ <- d: ..CDEFGH
            // t: Z...... ........ <- 0 (bit Z is cleared)
            // w:                  <- 1
            self.t = (self.t & 0x80FF) | (((data as u16) & 0b111111) << 8);
            self.t &= 0xBFFF;
            self.w = true;
        } else {
            // t: ....... ABCDEFGH <- d: ABCDEFGH
            // v: <...all bits...> <- t: <...all bits...>
            // w:                  <- 0
            self.t = (self.t & 0xFF00) | (data as u16);
            self.v = self.t;
            self.w = false;
        }
    }

    pub fn increment_vram_addr(&mut self) {
        let step = self.ctrl.vram_addr_increment();
        self.v = self.v.wrapping_add(step) & 0x3fff;
    }
}

pub struct Scroll {
    pub x: u8,
    pub y: u8,
    pub is_x: bool,
}

impl Scroll {
    pub fn write(&mut self, data: u8) {
        if self.is_x {
            self.x = data;
        } else {
            self.y = data;
        }

        self.is_x = !self.is_x;
    }
}

const PPU_REGS_SECTION_NAME: &str = "regs";

impl savestate::Save for Registers {
    fn save(&self, parent: &mut savestate::Section) {
        let s = parent.create_child(PPU_REGS_SECTION_NAME);

        s.data.write_u16(self.v);
        s.data.write_u16(self.t);
        s.data.write_u8(self.x);
        s.data.write_bool(self.w);
        s.data.write_bool(self.f);
        s.data.write_u8(self.ctrl.bits());
        s.data.write_u8(self.mask.bits());
        s.data.write_u8(self.status.bits());
        s.data.write_u8(self.oam_addr);
    }

    fn load(&mut self, parent: &mut savestate::Section) -> Result<(), SaveStateError> {
        let s = parent.get(PPU_REGS_SECTION_NAME)?;

        self.v = s.data.read_u16()?;
        self.t = s.data.read_u16()?;
        self.x = s.data.read_u8()?;
        self.w = s.data.read_bool()?;
        self.f = s.data.read_bool()?;
        *self.ctrl.0.bits_mut() = s.data.read_u8()?;
        *self.mask.0.bits_mut() = s.data.read_u8()?;
        *self.status.0.bits_mut() = s.data.read_u8()?;
        self.oam_addr = s.data.read_u8()?;

        Ok(())
    }
}
