mod instructions;
pub mod mappers;
pub mod memory;
pub mod rom;

use bitflags::bitflags;

use crate::savestate::{self, SaveStateError};

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
    total_cycles: u64,
    status: Status,
    pub bus: Bus,
    opcode: u8,
    irq_pending: bool,
    nmi_pending: bool,
    irq_sample: bool,
    irq_previous: bool,
    nmi_sample: bool,
    nmi_previous: bool,
    jammed: bool,
    instructions: [fn(&mut CPU); 256],
}

impl CPU {
    pub fn new(mut bus: Bus) -> CPU {
        let pc = bus.read_word(RESET_VECTOR);

        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc,
            sp: STACK_TOP,
            total_cycles: 0,
            status: Status::new(),
            bus,
            opcode: 0,
            irq_pending: false,
            nmi_pending: false,
            irq_sample: false,
            irq_previous: false,
            nmi_sample: false,
            nmi_previous: false,
            jammed: false,
            instructions: CPU::instructions_lut(),
        }
    }

    pub fn soft_reset(&mut self) {
        self.jammed = false;
        self.irq_pending = false;
        self.nmi_pending = false;
        self.irq_sample = false;
        self.irq_previous = false;
        self.nmi_sample = false;
        self.nmi_previous = false;
        self.bus.apu.soft_reset();
        self.read(self.pc);
        self.read(self.pc);
        for _ in 0..3 {
            self.read(STACK_START + self.sp as u16);
            self.sp = self.sp.wrapping_sub(1);
        }
        self.status.insert(Status::INTERRUPT_DISABLE);
        self.pc = self.read_word(RESET_VECTOR);
    }

    fn tick(&mut self) {
        self.bus.advance(1);
        self.total_cycles = self.total_cycles.wrapping_add(1);
        self.irq_previous = self.irq_sample;
        self.nmi_previous = self.nmi_sample;
        self.irq_sample = self.bus.irq_line() && !self.status.contains(Status::INTERRUPT_DISABLE);
        self.nmi_sample |= self.bus.ppu.is_asserting_nmi();
    }
    fn raw_read(&mut self, addr: u16) -> u8 {
        self.tick();
        self.bus.read_byte(addr)
    }
    fn service_dmc(&mut self, halted_addr: u16, oam: bool) {
        if let Some(addr) = self.bus.apu.pull_memory_read_request() {
            // DMA may halt reads, never CPU writes. DMC gets take priority over OAM gets.
            if !oam {
                self.raw_read(halted_addr);
                self.raw_read(halted_addr);
            }
            if self.total_cycles & 1 != 0 {
                self.raw_read(halted_addr);
            }
            let value = self.raw_read(addr);
            self.bus.apu.push_memory_read_response(value);
            if oam {
                // The stolen get is followed by a put-phase idle cycle before OAM resumes.
                self.raw_read(halted_addr);
            }
        }
    }
    fn read(&mut self, addr: u16) -> u8 {
        self.service_dmc(addr, false);
        self.raw_read(addr)
    }
    fn write(&mut self, addr: u16, val: u8) {
        self.tick();
        self.bus.write_byte(addr, val);
    }
    fn read_word(&mut self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr.wrapping_add(1)) as u16;
        lo | hi << 8
    }
    fn dma(&mut self) {
        self.bus.dma_transfer = false;
        // OAM gets use the same odd-cycle phase as DMC gets.
        let align = self.total_cycles & 1 == 0;
        self.raw_read(self.pc);
        if align {
            self.raw_read(self.pc);
        }
        for low in 0..256u16 {
            self.service_dmc(self.pc, true);
            let val = self.raw_read((self.bus.dma_page as u16) << 8 | low);
            self.write(0x2004, val);
        }
    }

    // Stack utils

    fn push(&mut self, val: u8) {
        let addr = STACK_START + self.sp as u16;
        self.write(addr, val);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn push_word(&mut self, val: u16) {
        let high = (val >> 8) as u8;
        let low = (val & 0xff) as u8;

        self.push(high);
        self.push(low);
    }

    fn pull(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read(STACK_START + self.sp as u16)
    }

    fn pull_word(&mut self) -> u16 {
        let low = self.pull() as u16;
        let high = self.pull() as u16;
        high << 8 | low
    }

    // Memory utils

    fn next_byte(&mut self) -> u8 {
        let byte = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    fn next_word(&mut self) -> u16 {
        let low = self.next_byte() as u16;
        let high = self.next_byte() as u16;
        high << 8 | low
    }

    // Addressing modes utils

    fn zero_page(&mut self) -> u8 {
        self.next_byte()
    }

    fn zero_page_val(&mut self) -> u8 {
        let addr = self.next_byte() as u16;
        self.read(addr)
    }

    fn zero_page_x(&mut self) -> u16 {
        // val = PEEK((arg + X) % 256)
        {
            let addr = self.next_byte();
            self.read(addr as u16);
            addr.wrapping_add(self.x) as u16
        }
    }

    fn zero_page_x_val(&mut self) -> u8 {
        let addr = self.zero_page_x();
        self.read(addr)
    }

    fn zero_page_y(&mut self) -> u16 {
        {
            let addr = self.next_byte();
            self.read(addr as u16);
            addr.wrapping_add(self.y) as u16
        }
    }

    fn zero_page_y_val(&mut self) -> u8 {
        let addr = self.zero_page_y();
        self.read(addr)
    }

    fn absolute(&mut self) -> u16 {
        self.next_word()
    }

    fn absolute_val(&mut self) -> u8 {
        let addr = self.absolute();
        self.read(addr)
    }

    fn absolute_x(&mut self, add_on_boundary_crossed: bool) -> u16 {
        let addr = self.next_word();
        let x = self.x as u16;
        let res = addr.wrapping_add(x);

        // if page boundary crossed
        if !add_on_boundary_crossed || self.page_crossed(addr, res) {
            self.read((addr & 0xff00) | (res & 0xff));
        }

        res
    }

    fn absolute_x_val(&mut self, add_on_boundary_crossed: bool) -> u8 {
        let addr = self.absolute_x(add_on_boundary_crossed);
        self.read(addr)
    }

    fn page_crossed(&self, prev: u16, next: u16) -> bool {
        prev & 0xff00 != next & 0xff00
    }

    fn absolute_y(&mut self, add_on_boundary_crossed: bool) -> u16 {
        let addr = self.next_word();
        let y = self.y as u16;
        let res = addr.wrapping_add(y);

        // if page boundary crossed
        if !add_on_boundary_crossed || self.page_crossed(addr, res) {
            self.read((addr & 0xff00) | (res & 0xff));
        }

        res
    }

    fn absolute_y_val(&mut self, add_on_boundary_crossed: bool) -> u8 {
        let addr = self.absolute_y(add_on_boundary_crossed);
        self.read(addr)
    }

    // indirect_indexed

    fn indirect_y(&mut self, add_on_boundary_crossed: bool) -> u16 {
        // val = PEEK(PEEK(arg) + PEEK((arg + 1) % 256) * 256 + Y)
        let addr1 = self.next_byte();
        let addr2 = addr1.wrapping_add(1);
        let val1 = self.read(addr1 as u16);
        let val2 = self.read(addr2 as u16);
        let addr = (val1 as u16) + (val2 as u16) * 256;
        let final_addr = addr.wrapping_add(self.y as u16);

        if !add_on_boundary_crossed || self.page_crossed(addr, final_addr) {
            self.read((addr & 0xff00) | (final_addr & 0xff));
        }

        final_addr
    }

    fn indirect_y_val(&mut self, add_on_boundary_crossed: bool) -> u8 {
        let addr = self.indirect_y(add_on_boundary_crossed);
        self.read(addr)
    }

    // indexed_indirect

    fn indirect_x(&mut self) -> u16 {
        // val = PEEK(PEEK((arg + X) % 256) + PEEK((arg + X + 1) % 256) * 256)
        let addr = self.next_byte();
        self.read(addr as u16);
        let addr1 = addr.wrapping_add(self.x);
        let addr2 = addr1.wrapping_add(1);
        let val1 = self.read(addr1 as u16);
        let val2 = self.read(addr2 as u16);

        (val1 as u16) + (val2 as u16) * 256
    }

    fn indirect_x_val(&mut self) -> u8 {
        let addr = self.indirect_x();
        self.read(addr)
    }

    fn toggle_zero_flag(&mut self, val: u8) {
        self.status.set(Status::ZERO, val == 0);
    }

    fn toggle_neg_flag(&mut self, val: u8) {
        self.status.set(Status::NEGATIVE, val >> 7 == 1);
    }

    fn toggle_nz(&mut self, val: u8) {
        self.toggle_neg_flag(val);
        self.toggle_zero_flag(val);
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

const CPU_SECTION_NAME: &str = "cpu";

impl savestate::Save for CPU {
    fn save(&self, parent: &mut savestate::Section) {
        let s = parent.create_child(CPU_SECTION_NAME);

        s.data.write_u8(self.a);
        s.data.write_u8(self.x);
        s.data.write_u8(self.y);
        s.data.write_u16(self.pc);
        s.data.write_u8(self.sp);
        s.data.write_u64(self.total_cycles);
        s.data.write_u8(self.status.bits());

        s.data.write_u8(self.opcode);
        for b in [
            self.irq_pending,
            self.nmi_pending,
            self.irq_sample,
            self.irq_previous,
            self.nmi_sample,
            self.nmi_previous,
            self.jammed,
        ] {
            s.data.write_bool(b);
        }
        self.bus.save(s);
    }

    fn load(&mut self, parent: &mut savestate::Section) -> Result<(), SaveStateError> {
        let s = parent.get(CPU_SECTION_NAME)?;

        self.a = s.data.read_u8()?;
        self.x = s.data.read_u8()?;
        self.y = s.data.read_u8()?;
        self.pc = s.data.read_u16()?;
        self.sp = s.data.read_u8()?;
        self.total_cycles = s.data.read_u64()?;
        *self.status.0.bits_mut() = s.data.read_u8()?;

        self.opcode = s.data.read_u8()?;
        self.irq_pending = s.data.read_bool()?;
        self.nmi_pending = s.data.read_bool()?;
        self.irq_sample = s.data.read_bool()?;
        self.irq_previous = s.data.read_bool()?;
        self.nmi_sample = s.data.read_bool()?;
        self.nmi_previous = s.data.read_bool()?;
        self.jammed = s.data.read_bool()?;
        self.bus.load(s)?;

        Ok(())
    }
}
