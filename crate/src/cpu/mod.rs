pub mod assembler;
mod instructions;
pub mod mappers;
pub mod memory;
mod opcodes;
pub mod rom;

use bitflags::bitflags;

use crate::cpu::opcodes::INST_NAMES;

use self::memory::Memory;
use super::bus::Bus;
use std::fmt;

const RESET_VECTOR: u16 = 0xfffc;
const STACK_START: u16 = 0x100;
const STACK_TOP: u8 = 0xfd;
const DEFAULT_STATUS_STATE: u8 = 0b0010_0100;

// 7  bit  0
// ---- ----
// NVss DIZC
// |||| ||||
// |||| |||+- Carry
// |||| ||+-- Zero
// |||| |+--- Interrupt Disable
// |||| +---- Decimal
// ||++------ No CPU effect, see: the B flag
// |+-------- Overflow
// +--------- Negative

bitflags! {
    pub struct Status: u8 {
        const CARRY = 0b0000_0001;
        const ZERO = 0b0000_0010;
        const INTERRUPT_DISABLE = 0b0000_0100;
        const DECIMAL = 0b0000_1000;
        const BREAK1 = 0b0001_0000;
        const BREAK2 = 0b0010_0000;
        const OVERFLOW = 0b0100_0000;
        const NEGATIVE = 0b1000_0000;
    }
}

impl Status {
    fn update(&mut self, value: u8) {
        *self.0.bits_mut() = value;
    }
}

impl Status {
    fn new() -> Self {
        Status::from_bits_truncate(DEFAULT_STATUS_STATE)
    }
}

// Represents the state of a MOS 6502 CPU
#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    pub pc: u16,
    sp: u8,
    cycles: usize,
    status: Status,
    pub bus: Bus,
}

impl CPU {
    pub fn new(mut bus: Bus) -> CPU {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: bus.read_word(RESET_VECTOR),
            sp: STACK_TOP,
            cycles: 0,
            status: Status::new(),
            bus,
        }
    }

    fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = STACK_TOP;
        self.cycles = 0;
        self.status.update(DEFAULT_STATUS_STATE);
        self.pc = self.bus.read_word(RESET_VECTOR);
    }

    // Stack utils

    #[inline]
    fn push(&mut self, val: u8) {
        let addr = STACK_START + self.sp as u16;
        self.bus.write_byte(addr, val);
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
        self.bus.read_byte(STACK_START + self.sp as u16)
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
    fn zero_page_x(&mut self) -> u16 {
        // val = PEEK((arg + X) % 256)
        self.next_byte().wrapping_add(self.x) as u16
    }

    #[inline]
    fn zero_page_x_val(&mut self) -> u8 {
        let addr = self.zero_page_x();
        self.bus.read_byte(addr)
    }

    #[inline]
    fn zero_page_y(&mut self) -> u16 {
        self.next_byte().wrapping_add(self.y) as u16
    }

    #[inline]
    fn zero_page_y_val(&mut self) -> u8 {
        let addr = self.zero_page_y();
        self.bus.read_byte(addr)
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
        let res = addr.wrapping_add(x);

        // if page boundary crossed
        if add_on_boundary_crossed && self.page_crossed(addr, res) {
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
    fn page_crossed(&self, prev: u16, next: u16) -> bool {
        prev & 0xff00 != next & 0xff00
    }

    #[inline]
    fn absolute_y(&mut self, add_on_boundary_crossed: bool) -> u16 {
        let addr = self.next_word();
        let y = self.y as u16;
        let res = addr.wrapping_add(y);

        // if page boundary crossed
        if add_on_boundary_crossed && self.page_crossed(addr, res) {
            self.cycles += 1;
        }

        res
    }

    #[inline]
    fn absolute_y_val(&mut self, add_on_boundary_crossed: bool) -> u8 {
        let addr = self.absolute_y(add_on_boundary_crossed);
        self.bus.read_byte(addr)
    }

    // indirect_indexed
    #[inline]
    fn indirect_y(&mut self, add_on_boundary_crossed: bool) -> u16 {
        // val = PEEK(PEEK(arg) + PEEK((arg + 1) % 256) * 256 + Y)
        let addr1 = self.next_byte();
        let addr2 = addr1.wrapping_add(1);
        let val1 = self.bus.read_byte(addr1 as u16);
        let val2 = self.bus.read_byte(addr2 as u16);
        let addr = (val1 as u16) + (val2 as u16) * 256;
        let final_addr = addr.wrapping_add(self.y as u16);

        if add_on_boundary_crossed && self.page_crossed(addr, final_addr) {
            self.cycles += 1;
        }

        final_addr
    }

    #[inline]
    fn indirect_y_val(&mut self, add_on_boundary_crossed: bool) -> u8 {
        let addr = self.indirect_y(add_on_boundary_crossed);
        self.bus.read_byte(addr)
    }

    // indexed_indirect
    #[inline]
    fn indirect_x(&mut self) -> u16 {
        // val = PEEK(PEEK((arg + X) % 256) + PEEK((arg + X + 1) % 256) * 256)
        let addr = self.next_byte();
        let addr1 = addr.wrapping_add(self.x);
        let addr2 = addr1.wrapping_add(1);
        let val1 = self.bus.read_byte(addr1 as u16);
        let val2 = self.bus.read_byte(addr2 as u16);

        (val1 as u16) + (val2 as u16) * 256
    }

    #[inline]
    fn indirect_x_val(&mut self) -> u8 {
        let addr = self.indirect_x();
        self.bus.read_byte(addr)
    }

    #[inline]
    fn toggle_zero_flag(&mut self, val: u8) {
        self.status.set(Status::ZERO, val == 0);
    }

    #[inline]
    fn toggle_neg_flag(&mut self, val: u8) {
        self.status.set(Status::NEGATIVE, val >> 7 == 1);
    }

    #[inline]
    fn toggle_nz(&mut self, val: u8) {
        self.toggle_neg_flag(val);
        self.toggle_zero_flag(val);
    }

    pub fn state_fmt(&mut self) -> String {
        let opcode = self.bus.read_byte(self.pc);
        let mnemonic = INST_NAMES[opcode as usize].unwrap();

        format!(
            "{:04X} {:02X} {} A {:02X} X {:02X} Y {:02X} P {:02X} SP {:02X}",
            self.pc,
            opcode,
            mnemonic,
            self.a,
            self.x,
            self.y,
            self.status.bits(),
            self.sp
        )
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
            self.status.bits(),
            self.sp,
        )
    }
}
