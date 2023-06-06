pub mod assembler;
mod instructions;
mod mappers;
pub mod memory;
mod opcodes;
pub mod rom;

use self::memory::Memory;
use self::rom::ROM;
use super::bus::Bus;

use std::fmt;

const RESET_VECTOR: u16 = 0xfffc;

// Represents the state of a MOS 6502 CPU
#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    pub pc: u16,
    sp: u8,
    cycles: u16,
    negative_flag: bool,
    overflow_flag: bool,
    dec_mode_flag: bool,
    break_command_flag1: bool,
    break_command_flag2: bool,
    interrupt_disable_flag: bool,
    zero_flag: bool,
    carry_flag: bool,
    pub bus: Bus,
}

impl CPU {
    pub fn new(bus: Bus) -> CPU {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: bus.read_word(RESET_VECTOR),
            sp: 0xfd,
            cycles: 0,
            carry_flag: false,
            zero_flag: false,
            interrupt_disable_flag: true,
            dec_mode_flag: false,
            break_command_flag1: false,
            break_command_flag2: true,
            overflow_flag: false,
            negative_flag: false,
            bus,
        }
    }

    fn reset(&mut self) {
        println!("0xfffc: {:?}", self.bus.read_word(RESET_VECTOR));
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xfd;
        self.cycles = 0;
        self.carry_flag = false;
        self.zero_flag = false;
        self.interrupt_disable_flag = true;
        self.dec_mode_flag = false;
        self.break_command_flag1 = false;
        self.break_command_flag2 = true;
        self.overflow_flag = false;
        self.negative_flag = false;
        self.pc = self.bus.read_word(RESET_VECTOR);
    }

    // Stack utils

    #[inline]
    fn push(&mut self, val: u8) {
        let addr = self.sp as usize;
        self.bus.write_byte(addr as u16, val);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn push_word(&mut self, val: u16) {
        let high = (val >> 8) as u8;
        let low = (val & 0xff) as u8;

        self.push(high);
        self.push(low);
    }

    #[inline]
    fn pull(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.read_byte(self.sp as u16)
    }

    #[inline]
    fn pull_word(&mut self) -> u16 {
        let low = self.pull() as u16;
        let high = self.pull() as u16;
        high << 8 | low
    }

    // Memory utils
    #[inline]
    fn next_byte(&mut self) -> u8 {
        let byte = self.bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    #[inline]
    fn next_word(&mut self) -> u16 {
        let low = self.bus.read_byte(self.pc) as u16;
        let high = self.bus.read_byte(self.pc + 1) as u16;
        self.pc = self.pc.wrapping_add(2);
        high << 8 | low
    }

    // Addressing modes utils

    #[inline]
    fn zero_page(&mut self) -> u8 {
        self.next_byte()
    }

    #[inline]
    fn zero_page_val(&mut self) -> u8 {
        let addr = self.next_byte() as u16;
        self.bus.read_byte(addr)
    }

    #[inline]
    fn zero_page_x(&mut self) -> u8 {
        let zp = self.zero_page();
        zp.wrapping_add(self.x)
    }

    #[inline]
    fn zero_page_x_val(&mut self) -> u8 {
        let addr = self.zero_page_x() as u16;
        self.bus.read_byte(addr)
    }

    #[inline]
    fn zero_page_y(&mut self) -> u8 {
        let zp = self.zero_page();
        zp.wrapping_add(self.y)
    }

    #[inline]
    fn absolute(&mut self) -> u16 {
        self.next_word()
    }

    #[inline]
    fn absolute_val(&mut self) -> u8 {
        let addr = self.absolute();
        self.bus.read_byte(addr)
    }

    #[inline]
    fn absolute_x(&mut self, add_on_boundary_crossed: bool) -> u16 {
        let addr = self.next_word();
        let x = self.x as u16;
        let (res, crossed) = addr.overflowing_add(x);

        // if page boundary crossed
        if add_on_boundary_crossed && crossed {
            self.cycles += 1;
        }

        res
    }

    #[inline]
    fn absolute_x_val(&mut self, add_on_boundary_crossed: bool) -> u8 {
        let addr = self.absolute_x(add_on_boundary_crossed);
        self.bus.read_byte(addr)
    }

    #[inline]
    fn absolute_y(&mut self, add_on_boundary_crossed: bool) -> u16 {
        let addr = self.next_word();
        let y = self.y as u16;
        let (res, crossed) = addr.overflowing_add(y);

        // if page boundary crossed
        if add_on_boundary_crossed && crossed {
            self.cycles += 1;
        }

        res
    }

    #[inline]
    fn absolute_y_val(&mut self, add_on_boundary_crossed: bool) -> u8 {
        let addr = self.absolute_y(add_on_boundary_crossed);
        self.bus.read_byte(addr)
    }

    // indirect
    #[inline]
    fn indirect(&mut self) -> u16 {
        let addr = self.next_word();
        self.bus.read_word(addr)
    }

    // indirect_indexed
    #[inline]
    fn indirect_y(&mut self, add_on_boundary_crossed: bool) -> u16 {
        let addr = self.next_word();
        let addr = self.bus.read_byte(addr) as u16;
        let y = self.y as u16;
        let (addr, crossed) = addr.overflowing_add(y);

        if add_on_boundary_crossed && crossed {
            self.cycles += 1;
        }

        self.bus.read_word(addr)
    }

    #[inline]
    fn indirect_y_val(&mut self, add_on_boundary_crossed: bool) -> u8 {
        let addr = self.indirect_y(add_on_boundary_crossed);
        self.bus.read_byte(addr)
    }

    // indexed_indirect
    #[inline]
    fn indirect_x(&mut self) -> u16 {
        let addr = self.zero_page();
        let addr = addr.wrapping_add(self.x) as u16;
        self.bus.read_word(addr)
    }

    #[inline]
    fn indirect_x_val(&mut self) -> u8 {
        let addr = self.indirect_x();
        self.bus.read_byte(addr)
    }

    #[inline]
    fn toggle_zero_flag(&mut self, val: u8) {
        self.zero_flag = val == 0;
    }

    #[inline]
    fn toggle_neg_flag(&mut self, val: u8) {
        self.negative_flag = val >> 7 == 1;
    }

    #[inline]
    fn toggle_nz(&mut self, val: u8) {
        self.toggle_neg_flag(val);
        self.toggle_zero_flag(val);
    }

    // Flags utils
    fn flags_to_u8(&self) -> u8 {
        let mut flags = 0;
        if self.carry_flag {
            flags |= 0b00000001;
        }
        if self.zero_flag {
            flags |= 0b00000010;
        }
        if self.interrupt_disable_flag {
            flags |= 0b00000100;
        }
        if self.dec_mode_flag {
            flags |= 0b00001000;
        }
        if self.break_command_flag1 {
            flags |= 0b00010000;
        }
        if self.break_command_flag2 {
            flags |= 0b00100000;
        }
        if self.overflow_flag {
            flags |= 0b01000000;
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
        self.break_command_flag1 = (flags >> 4) & 1 == 1;
        self.break_command_flag2 = (flags >> 5) & 1 == 1;
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
                    self.bus.ram.ram[(0xffu8).wrapping_sub(idx as u8) as usize]
                )
            })
            .collect::<String>()
            .trim()
            .to_owned()
    }
}

impl fmt::Debug for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.a,
            self.x,
            self.y,
            self.flags_to_u8(),
            self.sp,
        )
    }
}
