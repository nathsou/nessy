use super::memory::Memory;
use super::opcodes::INST_CYCLES;
use super::{Status, CPU};
use crate::bus::Interrupt;

const NMI_VECTOR: u16 = 0xfffa;
const IRQ_VECTOR: u16 = 0xfffe;

impl CPU {
    pub fn instructions_lut() -> [fn(&mut CPU); 256] {
        let mut instructions: [fn(&mut CPU); 256] = [CPU::nop; 256];

        instructions[0x00] = CPU::brk;
        instructions[0xEA] = CPU::nop;
        instructions[0xA9] = CPU::lda_imm;
        instructions[0xA5] = CPU::lda_zp;
        instructions[0xB5] = CPU::lda_zp_x;
        instructions[0xAD] = CPU::lda_abs;
        instructions[0xBD] = CPU::lda_abs_x;
        instructions[0xB9] = CPU::lda_abs_y;
        instructions[0xA1] = CPU::lda_ind_x;
        instructions[0xB1] = CPU::lda_ind_y;
        instructions[0xA2] = CPU::ldx_imm;
        instructions[0xA6] = CPU::ldx_zp;
        instructions[0xB6] = CPU::ldx_zp_y;
        instructions[0xAE] = CPU::ldx_abs;
        instructions[0xBE] = CPU::ldx_abs_y;
        instructions[0xA0] = CPU::ldy_imm;
        instructions[0xA4] = CPU::ldy_zp;
        instructions[0xB4] = CPU::ldy_zp_x;
        instructions[0xAC] = CPU::ldy_abs;
        instructions[0xBC] = CPU::ldy_abs_x;
        instructions[0x85] = CPU::sta_zp;
        instructions[0x95] = CPU::sta_zp_x;
        instructions[0x8D] = CPU::sta_abs;
        instructions[0x9D] = CPU::sta_abs_x;
        instructions[0x99] = CPU::sta_abs_y;
        instructions[0x81] = CPU::sta_ind_x;
        instructions[0x91] = CPU::sta_ind_y;
        instructions[0x86] = CPU::stx_zp;
        instructions[0x96] = CPU::stx_zp_y;
        instructions[0x8E] = CPU::stx_abs;
        instructions[0x84] = CPU::sty_zp;
        instructions[0x94] = CPU::sty_zp_x;
        instructions[0x8C] = CPU::sty_abs;
        instructions[0x69] = CPU::adc_imm;
        instructions[0x65] = CPU::adc_zp;
        instructions[0x75] = CPU::adc_zp_x;
        instructions[0x6D] = CPU::adc_abs;
        instructions[0x7D] = CPU::adc_abs_x;
        instructions[0x79] = CPU::adc_abs_y;
        instructions[0x61] = CPU::adc_ind_x;
        instructions[0x71] = CPU::adc_ind_y;
        instructions[0xE9] = CPU::sbc_imm;
        instructions[0xE5] = CPU::sbc_zp;
        instructions[0xF5] = CPU::sbc_zp_x;
        instructions[0xED] = CPU::sbc_abs;
        instructions[0xFD] = CPU::sbc_abs_x;
        instructions[0xF9] = CPU::sbc_abs_y;
        instructions[0xE1] = CPU::sbc_ind_x;
        instructions[0xF1] = CPU::sbc_ind_y;
        instructions[0xAA] = CPU::tax;
        instructions[0xA8] = CPU::tay;
        instructions[0xBA] = CPU::tsx;
        instructions[0x8A] = CPU::txa;
        instructions[0x9A] = CPU::txs;
        instructions[0x98] = CPU::tya;
        instructions[0x29] = CPU::and_imm;
        instructions[0x25] = CPU::and_zp;
        instructions[0x35] = CPU::and_zp_x;
        instructions[0x2D] = CPU::and_abs;
        instructions[0x3D] = CPU::and_abs_x;
        instructions[0x39] = CPU::and_abs_y;
        instructions[0x21] = CPU::and_ind_x;
        instructions[0x31] = CPU::and_ind_y;
        instructions[0x09] = CPU::ora_imm;
        instructions[0x05] = CPU::ora_zp;
        instructions[0x15] = CPU::ora_zp_x;
        instructions[0x0D] = CPU::ora_abs;
        instructions[0x1D] = CPU::ora_abs_x;
        instructions[0x19] = CPU::ora_abs_y;
        instructions[0x01] = CPU::ora_ind_x;
        instructions[0x11] = CPU::ora_ind_y;
        instructions[0x49] = CPU::eor_imm;
        instructions[0x45] = CPU::eor_zp;
        instructions[0x55] = CPU::eor_zp_x;
        instructions[0x4D] = CPU::eor_abs;
        instructions[0x5D] = CPU::eor_abs_x;
        instructions[0x59] = CPU::eor_abs_y;
        instructions[0x41] = CPU::eor_ind_x;
        instructions[0x51] = CPU::eor_ind_y;
        instructions[0x0A] = CPU::asl_acc;
        instructions[0x06] = CPU::asl_zp;
        instructions[0x16] = CPU::asl_zp_x;
        instructions[0x0E] = CPU::asl_abs;
        instructions[0x1E] = CPU::asl_abs_x;
        instructions[0x4A] = CPU::lsr_acc;
        instructions[0x46] = CPU::lsr_zp;
        instructions[0x56] = CPU::lsr_zp_x;
        instructions[0x4E] = CPU::lsr_abs;
        instructions[0x5E] = CPU::lsr_abs_x;
        instructions[0xE6] = CPU::inc_zp;
        instructions[0xF6] = CPU::inc_zp_x;
        instructions[0xEE] = CPU::inc_abs;
        instructions[0xFE] = CPU::inc_abs_x;
        instructions[0xE8] = CPU::inx;
        instructions[0xC8] = CPU::iny;
        instructions[0xC6] = CPU::dec_zp;
        instructions[0xD6] = CPU::dec_zp_x;
        instructions[0xCE] = CPU::dec_abs;
        instructions[0xDE] = CPU::dec_abs_x;
        instructions[0xCA] = CPU::dex;
        instructions[0x88] = CPU::dey;
        instructions[0x4C] = CPU::jmp_abs;
        instructions[0x6C] = CPU::jmp_ind;
        instructions[0x90] = CPU::bcc_rel;
        instructions[0xB0] = CPU::bcs_rel;
        instructions[0xF0] = CPU::beq_rel;
        instructions[0xD0] = CPU::bne_rel;
        instructions[0x10] = CPU::bpl_rel;
        instructions[0x30] = CPU::bmi_rel;
        instructions[0x50] = CPU::bvc_rel;
        instructions[0x70] = CPU::bvs_rel;
        instructions[0x18] = CPU::clc;
        instructions[0x38] = CPU::sec;
        instructions[0xD8] = CPU::cld;
        instructions[0xF8] = CPU::sed;
        instructions[0x58] = CPU::cli;
        instructions[0x78] = CPU::sei;
        instructions[0xB8] = CPU::clv;
        instructions[0xC9] = CPU::cmp_imm;
        instructions[0xC5] = CPU::cmp_zp;
        instructions[0xD5] = CPU::cmp_zp_x;
        instructions[0xCD] = CPU::cmp_abs;
        instructions[0xDD] = CPU::cmp_abs_x;
        instructions[0xD9] = CPU::cmp_abs_y;
        instructions[0xC1] = CPU::cmp_ind_x;
        instructions[0xD1] = CPU::cmp_ind_y;
        instructions[0xE0] = CPU::cpx_imm;
        instructions[0xE4] = CPU::cpx_zp;
        instructions[0xEC] = CPU::cpx_abs;
        instructions[0xC0] = CPU::cpy_imm;
        instructions[0xC4] = CPU::cpy_zp;
        instructions[0xCC] = CPU::cpy_abs;
        instructions[0x48] = CPU::pha;
        instructions[0x68] = CPU::pla;
        instructions[0x28] = CPU::plp;
        instructions[0x08] = CPU::php;
        instructions[0x20] = CPU::jsr;
        instructions[0x60] = CPU::rts;
        instructions[0x40] = CPU::rti;
        instructions[0x24] = CPU::bit_zp;
        instructions[0x2C] = CPU::bit_abs;
        instructions[0x2A] = CPU::rol_acc;
        instructions[0x26] = CPU::rol_zp;
        instructions[0x36] = CPU::rol_zp_x;
        instructions[0x2E] = CPU::rol_abs;
        instructions[0x3E] = CPU::rol_abs_x;
        instructions[0x6A] = CPU::ror_acc;
        instructions[0x66] = CPU::ror_zp;
        instructions[0x76] = CPU::ror_zp_x;
        instructions[0x6E] = CPU::ror_abs;
        instructions[0x7E] = CPU::ror_abs_x;

        instructions
    }

    pub fn step(&mut self) -> u32 {
        self.instr_cycles = 0;

        if self.bus.dma_transfer {
            self.bus.dma_transfer = false;
            self.stall += 513 + (self.total_cycles & 1);
        }

        if self.bus.apu.is_stalling_cpu() {
            return 1;
        }

        if self.stall > 0 {
            self.stall -= 1;
            return 1;
        }

        match self.bus.pull_interrupt() {
            Interrupt::None => {}
            Interrupt::Irq => {
                if !self.status.contains(Status::INTERRUPT_DISABLE) {
                    self.irq();
                }
            }
            Interrupt::Nmi => {
                self.nmi();
            }
        }

        let op_code = self.next_byte();
        self.instructions[op_code as usize](self);

        let instr_cycles = self.instr_cycles + INST_CYCLES[op_code as usize];
        self.total_cycles += instr_cycles;

        instr_cycles
    }

    // interrupts
    fn brk(&mut self) {
        self.push_word(self.pc);
        self.php();
        self.sei();
        self.pc = self.bus.read_word(IRQ_VECTOR);
    }

    fn nmi(&mut self) {
        self.push_word(self.pc);
        self.php();
        self.sei();
        self.pc = self.bus.read_word(NMI_VECTOR);
        self.instr_cycles += 7;
    }

    fn irq(&mut self) {
        self.brk();
        self.instr_cycles += 7;
    }

    // NOP: No Operation

    fn nop(&mut self) {}

    // LDA

    fn lda(&mut self, a: u8) {
        self.a = a;
        self.toggle_nz(a);
    }

    fn lda_imm(&mut self) {
        let a = self.next_byte();
        self.lda(a);
    }

    fn lda_zp(&mut self) {
        let a = self.zero_page_val();
        self.lda(a);
    }

    fn lda_zp_x(&mut self) {
        let a = self.zero_page_x_val();
        self.lda(a);
    }

    fn lda_abs(&mut self) {
        let a = self.absolute_val();
        self.lda(a);
    }

    fn lda_abs_x(&mut self) {
        let a = self.absolute_x_val(true);
        self.lda(a);
    }

    fn lda_abs_y(&mut self) {
        let a = self.absolute_y_val(true);
        self.lda(a);
    }

    fn lda_ind_x(&mut self) {
        let a = self.indirect_x_val();
        self.lda(a);
    }

    fn lda_ind_y(&mut self) {
        let addr = self.indirect_y(true);
        let a = self.bus.read_byte(addr);
        self.lda(a);
    }

    // LDX

    fn ldx(&mut self, x: u8) {
        self.x = x;
        self.toggle_nz(x);
    }

    fn ldx_imm(&mut self) {
        let x = self.next_byte();
        self.ldx(x);
    }

    fn ldx_zp(&mut self) {
        let x = self.zero_page_val();
        self.ldx(x);
    }

    fn ldx_zp_y(&mut self) {
        let x = self.zero_page_y_val();
        self.ldx(x);
    }

    fn ldx_abs(&mut self) {
        let x = self.absolute_val();
        self.ldx(x);
    }

    fn ldx_abs_y(&mut self) {
        let x = self.absolute_y_val(true);
        self.ldx(x);
    }

    // LDY

    fn ldy(&mut self, y: u8) {
        self.y = y;
        self.toggle_nz(y);
    }

    fn ldy_imm(&mut self) {
        let y = self.next_byte();
        self.ldy(y);
    }

    fn ldy_zp(&mut self) {
        let y = self.zero_page_val();
        self.ldy(y);
    }

    fn ldy_zp_x(&mut self) {
        let y = self.zero_page_x_val();
        self.ldy(y);
    }

    fn ldy_abs(&mut self) {
        let y = self.absolute_val();
        self.ldy(y);
    }

    fn ldy_abs_x(&mut self) {
        let y = self.absolute_x_val(true);
        self.ldy(y);
    }

    // STA
    fn sta(&mut self, addr: u16) {
        let a = self.a;
        self.bus.write_byte(addr, a);
    }

    fn sta_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.sta(addr);
    }

    fn sta_zp_x(&mut self) {
        let addr = self.zero_page_x();
        self.sta(addr);
    }

    fn sta_abs(&mut self) {
        let addr = self.absolute();
        self.sta(addr);
    }

    fn sta_abs_x(&mut self) {
        let addr = self.absolute_x(false);
        self.sta(addr);
    }

    fn sta_abs_y(&mut self) {
        let addr = self.absolute_y(false);
        self.sta(addr);
    }

    fn sta_ind_x(&mut self) {
        let addr = self.indirect_x();
        self.sta(addr);
    }

    fn sta_ind_y(&mut self) {
        let addr = self.indirect_y(false);
        self.sta(addr);
    }

    // STX
    fn stx(&mut self, addr: u16) {
        let x = self.x;
        self.bus.write_byte(addr, x);
    }

    fn stx_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.stx(addr);
    }

    fn stx_zp_y(&mut self) {
        let addr = self.zero_page_y();
        self.stx(addr);
    }

    fn stx_abs(&mut self) {
        let addr = self.absolute();
        self.stx(addr);
    }

    // STY
    fn sty(&mut self, addr: u16) {
        let y = self.y;
        self.bus.write_byte(addr, y);
    }

    fn sty_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.sty(addr);
    }

    fn sty_zp_x(&mut self) {
        let addr = self.zero_page_x();
        self.sty(addr);
    }

    fn sty_abs(&mut self) {
        let addr = self.absolute();
        self.sty(addr);
    }

    // ADC

    fn adc(&mut self, val: u8) {
        let sum = (self.a as u16).wrapping_add(val as u16).wrapping_add(
            if self.status.contains(Status::CARRY) {
                1
            } else {
                0
            },
        );

        let carry = sum > 0xff;
        self.status.set(Status::CARRY, carry);

        let sum = sum as u8;
        // http://www.6502.org/tutorials/vflag.html
        let overflow = (val ^ sum) & (sum ^ self.a) & 0x80 != 0;
        self.status.set(Status::OVERFLOW, overflow);

        self.a = sum;
        self.toggle_nz(sum);
    }

    fn adc_imm(&mut self) {
        let val = self.next_byte();
        self.adc(val);
    }

    fn adc_zp(&mut self) {
        let val = self.zero_page_val();
        self.adc(val);
    }

    fn adc_zp_x(&mut self) {
        let val = self.zero_page_x_val();
        self.adc(val);
    }

    fn adc_abs(&mut self) {
        let val = self.absolute_val();
        self.adc(val);
    }

    fn adc_abs_x(&mut self) {
        let val = self.absolute_x_val(true);
        self.adc(val);
    }

    fn adc_abs_y(&mut self) {
        let val = self.absolute_y_val(true);
        self.adc(val);
    }

    fn adc_ind_x(&mut self) {
        let val = self.indirect_x_val();
        self.adc(val);
    }

    fn adc_ind_y(&mut self) {
        let val = self.indirect_y_val(true);
        self.adc(val);
    }

    // SBC - Subtract with Carry

    fn sbc(&mut self, val: u8) {
        let (sum, carried1) = self.a.overflowing_sub(val);
        let (sum, carried2) = sum.overflowing_sub(!self.status.contains(Status::CARRY) as u8);
        let carried = carried1 || carried2;

        self.status.set(
            Status::OVERFLOW,
            (self.a ^ val) & (self.a ^ sum) & 0x80 == 0x80,
        );

        self.a = sum;
        self.status.set(Status::CARRY, !carried);
        self.toggle_nz(sum);
    }

    fn sbc_imm(&mut self) {
        let val = self.next_byte();
        self.sbc(val);
    }

    fn sbc_zp(&mut self) {
        let val = self.zero_page_val();
        self.sbc(val);
    }

    fn sbc_zp_x(&mut self) {
        let val = self.zero_page_x_val();
        self.sbc(val);
    }

    fn sbc_abs(&mut self) {
        let val = self.absolute_val();
        self.sbc(val);
    }

    fn sbc_abs_x(&mut self) {
        let val = self.absolute_x_val(true);
        self.sbc(val);
    }

    fn sbc_abs_y(&mut self) {
        let val = self.absolute_y_val(true);
        self.sbc(val);
    }

    fn sbc_ind_x(&mut self) {
        let val = self.indirect_x_val();
        self.sbc(val);
    }

    fn sbc_ind_y(&mut self) {
        let val = self.indirect_y_val(true);
        self.sbc(val);
    }

    // TAX

    fn tax(&mut self) {
        let x = self.a;
        self.x = x;
        self.toggle_nz(x);
    }

    // TAY

    fn tay(&mut self) {
        let y = self.a;
        self.y = y;
        self.toggle_nz(y);
    }

    // TSX

    fn tsx(&mut self) {
        let x = self.sp;
        self.x = x;
        self.toggle_nz(x);
    }

    // TXA

    fn txa(&mut self) {
        let a = self.x;
        self.a = a;
        self.toggle_nz(a);
    }

    // TXS

    fn txs(&mut self) {
        self.sp = self.x;
    }

    // TYA

    fn tya(&mut self) {
        let a = self.y;
        self.a = a;
        self.toggle_nz(a);
    }

    // AND

    fn and(&mut self, val: u8) {
        let a = self.a & val;
        self.a = a;
        self.toggle_nz(a);
    }

    fn and_imm(&mut self) {
        let val = self.next_byte();
        self.and(val);
    }

    fn and_zp(&mut self) {
        let val = self.zero_page_val();
        self.and(val);
    }

    fn and_zp_x(&mut self) {
        let val = self.zero_page_x_val();
        self.and(val);
    }

    fn and_abs(&mut self) {
        let val = self.absolute_val();
        self.and(val);
    }

    fn and_abs_x(&mut self) {
        let val = self.absolute_x_val(true);
        self.and(val);
    }

    fn and_abs_y(&mut self) {
        let val = self.absolute_y_val(true);
        self.and(val);
    }

    fn and_ind_x(&mut self) {
        let val = self.indirect_x_val();
        self.and(val);
    }

    fn and_ind_y(&mut self) {
        let val = self.indirect_y_val(true);
        self.and(val);
    }

    // ORA - Logical Inclusive OR

    fn ora(&mut self, val: u8) {
        let a = self.a | val;
        self.a = a;
        self.toggle_nz(a);
    }

    fn ora_imm(&mut self) {
        let val = self.next_byte();
        self.ora(val);
    }

    fn ora_zp(&mut self) {
        let val = self.zero_page_val();
        self.ora(val);
    }

    fn ora_zp_x(&mut self) {
        let val = self.zero_page_x_val();
        self.ora(val);
    }

    fn ora_abs(&mut self) {
        let val = self.absolute_val();
        self.ora(val);
    }

    fn ora_abs_x(&mut self) {
        let val = self.absolute_x_val(true);
        self.ora(val);
    }

    fn ora_abs_y(&mut self) {
        let val = self.absolute_y_val(true);
        self.ora(val);
    }

    fn ora_ind_x(&mut self) {
        let val = self.indirect_x_val();
        self.ora(val);
    }

    fn ora_ind_y(&mut self) {
        let val = self.indirect_y_val(true);
        self.ora(val);
    }

    // EOR - Exclusive OR

    fn eor(&mut self, val: u8) {
        let a = self.a ^ val;
        self.a = a;
        self.toggle_nz(a);
    }

    fn eor_imm(&mut self) {
        let val = self.next_byte();
        self.eor(val);
    }

    fn eor_zp(&mut self) {
        let val = self.zero_page_val();
        self.eor(val);
    }

    fn eor_zp_x(&mut self) {
        let val = self.zero_page_x_val();
        self.eor(val);
    }

    fn eor_abs(&mut self) {
        let val = self.absolute_val();
        self.eor(val);
    }

    fn eor_abs_x(&mut self) {
        let val = self.absolute_x_val(true);
        self.eor(val);
    }

    fn eor_abs_y(&mut self) {
        let val = self.absolute_y_val(true);
        self.eor(val);
    }

    fn eor_ind_x(&mut self) {
        let val = self.indirect_x_val();
        self.eor(val);
    }

    fn eor_ind_y(&mut self) {
        let val = self.indirect_y_val(true);
        self.eor(val);
    }

    // ASL - Arithmetic Shift Left

    fn asl(&mut self, addr: u16) {
        let mut val = self.bus.read_byte(addr);
        self.status.set(Status::CARRY, val & 128 == 128);
        val <<= 1;
        self.bus.write_byte(addr, val);
        self.toggle_nz(val);
    }

    fn asl_acc(&mut self) {
        let mut val = self.a;
        self.status.set(Status::CARRY, val & 128 == 128);
        val <<= 1;
        self.a = val;
        self.toggle_nz(val);
    }

    fn asl_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.asl(addr);
    }

    fn asl_zp_x(&mut self) {
        let addr = self.zero_page_x();
        self.asl(addr);
    }

    fn asl_abs(&mut self) {
        let addr = self.absolute();
        self.asl(addr);
    }

    fn asl_abs_x(&mut self) {
        let addr = self.absolute_x(false);
        self.asl(addr);
    }

    // LSR - Logical Shift Right

    fn lsr(&mut self, addr: u16) {
        let val = self.bus.read_byte(addr);
        self.status.set(Status::CARRY, val & 1 == 1);
        let val = val >> 1;
        self.bus.write_byte(addr, val);
        self.toggle_nz(val);
    }

    fn lsr_acc(&mut self) {
        let mut val = self.a;
        self.status.set(Status::CARRY, val & 1 == 1);
        val >>= 1;
        self.a = val;
        self.toggle_nz(val);
    }

    fn lsr_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.lsr(addr);
    }

    fn lsr_zp_x(&mut self) {
        let addr = self.zero_page_x();
        self.lsr(addr);
    }

    fn lsr_abs(&mut self) {
        let addr = self.absolute();
        self.lsr(addr);
    }

    fn lsr_abs_x(&mut self) {
        let addr = self.absolute_x(false);
        self.lsr(addr);
    }

    // INC - Increment Memory

    fn inc(&mut self, addr: u16) {
        let val = self.bus.read_byte(addr);
        let val = val.wrapping_add(1);
        self.bus.write_byte(addr, val);
        self.toggle_nz(val);
    }

    fn inc_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.inc(addr);
    }

    fn inc_zp_x(&mut self) {
        let addr = self.zero_page_x();
        self.inc(addr);
    }

    fn inc_abs(&mut self) {
        let val = self.absolute();
        self.inc(val);
    }

    fn inc_abs_x(&mut self) {
        let addr = self.absolute_x(false);
        self.inc(addr);
    }

    // INX - Increment X Register

    fn inx(&mut self) {
        let val = self.x;
        let val = if val == 0xff { 0 } else { val + 1 };
        self.x = val;
        self.toggle_nz(val);
    }

    // INY - Increment Y Register

    fn iny(&mut self) {
        let val = self.y;
        let val = if val == 0xff { 0 } else { val + 1 };
        self.y = val;
        self.toggle_nz(val);
    }

    // DEC - Decrement Memory

    fn dec(&mut self, addr: u16) {
        let val = self.bus.read_byte(addr);
        let val = if val == 0 { 0xff } else { val - 1 };
        self.bus.write_byte(addr, val);
        self.toggle_nz(val);
    }

    fn dec_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.dec(addr);
    }

    fn dec_zp_x(&mut self) {
        let addr = self.zero_page_x();
        self.dec(addr);
    }

    fn dec_abs(&mut self) {
        let addr = self.absolute();
        self.dec(addr);
    }

    fn dec_abs_x(&mut self) {
        let addr = self.absolute_x(false);
        self.dec(addr);
    }

    // DEX - Decrement X Register

    fn dex(&mut self) {
        let val = self.x;
        let val = if val == 0 { 0xff } else { val - 1 };
        self.x = val;
        self.toggle_nz(val);
    }

    // DEY - Decrement Y Register

    fn dey(&mut self) {
        let val = self.y;
        let val = if val == 0 { 0xff } else { val - 1 };
        self.y = val;
        self.toggle_nz(val);
    }

    // JMP - Jump

    fn jmp(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn jmp_abs(&mut self) {
        let addr = self.absolute();
        self.jmp(addr);
    }

    fn jmp_ind(&mut self) {
        // An original 6502 does not correctly fetch the target address
        // if the indirect vector falls on a page boundary
        // (e.g. $xxFF where xx is any value from $00 to $FF).
        // In this case fetches the LSB from $xxFF as expected but takes the MSB from $xx00.
        // This is fixed in some later chips like the 65SC02 so for compatibility
        // always ensure the indirect vector is not at the end of the page.

        let addr = self.next_word();
        let addr = if addr & 0x00ff == 0xff {
            let lo = self.bus.read_byte(addr);
            let hi = self.bus.read_byte(addr & 0xff00);
            (hi as u16) << 8 | (lo as u16)
        } else {
            self.bus.read_word(addr)
        };

        self.jmp(addr);
    }

    fn branch_rel(&mut self) {
        let rel: i8 = self.next_byte() as i8;
        let jump_addr = self.pc.wrapping_add(rel as u16);
        let prev_pc = self.pc;
        self.pc = jump_addr;
        self.instr_cycles += 1;

        if self.page_crossed(prev_pc, jump_addr) {
            self.instr_cycles += 1;
        }
    }

    // BCS - Branch if Carry Clear

    fn bcc_rel(&mut self) {
        if !self.status.contains(Status::CARRY) {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // BCS - Branch if Carry Set

    fn bcs_rel(&mut self) {
        if self.status.contains(Status::CARRY) {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // BEQ - Branch if Equal

    fn beq_rel(&mut self) {
        if self.status.contains(Status::ZERO) {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // BNE - Branch if Not Equal

    fn bne_rel(&mut self) {
        if !self.status.contains(Status::ZERO) {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // BPL - Branch if Positive

    fn bpl_rel(&mut self) {
        if !self.status.contains(Status::NEGATIVE) {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // BMI - Branch if Minus

    fn bmi_rel(&mut self) {
        if self.status.contains(Status::NEGATIVE) {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // BVC - Branch if Overflow Clear

    fn bvc_rel(&mut self) {
        if !self.status.contains(Status::OVERFLOW) {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // BVS - Branch if Overflow Set

    fn bvs_rel(&mut self) {
        if self.status.contains(Status::OVERFLOW) {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // CLC - Clear Carry Flag

    fn clc(&mut self) {
        self.status.remove(Status::CARRY);
    }

    // SEC - Set Carry Flag

    fn sec(&mut self) {
        self.status.insert(Status::CARRY);
    }

    // CLD - Clear Decimal Flag

    fn cld(&mut self) {
        self.status.remove(Status::DECIMAL);
    }

    // SED - Set Decimal Flag

    fn sed(&mut self) {
        self.status.insert(Status::DECIMAL);
    }

    // CLI - Clear Interrupt Disable

    fn cli(&mut self) {
        self.status.remove(Status::INTERRUPT_DISABLE);
    }

    // SEI - Set Interrupt Disable

    fn sei(&mut self) {
        self.status.insert(Status::INTERRUPT_DISABLE);
    }

    // CLV - Clear Overflow Flag

    fn clv(&mut self) {
        self.status.remove(Status::OVERFLOW);
    }

    fn cmp_vals(&mut self, a: u8, b: u8) {
        self.status.set(Status::CARRY, a >= b);
        let res = a.wrapping_sub(b);
        self.toggle_nz(res);
    }

    // CMP - Compare

    fn cmp(&mut self, val: u8) {
        self.cmp_vals(self.a, val);
    }

    fn cmp_imm(&mut self) {
        let val = self.next_byte();
        self.cmp(val);
    }

    fn cmp_zp(&mut self) {
        let val = self.zero_page_val();
        self.cmp(val);
    }

    fn cmp_zp_x(&mut self) {
        let val = self.zero_page_x_val();
        self.cmp(val);
    }

    fn cmp_abs(&mut self) {
        let val = self.absolute_val();
        self.cmp(val);
    }

    fn cmp_abs_x(&mut self) {
        let val = self.absolute_x_val(true);
        self.cmp(val);
    }

    fn cmp_abs_y(&mut self) {
        let val = self.absolute_y_val(true);
        self.cmp(val);
    }

    fn cmp_ind_x(&mut self) {
        let val = self.indirect_x_val();
        self.cmp(val);
    }

    fn cmp_ind_y(&mut self) {
        let val = self.indirect_y_val(true);
        self.cmp(val);
    }

    // CPX - Compare X Register

    fn cpx(&mut self, val: u8) {
        let x = self.x;
        self.cmp_vals(x, val);
    }

    fn cpx_imm(&mut self) {
        let val = self.next_byte();
        self.cpx(val);
    }

    fn cpx_zp(&mut self) {
        let val = self.zero_page_val();
        self.cpx(val);
    }

    fn cpx_abs(&mut self) {
        let val = self.absolute_val();
        self.cpx(val);
    }

    // CPY - Compare Y Register

    fn cpy(&mut self, val: u8) {
        let y = self.y;
        self.cmp_vals(y, val);
    }

    fn cpy_imm(&mut self) {
        let val = self.next_byte();
        self.cpy(val);
    }

    fn cpy_zp(&mut self) {
        let val = self.zero_page_val();
        self.cpy(val);
    }

    fn cpy_abs(&mut self) {
        let val = self.absolute_val();
        self.cpy(val);
    }

    // PHA - Push Accumulator

    fn pha(&mut self) {
        let a = self.a;
        self.push(a);
    }

    // PLA - Pull Accumulator

    fn pla(&mut self) {
        let a = self.pull();
        self.a = a;
        self.toggle_nz(a);
    }

    // PHP - Push Processor Status

    fn php(&mut self) {
        // set the break flags
        let status_flags = self.status.bits() | Status::BREAK1.bits() | Status::BREAK2.bits();
        self.push(status_flags);
    }

    // PLP - Pull Processor Status
    fn plp(&mut self) {
        let mut flags = self.pull();
        flags &= 0b11101111;
        flags |= 0b00100000;
        self.status.update(flags);
    }

    // JSR - Jump to Subroutine

    fn jsr(&mut self) {
        let ret_addr = self.pc + 1;
        self.push_word(ret_addr);
        let target_addr = self.absolute();
        self.pc = target_addr;
    }

    // RTS - Return from Subroutine

    fn rts(&mut self) {
        self.pc = self.pull_word() + 1;
    }

    // RTI - Return from Interrupt

    fn rti(&mut self) {
        self.plp();
        self.pc = self.pull_word();
    }

    // BIT - Bit Test

    fn bit(&mut self, val: u8) {
        let res = self.a & val;
        self.status.set(Status::ZERO, res == 0);
        self.status.set(Status::OVERFLOW, val & 0x40 != 0);
        self.status.set(Status::NEGATIVE, val & 0x80 != 0);
    }

    fn bit_zp(&mut self) {
        let val = self.zero_page_val();
        self.bit(val);
    }

    fn bit_abs(&mut self) {
        let val = self.absolute_val();
        self.bit(val);
    }

    // ROL - Rotate Left

    fn rol(&mut self, addr: u16) {
        let mut val = self.bus.read_byte(addr);
        let next_carry = (val >> 7) == 1;
        val <<= 1;
        val |= if self.status.contains(Status::CARRY) {
            1
        } else {
            0
        };
        self.status.set(Status::CARRY, next_carry);
        self.bus.write_byte(addr, val);
        self.toggle_nz(val);
    }

    fn rol_acc(&mut self) {
        let mut a = self.a;
        let next_carry = a >> 7 == 1;
        a <<= 1;
        a |= if self.status.contains(Status::CARRY) {
            1
        } else {
            0
        };
        self.status.set(Status::CARRY, next_carry);
        self.a = a;
        self.toggle_nz(a);
    }

    fn rol_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.rol(addr);
    }

    fn rol_zp_x(&mut self) {
        let addr = self.zero_page_x();
        self.rol(addr);
    }

    fn rol_abs(&mut self) {
        let addr = self.absolute();
        self.rol(addr);
    }

    fn rol_abs_x(&mut self) {
        let addr = self.absolute_x(false);
        self.rol(addr);
    }

    // ROR - Rotate Right

    fn ror(&mut self, addr: u16) {
        let mut val = self.bus.read_byte(addr);
        let old_carry = self.status.contains(Status::CARRY);
        self.status.set(Status::CARRY, val & 1 == 1);

        val >>= 1;

        if old_carry {
            val |= 1 << 7;
        }

        self.bus.write_byte(addr, val);
        self.toggle_nz(val);
    }

    fn ror_acc(&mut self) {
        let mut a = self.a;
        let old_carry = self.status.contains(Status::CARRY);
        self.status.set(Status::CARRY, a & 1 == 1);

        a >>= 1;

        if old_carry {
            a |= 1 << 7;
        }

        self.a = a;
        self.toggle_nz(a);
    }

    fn ror_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.ror(addr);
    }

    fn ror_zp_x(&mut self) {
        let addr = self.zero_page_x();
        self.ror(addr);
    }

    fn ror_abs(&mut self) {
        let addr = self.absolute();
        self.ror(addr);
    }

    fn ror_abs_x(&mut self) {
        let addr = self.absolute_x(false);
        self.ror(addr);
    }
}
