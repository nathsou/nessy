use super::Memory;
use cpu::opcodes::{INST_ADDR_MODES, INST_ADDR_MODE_NAMES, INST_CYCLES, INST_LENGTHS, INST_NAMES};
use cpu::MOS6502;
use std::cmp::Ordering;

impl MOS6502 {
    pub fn step(&mut self) -> u16 {
        self.cycles = 0;
        let op_code = self.next_byte();

        match op_code {
            0x00 => self.brk(),
            0xEA => self.nop(),

            0xA9 => self.lda_imm(),
            0xA5 => self.lda_zp(),
            0xB5 => self.lda_zp_x(),
            0xAD => self.lda_abs(),
            0xBD => self.lda_abs_x(),
            0xB9 => self.lda_abs_y(),
            0xA1 => self.lda_ind_x(),
            0xB1 => self.lda_ind_y(),

            0xA2 => self.ldx_imm(),
            0xA6 => self.ldx_zp(),
            0xB6 => self.ldx_zp_y(),
            0xAE => self.ldx_abs(),
            0xBE => self.ldx_abs_y(),

            0xA0 => self.ldy_imm(),
            0xA4 => self.ldy_zp(),
            0xB4 => self.ldy_zp_x(),
            0xAC => self.ldy_abs(),
            0xBC => self.ldy_abs_x(),

            0x85 => self.sta_zp(),
            0x95 => self.sta_zp_x(),
            0x8D => self.sta_abs(),
            0x9D => self.sta_abs_x(),
            0x99 => self.sta_abs_y(),
            0x81 => self.sta_ind_x(),
            0x91 => self.sta_ind_y(),

            0x86 => self.stx_zp(),
            0x96 => self.stx_zp_y(),
            0x8E => self.stx_abs(),

            0x84 => self.sty_zp(),
            0x94 => self.sty_zp_x(),
            0x8C => self.sty_abs(),

            0x69 => self.adc_imm(),
            0x65 => self.adc_zp(),
            0x75 => self.adc_zp_x(),
            0x6D => self.adc_abs(),
            0x7D => self.adc_abs_x(),
            0x79 => self.adc_abs_y(),
            0x61 => self.adc_ind_x(),
            0x71 => self.adc_ind_y(),

            0xE9 => self.sbc_imm(),
            0xE5 => self.sbc_zp(),
            0xF5 => self.sbc_zp_x(),
            0xED => self.sbc_abs(),
            0xFD => self.sbc_abs_x(),
            0xF9 => self.sbc_abs_y(),
            0xE1 => self.sbc_ind_x(),
            0xF1 => self.sbc_ind_y(),

            0xAA => self.tax(),
            0xA8 => self.tay(),
            0xBA => self.tsx(),
            0x8A => self.txa(),
            0x9A => self.txs(),
            0x98 => self.tya(),

            0x29 => self.and_imm(),
            0x25 => self.and_zp(),
            0x35 => self.and_zp_x(),
            0x2D => self.and_abs(),
            0x3D => self.and_abs_x(),
            0x39 => self.and_abs_y(),
            0x21 => self.and_ind_x(),
            0x31 => self.and_ind_y(),

            0x09 => self.ora_imm(),
            0x05 => self.ora_zp(),
            0x15 => self.ora_zp_x(),
            0x0D => self.ora_abs(),
            0x1D => self.ora_abs_x(),
            0x19 => self.ora_abs_y(),
            0x01 => self.ora_ind_x(),
            0x11 => self.ora_ind_y(),

            0x49 => self.eor_imm(),
            0x45 => self.eor_zp(),
            0x55 => self.eor_zp_x(),
            0x4D => self.eor_abs(),
            0x5D => self.eor_abs_x(),
            0x59 => self.eor_abs_y(),
            0x41 => self.eor_ind_x(),
            0x51 => self.eor_ind_y(),

            0x0A => self.asl_acc(),
            0x06 => self.asl_zp(),
            0x16 => self.asl_zp_x(),
            0x0E => self.asl_abs(),
            0x1E => self.asl_abs_x(),

            0x4A => self.lsr_acc(),
            0x46 => self.lsr_zp(),
            0x56 => self.lsr_zp_x(),
            0x4E => self.lsr_abs(),
            0x5E => self.lsr_abs_x(),

            0xE6 => self.inc_zp(),
            0xF6 => self.inc_zp_x(),
            0xEE => self.inc_abs(),
            0xFE => self.inc_abs_x(),

            0xE8 => self.inx(),
            0xC8 => self.iny(),

            0xC6 => self.dec_zp(),
            0xD6 => self.dec_zp_x(),
            0xCE => self.dec_abs(),
            0xDE => self.dec_abs_x(),

            0xCA => self.dex(),
            0x88 => self.dey(),

            0x4C => self.jmp_abs(),
            0x6C => self.jmp_ind(),

            0x90 => self.bcc_rel(),
            0xB0 => self.bcs_rel(),
            0xF0 => self.beq_rel(),
            0xD0 => self.bne_rel(),
            0x10 => self.bpl_rel(),
            0x30 => self.bmi_rel(),
            0x50 => self.bvc_rel(),
            0x70 => self.bvs_rel(),

            0x18 => self.clc(),
            0x38 => self.sec(),
            0xD8 => self.cld(),
            0xF8 => self.sed(),
            0x58 => self.cli(),
            0x78 => self.sei(),
            0xB8 => self.clv(),

            0xC9 => self.cmp_imm(),
            0xC5 => self.cmp_zp(),
            0xD5 => self.cmp_zp_x(),
            0xCD => self.cmp_abs(),
            0xDD => self.cmp_abs_x(),
            0xD9 => self.cmp_abs_y(),
            0xC1 => self.cmp_ind_x(),
            0xD1 => self.cmp_ind_y(),

            0xE0 => self.cpx_imm(),
            0xE4 => self.cpx_zp(),
            0xEC => self.cpx_abs(),

            0xC0 => self.cpy_imm(),
            0xC4 => self.cpy_zp(),
            0xCC => self.cpy_abs(),

            0x48 => self.pha(),
            0x68 => self.pla(),
            0x28 => self.plp(),
            0x08 => self.php(),

            0x20 => self.jsr_abs(),
            0x60 => self.rts(),
            0x40 => self.rti(),

            0x24 => self.bit_zp(),
            0x2C => self.bit_abs(),

            0x2A => self.rol_acc(),
            0x26 => self.rol_zp(),
            0x36 => self.rol_zp_x(),
            0x2E => self.rol_abs(),
            0x3E => self.rol_abs_x(),

            0x6A => self.ror_acc(),
            0x66 => self.ror_zp(),
            0x76 => self.ror_zp_x(),
            0x6E => self.ror_abs(),
            0x7E => self.ror_abs_x(),

            _ => {
                panic!("Unknown opcode: {:#X}", op_code);
            }
        }

        self.cycles + INST_CYCLES[op_code as usize]
    }

    pub fn trace_step(&mut self) -> u16 {
        println!("{:?}", self);
        {
            let op_code = self.read_byte(self.pc) as usize;
            let inst_name = INST_NAMES[op_code].unwrap();
            let addr_mode = INST_ADDR_MODE_NAMES[INST_ADDR_MODES[op_code] as usize];

            let mut opcodes = String::new();

            for pc in
                (self.pc + 1)..(self.pc + (INST_LENGTHS[self.read_byte(self.pc) as usize]) as u16)
            {
                opcodes.push_str(&format!("{:02X} ", self.read_byte(pc))[..]);
            }

            println!(
                "[{:04X}]: {} ({}) {}",
                self.pc, inst_name, addr_mode, opcodes
            );
        }

        self.step()
    }

    // NOP: No Operation
    fn nop(&self) {}

    // LDA
    #[inline]
    fn lda(&mut self, a: u8) {
        self.a = a;
        self.toggle_nz(a);
    }

    #[inline(always)]
    fn lda_imm(&mut self) {
        let a = self.next_byte();
        self.lda(a);
    }

    #[inline(always)]
    fn lda_zp(&mut self) {
        let a = self.zero_page_val();
        self.lda(a);
    }

    #[inline(always)]
    fn lda_zp_x(&mut self) {
        let a = self.zero_page_x();
        self.lda(a);
    }

    #[inline(always)]
    fn lda_abs(&mut self) {
        let a = self.absolute_val();
        self.lda(a);
    }

    #[inline(always)]
    fn lda_abs_x(&mut self) {
        let a = self.absolute_x_val(true);
        self.lda(a);
    }

    #[inline(always)]
    fn lda_abs_y(&mut self) {
        let a = self.absolute_y_val(true);
        self.lda(a);
    }

    #[inline(always)]
    fn lda_ind_x(&mut self) {
        let addr = self.indirect_x();
        let a = self.read_byte(addr);
        self.lda(a);
    }

    #[inline(always)]
    fn lda_ind_y(&mut self) {
        let addr = self.indirect_y(true);
        let a = self.read_byte(addr);
        self.lda(a);
    }

    // LDX
    #[inline]
    fn ldx(&mut self, x: u8) {
        self.x = x;
        self.toggle_nz(x);
    }

    #[inline(always)]
    fn ldx_imm(&mut self) {
        let x = self.next_byte();
        self.ldx(x);
    }

    #[inline(always)]
    fn ldx_zp(&mut self) {
        let x = self.zero_page_val();
        self.ldx(x);
    }

    #[inline(always)]
    fn ldx_zp_y(&mut self) {
        let x = self.zero_page_y();
        self.ldx(x);
    }

    #[inline(always)]
    fn ldx_abs(&mut self) {
        let x = self.absolute_val();
        self.ldx(x);
    }

    #[inline(always)]
    fn ldx_abs_y(&mut self) {
        let x = self.absolute_y_val(true);
        self.ldx(x);
    }

    // LDY
    #[inline]
    fn ldy(&mut self, y: u8) {
        self.y = y;
        self.toggle_nz(y);
    }

    #[inline(always)]
    fn ldy_imm(&mut self) {
        let y = self.next_byte();
        self.ldy(y);
    }

    #[inline(always)]
    fn ldy_zp(&mut self) {
        let y = self.zero_page_val();
        self.ldy(y);
    }

    #[inline(always)]
    fn ldy_zp_x(&mut self) {
        let y = self.zero_page_x();
        self.ldy(y);
    }

    #[inline(always)]
    fn ldy_abs(&mut self) {
        let y = self.absolute_val();
        self.ldy(y);
    }

    #[inline(always)]
    fn ldy_abs_x(&mut self) {
        let y = self.absolute_x_val(true);
        self.lda(y);
    }

    // STA
    fn sta(&mut self, addr: u16) {
        let a = self.a;
        self.write_byte(addr, a);
    }

    #[inline(always)]
    fn sta_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.sta(addr);
    }

    #[inline(always)]
    fn sta_zp_x(&mut self) {
        let addr = self.zero_page_x() as u16;
        self.sta(addr);
    }

    #[inline(always)]
    fn sta_abs(&mut self) {
        let addr = self.absolute();
        self.sta(addr);
    }

    #[inline(always)]
    fn sta_abs_x(&mut self) {
        let addr = self.absolute_x(false);
        self.sta(addr);
    }

    #[inline(always)]
    fn sta_abs_y(&mut self) {
        let addr = self.absolute_y(false);
        self.sta(addr);
    }

    #[inline(always)]
    fn sta_ind_x(&mut self) {
        let addr = self.indirect_x();
        self.sta(addr);
    }

    #[inline(always)]
    fn sta_ind_y(&mut self) {
        let addr = self.indirect_y(false);
        self.sta(addr);
    }

    // STX
    fn stx(&mut self, addr: u16) {
        let x = self.x;
        self.write_byte(addr, x);
    }

    #[inline(always)]
    fn stx_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.stx(addr);
    }

    #[inline(always)]
    fn stx_zp_y(&mut self) {
        let addr = self.zero_page_y() as u16;
        self.stx(addr);
    }

    #[inline(always)]
    fn stx_abs(&mut self) {
        let addr = self.absolute();
        self.stx(addr);
    }

    // STY
    fn sty(&mut self, addr: u16) {
        let y = self.y;
        self.write_byte(addr, y);
    }

    #[inline(always)]
    fn sty_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.sty(addr);
    }

    #[inline(always)]
    fn sty_zp_x(&mut self) {
        let addr = self.zero_page_x() as u16;
        self.sty(addr);
    }

    #[inline(always)]
    fn sty_abs(&mut self) {
        let addr = self.absolute();
        self.sty(addr);
    }

    // ADC
    #[inline(always)]
    fn adc(&mut self, val: u8) {
        let val_carry = if self.carry_flag {
            val.wrapping_add(1)
        } else {
            val
        };

        let (sum, carry) = self.a.overflowing_add(val_carry);

        self.carry_flag = carry;
        // http://www.6502.org/tutorials/vflag.html
        self.overflow_flag = !(self.a ^ val) & (self.a ^ sum) & 0x80 == 0x80;

        self.a = sum;

        self.toggle_nz(sum);
    }

    #[inline(always)]
    fn adc_imm(&mut self) {
        let val = self.next_byte();
        self.adc(val);
    }

    #[inline(always)]
    fn adc_zp(&mut self) {
        let val = self.zero_page_val();
        self.adc(val);
    }

    #[inline(always)]
    fn adc_zp_x(&mut self) {
        let val = self.zero_page_x_val();
        self.adc(val);
    }

    #[inline(always)]
    fn adc_abs(&mut self) {
        let val = self.absolute_val();
        self.adc(val);
    }

    #[inline(always)]
    fn adc_abs_x(&mut self) {
        let val = self.absolute_x_val(true);
        self.adc(val);
    }

    #[inline(always)]
    fn adc_abs_y(&mut self) {
        let val = self.absolute_y_val(true);
        self.adc(val);
    }

    #[inline(always)]
    fn adc_ind_x(&mut self) {
        let val = self.indirect_x_val();
        self.adc(val);
    }

    #[inline(always)]
    fn adc_ind_y(&mut self) {
        let val = self.indirect_y_val(true);
        self.adc(val);
    }

    // SBC - Subtract with Carry
    #[inline(always)]
    fn sbc(&mut self, val: u8) {
        let sub = if !self.carry_flag {
            val.wrapping_add(1)
        } else {
            val
        };

        let (sum, carried) = self.a.overflowing_sub(sub);

        self.overflow_flag = (self.a ^ val) & (self.a ^ sum) & 0x80 == 0x80;

        self.a = sum;
        self.carry_flag = !carried;
        self.toggle_nz(sum);
    }

    #[inline(always)]
    fn sbc_imm(&mut self) {
        let val = self.next_byte();
        self.sbc(val);
    }

    #[inline(always)]
    fn sbc_zp(&mut self) {
        let val = self.zero_page_val();
        self.sbc(val);
    }

    #[inline(always)]
    fn sbc_zp_x(&mut self) {
        let val = self.zero_page_x_val();
        self.sbc(val);
    }

    #[inline(always)]
    fn sbc_abs(&mut self) {
        let val = self.absolute_val();
        self.sbc(val);
    }

    #[inline(always)]
    fn sbc_abs_x(&mut self) {
        let val = self.absolute_x_val(true);
        self.sbc(val);
    }

    #[inline(always)]
    fn sbc_abs_y(&mut self) {
        let val = self.absolute_y_val(true);
        self.sbc(val);
    }

    #[inline(always)]
    fn sbc_ind_x(&mut self) {
        let val = self.indirect_x_val();
        self.sbc(val);
    }

    #[inline(always)]
    fn sbc_ind_y(&mut self) {
        let val = self.indirect_y_val(true);
        self.sbc(val);
    }

    // TAX
    #[inline(always)]
    fn tax(&mut self) {
        let x = self.a;
        self.x = x;
        self.toggle_nz(x);
    }

    // TAY
    #[inline(always)]
    fn tay(&mut self) {
        let y = self.a;
        self.y = y;
        self.toggle_nz(y);
    }

    // TSX
    #[inline(always)]
    fn tsx(&mut self) {
        let x = self.sp;
        self.x = x;
        self.toggle_nz(x);
    }

    // TXA
    #[inline(always)]
    fn txa(&mut self) {
        let a = self.x;
        self.a = a;
        self.toggle_nz(a);
    }

    // TXS
    #[inline(always)]
    fn txs(&mut self) {
        let sp = self.x;
        self.sp = sp;
        self.toggle_nz(sp);
    }

    // TYA
    #[inline(always)]
    fn tya(&mut self) {
        let a = self.y;
        self.a = a;
        self.toggle_nz(a);
    }

    // AND
    #[inline(always)]
    fn and(&mut self, val: u8) {
        let a = self.a & val;
        self.a = a;
        self.toggle_nz(a);
    }

    #[inline(always)]
    fn and_imm(&mut self) {
        let val = self.next_byte();
        self.and(val);
    }

    #[inline(always)]
    fn and_zp(&mut self) {
        let val = self.zero_page_val();
        self.and(val);
    }

    #[inline(always)]
    fn and_zp_x(&mut self) {
        let val = self.zero_page_x_val();
        self.and(val);
    }

    #[inline(always)]
    fn and_abs(&mut self) {
        let val = self.absolute_val();
        self.and(val);
    }

    #[inline(always)]
    fn and_abs_x(&mut self) {
        let val = self.absolute_x_val(true);
        self.and(val);
    }

    #[inline(always)]
    fn and_abs_y(&mut self) {
        let val = self.absolute_y_val(true);
        self.and(val);
    }

    #[inline(always)]
    fn and_ind_x(&mut self) {
        let val = self.indirect_x_val();
        self.and(val);
    }

    #[inline(always)]
    fn and_ind_y(&mut self) {
        let val = self.indirect_y_val(true);
        self.and(val);
    }

    // ORA - Logical Inclusive OR
    #[inline(always)]
    fn ora(&mut self, val: u8) {
        let a = self.a | val;
        self.a = a;
        self.toggle_nz(a);
    }

    #[inline(always)]
    fn ora_imm(&mut self) {
        let val = self.next_byte();
        self.ora(val);
    }

    #[inline(always)]
    fn ora_zp(&mut self) {
        let val = self.zero_page_val();
        self.ora(val);
    }

    #[inline(always)]
    fn ora_zp_x(&mut self) {
        let val = self.zero_page_x_val();
        self.ora(val);
    }

    #[inline(always)]
    fn ora_abs(&mut self) {
        let val = self.absolute_val();
        self.ora(val);
    }

    #[inline(always)]
    fn ora_abs_x(&mut self) {
        let val = self.absolute_x_val(true);
        self.ora(val);
    }

    #[inline(always)]
    fn ora_abs_y(&mut self) {
        let val = self.absolute_y_val(true);
        self.ora(val);
    }

    #[inline(always)]
    fn ora_ind_x(&mut self) {
        let val = self.indirect_x_val();
        self.ora(val);
    }

    #[inline(always)]
    fn ora_ind_y(&mut self) {
        let val = self.indirect_y_val(true);
        self.ora(val);
    }

    // EOR - Exclusive OR
    #[inline(always)]
    fn eor(&mut self, val: u8) {
        let a = self.a ^ val;
        self.a = a;
        self.toggle_nz(a);
    }

    #[inline(always)]
    fn eor_imm(&mut self) {
        let val = self.next_byte();
        self.eor(val);
    }

    #[inline(always)]
    fn eor_zp(&mut self) {
        let val = self.zero_page_val();
        self.eor(val);
    }

    #[inline(always)]
    fn eor_zp_x(&mut self) {
        let val = self.zero_page_x_val();
        self.eor(val);
    }

    #[inline(always)]
    fn eor_abs(&mut self) {
        let val = self.absolute_val();
        self.eor(val);
    }

    #[inline(always)]
    fn eor_abs_x(&mut self) {
        let val = self.absolute_x_val(true);
        self.eor(val);
    }

    #[inline(always)]
    fn eor_abs_y(&mut self) {
        let val = self.absolute_y_val(true);
        self.eor(val);
    }

    #[inline(always)]
    fn eor_ind_x(&mut self) {
        let val = self.indirect_x_val();
        self.eor(val);
    }

    #[inline(always)]
    fn eor_ind_y(&mut self) {
        let val = self.indirect_y_val(true);
        self.eor(val);
    }

    // ASL - Arithmetic Shift Left
    #[inline(always)]
    fn asl(&mut self, addr: u16) {
        let mut val = self.read_byte(addr);
        self.carry_flag = val & 128 == 128;
        val <<= 1;
        self.write_byte(addr, val);
        self.toggle_nz(val);
    }

    #[inline(always)]
    fn asl_acc(&mut self) {
        let mut val = self.a;
        self.carry_flag = val & 128 == 128;
        val <<= 1;
        self.a = val;
        self.toggle_nz(val);
    }

    #[inline(always)]
    fn asl_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.asl(addr);
    }

    #[inline(always)]
    fn asl_zp_x(&mut self) {
        let addr = self.zero_page_x() as u16;
        self.asl(addr);
    }

    #[inline(always)]
    fn asl_abs(&mut self) {
        let addr = self.absolute();
        self.asl(addr);
    }

    #[inline(always)]
    fn asl_abs_x(&mut self) {
        let addr = self.absolute_x(false);
        self.asl(addr);
    }

    // LSR - Logical Shift Right
    #[inline(always)]
    fn lsr(&mut self, addr: u16) {
        let val = self.read_byte(addr);
        self.carry_flag = val & 1 == 1;
        let val = val >> 1;
        self.write_byte(addr, val);
        self.toggle_nz(val);
    }

    #[inline(always)]
    fn lsr_acc(&mut self) {
        let mut val = self.a;
        self.carry_flag = val & 1 == 1;
        val >>= 1;
        self.a = val;
        self.toggle_nz(val);
    }

    #[inline(always)]
    fn lsr_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.lsr(addr);
    }

    #[inline(always)]
    fn lsr_zp_x(&mut self) {
        let addr = self.zero_page_x() as u16;
        self.lsr(addr);
    }

    #[inline(always)]
    fn lsr_abs(&mut self) {
        let addr = self.absolute();
        self.lsr(addr);
    }

    #[inline(always)]
    fn lsr_abs_x(&mut self) {
        let addr = self.absolute_x(false);
        self.lsr(addr);
    }

    // INC - Increment Memory
    #[inline(always)]
    fn inc(&mut self, addr: u16) {
        let val = self.read_byte(addr);
        let val = if val == 0xff { 0 } else { val + 1 };
        self.write_byte(addr, val);
        self.toggle_nz(val);
    }

    #[inline(always)]
    fn inc_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.inc(addr);
    }

    #[inline(always)]
    fn inc_zp_x(&mut self) {
        let addr = self.zero_page_x() as u16;
        self.inc(addr);
    }

    #[inline(always)]
    fn inc_abs(&mut self) {
        let addr = self.absolute();
        self.inc(addr);
    }

    #[inline(always)]
    fn inc_abs_x(&mut self) {
        let addr = self.absolute_x(false);
        self.inc(addr);
    }

    // INX - Increment X Register
    #[inline(always)]
    fn inx(&mut self) {
        let val = self.x;
        let val = if val == 0xff { 0 } else { val + 1 };
        self.x = val;
        self.toggle_nz(val);
    }

    // INY - Increment Y Register
    #[inline(always)]
    fn iny(&mut self) {
        let val = self.y;
        let val = if val == 0xff { 0 } else { val + 1 };
        self.y = val;
        self.toggle_nz(val);
    }

    // DEC - Decrement Memory
    #[inline(always)]
    fn dec(&mut self, addr: u16) {
        let val = self.read_byte(addr);
        let val = if val == 0 { 0xff } else { val - 1 };
        self.write_byte(addr, val);
        self.toggle_nz(val);
    }

    #[inline(always)]
    fn dec_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.dec(addr);
    }

    #[inline(always)]
    fn dec_zp_x(&mut self) {
        let addr = self.zero_page_x() as u16;
        self.dec(addr);
    }

    #[inline(always)]
    fn dec_abs(&mut self) {
        let addr = self.absolute();
        self.dec(addr);
    }

    #[inline(always)]
    fn dec_abs_x(&mut self) {
        let addr = self.absolute_x(false);
        self.dec(addr);
    }

    // DEX - Decrement X Register
    #[inline(always)]
    fn dex(&mut self) {
        let val = self.x;
        let val = if val == 0 { 0xff } else { val - 1 };
        self.x = val;
        self.toggle_nz(val);
    }

    // DEY - Decrement Y Register
    #[inline(always)]
    fn dey(&mut self) {
        let val = self.y;
        let val = if val == 0 { 0xff } else { val - 1 };
        self.y = val;
        self.toggle_nz(val);
    }

    // JMP - Jump
    #[inline(always)]
    fn jmp(&mut self, addr: u16) {
        self.pc = addr;
    }

    #[inline(always)]
    fn jmp_abs(&mut self) {
        let addr = self.absolute();
        self.jmp(addr);
    }

    #[inline(always)]
    fn jmp_ind(&mut self) {
        let addr = self.indirect();
        self.jmp(addr);
    }

    #[inline(always)]
    fn branch_rel(&mut self) {
        let mut rel = self.next_byte() as i32;
        if rel > 127 {
            rel = -(0x100 - rel);
        }

        let pc = self.pc as i32;
        self.pc = (pc + rel) as u16;
        self.cycles += 1;

        let new_page_check = (pc & 0xff) + rel;
        if new_page_check > 0xff || new_page_check < 0 {
            self.cycles += 1;
        }
    }

    // BCS - Branch if Carry Clear
    #[inline(always)]
    fn bcc_rel(&mut self) {
        if !self.carry_flag {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // BCS - Branch if Carry Set
    #[inline(always)]
    fn bcs_rel(&mut self) {
        if self.carry_flag {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // BEQ - Branch if Equal
    #[inline(always)]
    fn beq_rel(&mut self) {
        if self.zero_flag {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // BNE - Branch if Not Equal
    #[inline(always)]
    fn bne_rel(&mut self) {
        if !self.zero_flag {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // BPL - Branch if Positive
    #[inline(always)]
    fn bpl_rel(&mut self) {
        if !self.negative_flag {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // BMI - Branch if Minus
    #[inline(always)]
    fn bmi_rel(&mut self) {
        if self.negative_flag {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // BVC - Branch if Overflow Clear
    #[inline(always)]
    fn bvc_rel(&mut self) {
        if !self.overflow_flag {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // BVS - Branch if Overflow Set
    #[inline(always)]
    fn bvs_rel(&mut self) {
        if !self.overflow_flag {
            self.branch_rel();
        } else {
            self.pc += 1;
        }
    }

    // CLC - Clear Carry Flag
    #[inline(always)]
    fn clc(&mut self) {
        self.carry_flag = false;
    }

    // SEC - Set Carry Flag
    #[inline(always)]
    fn sec(&mut self) {
        self.carry_flag = true;
    }

    // CLD - Clear Decimal Flag
    #[inline(always)]
    fn cld(&mut self) {
        self.dec_mode_flag = false;
    }

    // SED - Set Decimal Flag
    #[inline(always)]
    fn sed(&mut self) {
        self.dec_mode_flag = true;
    }

    // CLI - Clear Interrupt Disable
    #[inline(always)]
    fn cli(&mut self) {
        self.interrupt_disable_flag = false;
    }

    // SEI - Set Interrupt Disable
    #[inline(always)]
    fn sei(&mut self) {
        self.interrupt_disable_flag = true;
    }

    // CLV - Clear Overflow Flag
    #[inline(always)]
    fn clv(&mut self) {
        self.overflow_flag = false;
    }

    #[inline]
    fn cmp_vals(&mut self, a: u8, b: u8) {
        match a.cmp(&b) {
            Ordering::Equal => {
                self.zero_flag = true;
                self.negative_flag = false;
                self.carry_flag = true;
            }
            Ordering::Greater => {
                self.zero_flag = false;
                self.negative_flag = false;
                self.overflow_flag = true;
            }
            Ordering::Less => {
                self.zero_flag = false;
                self.negative_flag = true;
                self.carry_flag = false;
            }
        }
    }

    // CMP - Compare
    #[inline(always)]
    fn cmp(&mut self, val: u8) {
        // let diff = ((self.a as i16) - (val as i16));
        // if diff >= 0 {
        //     self.overflow_flag = true;
        // }
        // self.toggle_nz((diff & 0xff) as u8);
        let a = self.a;
        self.cmp_vals(a, val);
    }

    #[inline(always)]
    fn cmp_imm(&mut self) {
        let val = self.next_byte();
        self.cmp(val);
    }

    #[inline(always)]
    fn cmp_zp(&mut self) {
        let val = self.zero_page_val();
        self.cmp(val);
    }

    #[inline(always)]
    fn cmp_zp_x(&mut self) {
        let val = self.zero_page_x_val();
        self.cmp(val);
    }

    #[inline(always)]
    fn cmp_abs(&mut self) {
        let val = self.absolute_val();
        self.cmp(val);
    }

    #[inline(always)]
    fn cmp_abs_x(&mut self) {
        let val = self.absolute_x_val(true);
        self.cmp(val);
    }

    #[inline(always)]
    fn cmp_abs_y(&mut self) {
        let val = self.absolute_y_val(true);
        self.cmp(val);
    }

    #[inline(always)]
    fn cmp_ind_x(&mut self) {
        let val = self.indirect_x_val();
        self.cmp(val);
    }

    #[inline(always)]
    fn cmp_ind_y(&mut self) {
        let val = self.indirect_y_val(true);
        self.cmp(val);
    }

    // CPX - Compare X Register
    #[inline(always)]
    fn cpx(&mut self, val: u8) {
        let x = self.x;
        self.cmp_vals(x, val);
    }

    #[inline(always)]
    fn cpx_imm(&mut self) {
        let val = self.next_byte();
        self.cpx(val);
    }

    #[inline(always)]
    fn cpx_zp(&mut self) {
        let val = self.zero_page_val();
        self.cpx(val);
    }

    #[inline(always)]
    fn cpx_abs(&mut self) {
        let val = self.absolute_val();
        self.cpx(val);
    }

    // CPY - Compare Y Register
    #[inline(always)]
    fn cpy(&mut self, val: u8) {
        let y = self.y;
        self.cmp_vals(y, val);
    }

    #[inline(always)]
    fn cpy_imm(&mut self) {
        let val = self.next_byte();
        self.cpy(val);
    }

    #[inline(always)]
    fn cpy_zp(&mut self) {
        let val = self.zero_page_val();
        self.cpy(val);
    }

    #[inline(always)]
    fn cpy_abs(&mut self) {
        let val = self.absolute_val();
        self.cpy(val);
    }

    // PHA - Push Accumulator
    #[inline(always)]
    fn pha(&mut self) {
        let a = self.a;
        self.push(a);
    }

    // PLA - Pull Accumulator
    #[inline(always)]
    fn pla(&mut self) {
        let a = self.pull();
        self.a = a;
        self.toggle_nz(a);
    }

    // PHP - Push Processor Status
    #[inline(always)]
    fn php(&mut self) {
        let status_flags = self.flags_to_u8();
        self.push(status_flags);
    }

    // PLP - Pull Processor Status
    fn plp(&mut self) {
        let flags = self.pull();
        self.set_flags_from_u8(flags);
    }

    // BRK - Force Interrupt
    #[inline(always)]
    fn brk(&mut self) {
        let pc_high = (self.pc >> 8) as u8;
        let pc_low = (self.pc & 0xff) as u8;

        // push the program counter and status flags to the stack
        self.push(pc_high);
        self.push(pc_low);
        self.php();

        // load the IRQ interrupt vector into the PC
        let addr = self.read_word(0xfffe);
        let addr = self.read_word(addr);
        self.pc = addr;

        // set the B flag to 1
        self.break_command_flag = true;
    }

    // JSR - Jump to Subroutine
    #[inline(always)]
    fn jsr_abs(&mut self) {
        let ret_addr = self.pc + 1;
        let ret_high = (ret_addr >> 8) as u8;
        let ret_low = (ret_addr & 0xff) as u8;

        self.push(ret_high);
        self.push(ret_low);

        let target_addr = self.absolute();
        self.pc = target_addr;
    }

    // RTS - Return from Subroutine
    #[inline(always)]
    fn rts(&mut self) {
        self.pc = self.pull_word() + 1;
    }

    // RTI - Return from Interrupt
    #[inline(always)]
    fn rti(&mut self) {
        self.plp();
        self.rts();
    }

    // BIT - Bit Test
    #[inline(always)]
    fn bit(&mut self, val: u8) {
        let res = self.a & val;
        self.zero_flag = res == 0;
        self.overflow_flag = val & 0x40 != 0;
        self.negative_flag = val & 0x80 != 0;
    }

    #[inline(always)]
    fn bit_zp(&mut self) {
        let val = self.zero_page_val();
        self.bit(val);
    }

    #[inline(always)]
    fn bit_abs(&mut self) {
        let val = self.absolute_val();
        self.bit(val);
    }

    // ROL - Rotate Left
    #[inline(always)]
    fn rol(&mut self, addr: u16) {
        let mut val = self.read_byte(addr);
        let next_carry = (val >> 7) == 1;
        val <<= 1;
        val |= if self.carry_flag { 1 } else { 0 };
        self.carry_flag = next_carry;
        self.write_byte(addr, val);
    }

    #[inline(always)]
    fn rol_acc(&mut self) {
        let mut a = self.a;
        let next_carry = (a >> 7) == 1;
        a <<= 1;
        a |= if self.carry_flag { 1 } else { 0 };
        self.carry_flag = next_carry;
        self.a = a;
    }

    #[inline(always)]
    fn rol_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.rol(addr);
    }

    #[inline(always)]
    fn rol_zp_x(&mut self) {
        let addr = self.zero_page_x() as u16;
        self.rol(addr);
    }

    #[inline(always)]
    fn rol_abs(&mut self) {
        let addr = self.absolute();
        self.rol(addr);
    }

    #[inline(always)]
    fn rol_abs_x(&mut self) {
        let addr = self.absolute_x(false);
        self.rol(addr);
    }

    // ROR - Rotate Right
    #[inline(always)]
    fn ror(&mut self, addr: u16) {
        let mut val = self.read_byte(addr);
        let next_carry = (val & 1) == 1;
        val >>= 1;
        if self.carry_flag {
            val |= 128;
        }
        self.carry_flag = next_carry;
        self.write_byte(addr, val);
    }

    #[inline(always)]
    fn ror_acc(&mut self) {
        let mut a = self.a;
        let next_carry = (a & 1) == 1;
        a >>= 1;
        if self.carry_flag {
            a |= 128;
        };
        self.carry_flag = next_carry;
        self.a = a;
    }

    #[inline(always)]
    fn ror_zp(&mut self) {
        let addr = self.zero_page() as u16;
        self.ror(addr);
    }

    #[inline(always)]
    fn ror_zp_x(&mut self) {
        let addr = self.zero_page_x() as u16;
        self.ror(addr);
    }

    #[inline(always)]
    fn ror_abs(&mut self) {
        let addr = self.absolute();
        self.ror(addr);
    }

    #[inline(always)]
    fn ror_abs_x(&mut self) {
        let addr = self.absolute_x(false);
        self.ror(addr);
    }
}
