pub mod assembler;
mod instructions;
mod mappers;
pub mod memory;
mod opcodes;
pub mod rom;

use self::memory::{MappedMemory, Memory};
use self::rom::ROM;
use ppu::PPU;

use std::fmt;

// Represents the state of a MOS 6502 CPU
pub struct MOS6502 {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u8,
    rom: Box<dyn Memory>,
    ram: RAM,
    cycles: u16,
    negative_flag: bool,
    overflow_flag: bool,
    dec_mode_flag: bool,
    break_command_flag: bool,
    interrupt_disable_flag: bool,
    zero_flag: bool,
    carry_flag: bool,
    ppu: PPU,
}

impl MOS6502 {
    pub fn new(rom: ROM) -> MOS6502 {
        let mut mapper = rom.get_mapper().expect("mapper error");

        MOS6502 {
            a: 0,
            x: 0,
            y: 0,
            pc: mapper.read_word(0xfffc),
            sp: 0xff,
            rom: mapper,
            ram: RAM { ram: [0; 0x800] },
            cycles: 0,
            negative_flag: false,
            overflow_flag: false,
            dec_mode_flag: false,
            break_command_flag: false,
            interrupt_disable_flag: false,
            zero_flag: false,
            carry_flag: false,
            ppu: PPU::new(),
        }
    }

    // pub fn program_counter(&self) -> u16 {
    //     self.pc
    // }

    pub fn interrupted(&self) -> bool {
        self.break_command_flag
    }

    // Stack utils

    #[inline(always)]
    fn push(&mut self, val: u8) {
        let addr = 0x100 | (self.sp as usize);
        self.write_byte(addr as u16, val);

        // decrement the stack pointer
        if self.sp == 0 {
            self.sp = 0xff;
        } else {
            self.sp -= 1;
        }
    }

    #[inline(always)]
    fn pull(&mut self) -> u8 {
        // increment the stack pointer
        if self.sp == 0xff {
            self.sp = 0;
        } else {
            self.sp += 1;
        }

        let addr = 0x100 | (self.sp as usize);
        self.read_byte(addr as u16)
    }

    #[inline(always)]
    fn pull_word(&mut self) -> u16 {
        let low = self.pull() as u16;
        let high = (self.pull() as u16) << 8;

        high | low
    }

    // Memory utils
    #[inline(always)]
    fn next_byte(&mut self) -> u8 {
        let byte = self.read_byte(self.pc);
        self.pc += 1;
        return byte;
    }

    #[inline(always)]
    fn next_word(&mut self) -> u16 {
        let high = (self.read_byte(self.pc + 1) as u16) << 8;
        let low = self.read_byte(self.pc) as u16;
        self.pc += 2;
        // The 6502 is little-endian
        return high | low;
    }

    // Adessing modes utils

    #[inline(always)]
    fn zero_page(&mut self) -> u8 {
        self.next_byte()
    }

    #[inline(always)]
    fn zero_page_val(&mut self) -> u8 {
        let addr = self.next_byte() as u16;
        self.read_byte(addr)
    }

    #[inline(always)]
    fn zero_page_x(&mut self) -> u8 {
        let zp = self.zero_page() as u16;
        ((zp + (self.x as u16)) & 0xff) as u8
    }

    #[inline(always)]
    fn zero_page_x_val(&mut self) -> u8 {
        let addr = self.zero_page_x() as u16;
        self.read_byte(addr)
    }

    #[inline(always)]
    fn zero_page_y(&mut self) -> u8 {
        let zp = self.zero_page() as u16;
        ((zp + (self.y as u16)) & 0xff) as u8
    }

    // #[inline(always)]
    // fn zero_page_y_val(&mut self) -> u8 {
    //     let addr = self.zero_page_y() as usize;
    //     self.read_byte(addr)
    // }

    #[inline(always)]
    fn absolute(&mut self) -> u16 {
        self.next_word()
    }

    #[inline(always)]
    fn absolute_val(&mut self) -> u8 {
        let addr = self.absolute() as u16;
        self.read_byte(addr)
    }

    #[inline(always)]
    fn absolute_x(&mut self, add_on_boundary_crossed: bool) -> u16 {
        let addr = self.next_word() as u16;
        let x = self.x as u16;

        // if page boundary crossed
        if add_on_boundary_crossed && (addr & 0xff) + x > 0xff {
            self.cycles += 1;
        }

        addr + x
    }

    #[inline(always)]
    fn absolute_x_val(&mut self, add_on_boundary_crossed: bool) -> u8 {
        let addr = self.absolute_x(add_on_boundary_crossed);
        self.read_byte(addr)
    }

    #[inline(always)]
    fn absolute_y(&mut self, add_on_boundary_crossed: bool) -> u16 {
        let addr = self.next_word();
        let y = self.y as u16;

        // if page boundary crossed
        if add_on_boundary_crossed && (addr & 0xff) + y > 0xff {
            self.cycles += 1;
        }

        addr + y
    }

    #[inline(always)]
    fn absolute_y_val(&mut self, add_on_boundary_crossed: bool) -> u8 {
        let addr = self.absolute_y(add_on_boundary_crossed);
        self.read_byte(addr)
    }

    // indirect
    #[inline(always)]
    fn indirect(&mut self) -> u16 {
        let addr = self.next_word();
        self.read_word(addr)
    }

    // indirect_indexed
    #[inline(always)]
    fn indirect_y(&mut self, add_on_boundary_crossed: bool) -> u16 {
        let addr = self.next_word();
        let addr = self.read_byte(addr) as u16;
        let y = self.y as u16;

        // if page boundary crossed
        if add_on_boundary_crossed && (addr & 0xff) + y > 0xff {
            self.cycles += 1;
        }

        self.read_word(addr + y)
    }

    #[inline(always)]
    fn indirect_y_val(&mut self, add_on_boundary_crossed: bool) -> u8 {
        let addr = self.indirect_y(add_on_boundary_crossed);
        self.read_byte(addr)
    }

    // indexed_indirect
    #[inline(always)]
    fn indirect_x(&mut self) -> u16 {
        let addr = self.zero_page();
        let addr = addr.wrapping_add(self.x) as u16;
        self.read_word(addr)
    }

    #[inline(always)]
    fn indirect_x_val(&mut self) -> u8 {
        let addr = self.indirect_x();
        self.read_byte(addr)
    }

    #[inline(always)]
    fn toggle_zero_flag(&mut self, val: u8) {
        self.zero_flag = val == 0;
    }

    #[inline(always)]
    fn toggle_neg_flag(&mut self, val: u8) {
        self.negative_flag = val >> 7 == 1;
    }

    #[inline(always)]
    fn toggle_nz(&mut self, val: u8) {
        self.toggle_neg_flag(val);
        self.toggle_zero_flag(val);
    }

    // Flags utils
    fn flags_to_u8(&self) -> u8 {
        let mut flags = 32u8; //5th bit is always set
        if self.carry_flag {
            flags |= 0b1;
        }
        if self.zero_flag {
            flags |= 0b10;
        }
        if self.interrupt_disable_flag {
            flags |= 0b100;
        }
        if self.dec_mode_flag {
            flags |= 0b1000;
        }
        if self.break_command_flag {
            flags |= 0b10000;
        }
        if self.overflow_flag {
            flags |= 0b1000000;
        }
        if self.negative_flag {
            flags |= 0b10000000;
        }

        flags
    }

    fn set_flags_from_u8(&mut self, flags: u8) {
        self.carry_flag = flags & 1 == 1;
        self.zero_flag = (flags >> 1) & 1 == 1;
        self.interrupt_disable_flag = (flags >> 2) & 1 == 1;
        self.dec_mode_flag = (flags >> 3) & 1 == 1;
        self.break_command_flag = (flags >> 4) & 1 == 1;
        self.overflow_flag = (flags >> 6) & 1 == 1;
        self.negative_flag = (flags >> 7) & 1 == 1;
    }

    fn flags_to_str(&self) -> String {
        let flags = self.flags_to_u8();
        let mut flags_str = ['N', 'V', '_', 'B', 'D', 'I', 'Z', 'C'];
        let mut curr_flag = 1u8;

        for i in 0..8 {
            if flags & curr_flag == 0 {
                flags_str[7 - i] = '_';
            }
            curr_flag <<= 1;
        }

        flags_str.iter().collect()
    }

    fn stack_to_str(&self) -> String {
        let stack_len = (0xff - self.sp) as usize;

        (0..stack_len)
            .map(|idx| {
                format!(
                    "{:02X} ",
                    self.ram.ram[(0xffu8).wrapping_sub(idx as u8) as usize]
                )
            })
            .collect::<String>()
            .trim()
            .to_owned()
    }
}

struct RAM {
    pub ram: [u8; 0x800],
}

impl Memory for RAM {
    fn read_byte(&mut self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        self.ram[addr as usize] = val;
    }
}

// https://wiki.nesdev.com/w/index.php/CPU_memory_map
impl Memory for MOS6502 {
    fn read_byte(&mut self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.ram.read_byte(addr)
        } else if addr < 0x4000 {
            self.ppu.read_byte(addr, &mut self.rom)
        } else if addr < 0x4018 {
            0 // APU
        } else {
            self.rom.read_byte(addr)
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        if addr < 0x2000 {
            self.ram.write_byte(addr, val);
        } else if addr < 0x4000 {
            self.ppu.write_byte(addr, val, &mut self.rom)
        } else if addr < 0x4018 {
            // APU
        } else {
            self.rom.write_byte(addr, val);
        }
    }
}

impl fmt::Debug for MOS6502 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "A: {:X}, X: {:X}, Y: {:X}, SP: {:X} {}, stack: [{}]",
            self.a,
            self.x,
            self.y,
            self.sp,
            self.flags_to_str(),
            self.stack_to_str(),
        )
    }
}
