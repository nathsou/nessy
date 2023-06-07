mod registers;
mod screen;
use crate::cpu::rom::{Mirroring, ROM};
use std::rc::Rc;

use self::registers::{Registers, PPU_CTRL, PPU_STATUS};
use self::screen::Screen;

#[allow(clippy::upper_case_acronyms)]
pub struct PPU {
    rom: Rc<ROM>,
    pub regs: Registers,
    vram: [u8; 2 * 1024],
    palette: [u8; 32],
    attributes: [u8; 64 * 4],
    screen: Screen,
    cycle: usize,
    scanline: u16,
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

    pub fn step(&mut self) {
        self.cycle += 1;

        if self.cycle == 341 {
            self.cycle = 0;
            self.scanline += 1;

            // VBlank
            if self.scanline == 241 {
                if self.regs.ctrl.contains(PPU_CTRL::GENERATE_NMI) {
                    self.regs.status.set(PPU_STATUS::VBLANK_STARTED, true);
                    self.nmi_triggered = true;
                }
            }

            if self.scanline == 262 {
                self.scanline = 0;
                self.regs.status.set(PPU_STATUS::VBLANK_STARTED, false);
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
        let in_vblank = self.regs.status.contains(PPU_STATUS::VBLANK_STARTED);

        if in_vblank && !prev_nmi_status && new_nmi_status {
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
