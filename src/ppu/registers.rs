use bitflags::bitflags;

// https://wiki.nesdev.com/w/index.php/PPU_registers
pub struct Registers {
    pub ctrl: PPU_CTRL,
    pub mask: PPU_MASK,
    pub status: PPU_STATUS,
    pub oam_addr: u8,
    pub scroll: PPU_SCROLL,
    pub addr: PPU_ADDR,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            ctrl: PPU_CTRL::empty(),
            mask: PPU_MASK { val: 0 },
            status: PPU_STATUS::empty(),
            oam_addr: 0,
            scroll: PPU_SCROLL {
                x: 0,
                y: 0,
                is_x: true,
            },
            addr: PPU_ADDR {
                addr: 0,
                is_high: true,
            },
        }
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
    pub struct PPU_CTRL: u8 {
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

impl PPU_CTRL {
    pub fn base_nametable_addr(&self) -> u16 {
        match self.bits() & 0b11 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2c00,
            _ => unreachable!(),
        }
    }
}

pub enum SpriteSize {
    Sprite8x8,
    Sprite8x16,
}

impl PPU_CTRL {
    pub fn vram_addr_increment(self) -> u8 {
        if self.contains(PPU_CTRL::VRAM_ADD_INCREMENT) {
            32
        } else {
            1
        }
    }

    #[inline]
    pub fn update(&mut self, data: u8) {
        *self.0.bits_mut() = data;
    }
}

pub struct PPU_MASK {
    pub val: u8,
}

// TODO: support greyscale etc..
impl PPU_MASK {
    pub fn show_background(self) -> bool {
        self.val & 0b1000 != 0
    }

    pub fn show_sprites(self) -> bool {
        self.val & 0b10000 != 0
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
    pub struct PPU_STATUS: u8 {
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

pub struct PPU_SCROLL {
    pub x: u8,
    pub y: u8,
    pub is_x: bool,
}

impl PPU_SCROLL {
    pub fn write(&mut self, data: u8) {
        if self.is_x {
            self.x = data;
        } else {
            self.y = data;
        }

        self.is_x = !self.is_x;
    }
}

pub struct PPU_ADDR {
    pub addr: u16,
    pub is_high: bool,
}

impl PPU_ADDR {
    pub fn write(&mut self, data: u8) {
        if self.is_high {
            self.addr = (self.addr & 0x00ff) | ((data as u16) << 8);
        } else {
            self.addr = (self.addr & 0xff00) | (data as u16);
        }

        self.addr &= 0x3fff;
        self.is_high = !self.is_high;
    }

    pub fn increment(&mut self, step: u8) {
        self.addr = self.addr.wrapping_add(step as u16) & 0x3fff;
    }
}
