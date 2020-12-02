// https://wiki.nesdev.com/w/index.php/PPU_registers
pub struct PPU_Registers {
    pub ctrl: PPU_CTRL,
    pub mask: PPU_MASK,
    pub status: PPU_STATUS,
    pub oam_addr: u8,
    pub scroll: PPU_SCROLL,
    pub addr: PPU_ADDR,
}

impl PPU_Registers {
    pub fn new() -> Self {
        PPU_Registers {
            ctrl: PPU_CTRL { val: 0 },
            mask: PPU_MASK { val: 0 },
            status: PPU_STATUS { val: 0 },
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

#[derive(Clone, Copy)]
pub struct PPU_CTRL {
    pub val: u8,
}

pub enum SpriteSize {
    Sprite8x8,
    Sprite8x16,
}

impl PPU_CTRL {
    pub fn x_scroll_offset(self) -> u16 {
        if self.val & 0b1 != 0 {
            256
        } else {
            0
        }
    }

    pub fn y_scroll_offset(self) -> u16 {
        if self.val & 0b10 != 0 {
            240
        } else {
            0
        }
    }

    pub fn vram_addr_increment(self) -> u16 {
        if self.val & 0b100 != 0 {
            32
        } else {
            1
        }
    }

    pub fn sprite_pattern_table_addr(self) -> u16 {
        if self.val & 0b1000 != 0 {
            0x1000
        } else {
            0
        }
    }

    pub fn background_pattern_table_addr(self) -> SpriteSize {
        if self.val & 0b10000 != 0 {
            SpriteSize::Sprite8x16
        } else {
            SpriteSize::Sprite8x8
        }
    }

    pub fn nmi_at_vblank(self) -> bool {
        self.val & 0b100000 != 0
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

pub struct PPU_STATUS {
    pub val: u8,
}

impl PPU_STATUS {
    #[inline(always)]
    fn clear_bits(&mut self, regs: &mut PPU_Registers) {
        // clear bit 7
        self.val &= 0b01111111;
        // clear the address latch used by PPUSCROLL and PPUADDR
        regs.scroll.is_x = true;
        regs.addr.is_high = true;
    }

    pub fn sprite_overflow(&mut self, regs: &mut PPU_Registers) -> bool {
        let res = self.val & 0b100000 != 0;
        self.clear_bits(regs);
        res
    }

    pub fn sprite_0_hit(&mut self, regs: &mut PPU_Registers) -> bool {
        let res = self.val & 0b1000000 != 0;
        self.clear_bits(regs);
        res
    }

    pub fn vblank_started(&mut self, regs: &mut PPU_Registers) -> bool {
        let res = self.val & 0b10000000 != 0;
        self.clear_bits(regs);
        res
    }
}

pub struct PPU_SCROLL {
    pub x: u8,
    pub y: u8,
    pub is_x: bool,
}

pub struct PPU_ADDR {
    pub addr: u16,
    pub is_high: bool,
}
