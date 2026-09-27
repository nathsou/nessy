//! Hardware regressions and bundled CPU conformance checks.
use nessy::{
    apu::APU,
    bus::Bus,
    cpu::{
        memory::Memory,
        rom::{Mirroring, ROM},
        CPU,
    },
    ppu::PPU,
    savestate::{Save, SaveState, Section},
};

fn rom(mapper: u8, prg: u8, chr: u8) -> ROM {
    let mut bytes = vec![0; 16 + prg as usize * 0x4000 + chr as usize * 0x2000];
    bytes[..4].copy_from_slice(b"NES\x1a");
    bytes[4] = prg;
    bytes[5] = chr;
    bytes[6] = mapper << 4;
    for bank in 0..prg as usize * 2 {
        bytes[16 + bank * 0x2000..16 + (bank + 1) * 0x2000].fill(bank as u8);
    }
    for bank in 0..chr as usize * 8 {
        let base = 16 + prg as usize * 0x4000 + bank * 0x400;
        bytes[base..base + 0x400].fill(bank as u8);
    }
    ROM::new(bytes).unwrap()
}

fn cpu(program: &[u8]) -> CPU {
    let mut rom = rom(4, 4, 1);
    let start = rom.cart.prg_rom_start;
    // All interrupt handlers execute NOP; vectors point to $E000.
    rom.cart.bytes[start + 0xe000] = 0xea;
    for offset in [0xfffa, 0xfffc, 0xfffe] {
        rom.cart.bytes[start + offset..start + offset + 2].copy_from_slice(&[0, 0xe0]);
    }
    let mut cpu = CPU::new(Bus::new(rom, 44100.0));
    cpu.pc = 0x200;
    for (i, byte) in program.iter().enumerate() {
        cpu.bus.write_byte(0x200 + i as u16, *byte);
    }
    cpu
}

fn reg(cpu: &CPU, name: &str) -> u8 {
    let debug = format!("{cpu:?}");
    let value = debug
        .split_whitespace()
        .find_map(|item| item.strip_prefix(name))
        .unwrap();
    u8::from_str_radix(value, 16).unwrap()
}

fn mmc_write(rom: &mut ROM, addr: u16, val: u8) {
    rom.mapper.write(&mut rom.cart, addr, val);
}
fn mmc_read(rom: &mut ROM, addr: u16) -> u8 {
    rom.mapper.read(&mut rom.cart, addr)
}
fn irq(cpu: &mut CPU) {
    mmc_write(&mut cpu.bus.ppu.rom, 0xc000, 0);
    mmc_write(&mut cpu.bus.ppu.rom, 0xe001, 0);
    cpu.bus.ppu.rom.mapper.step_scanline();
}
fn address(ppu: &mut PPU, addr: u16) {
    ppu.read_register(0x2002);
    ppu.write_register(0x2006, (addr >> 8) as u8);
    ppu.write_register(0x2006, addr as u8);
}

fn trace(include_unofficial: bool) {
    let rom = ROM::new(include_bytes!("../src/tests/nestest.nes").to_vec()).unwrap();
    let mut cpu = CPU::new(Bus::new(rom, 44100.0));
    cpu.pc = 0xc000;
    let mut cycles = 7;
    let mut count = 0;
    for (i, line) in include_str!("../src/tests/nestest.log").lines().enumerate() {
        if !include_unofficial && line.contains('*') {
            break;
        }
        assert_eq!(
            format!("{:04X}", cpu.pc),
            &line[..4],
            "trace line {}",
            i + 1
        );
        assert_eq!(format!("{cpu:?}"), &line[48..73], "trace line {}", i + 1);
        assert_eq!(
            cycles,
            line.split("CYC:").nth(1).unwrap().parse::<u32>().unwrap(),
            "trace line {}",
            i + 1
        );
        cycles += cpu.step();
        count += 1;
    }
    if !include_unofficial {
        assert_eq!(count, 5003);
    }
}

#[test]
fn nestest_official_trace() {
    trace(false);
}

#[test]
fn exhaustive_adc_sbc_binary_flags() {
    let mut c = cpu(&[]);
    for op in [0x69, 0xe9] {
        for a in 0u16..256 {
            for b in 0u16..256 {
                for carry in 0u16..2 {
                    c.pc = 0x200;
                    // SED verifies that the NES still performs binary arithmetic.
                    for (i, byte) in [
                        0xf8,
                        if carry == 1 { 0x38 } else { 0x18 },
                        0xa9,
                        a as u8,
                        op,
                        b as u8,
                    ]
                    .iter()
                    .enumerate()
                    {
                        c.bus.write_byte(0x200 + i as u16, *byte);
                    }
                    for _ in 0..4 {
                        assert_eq!(c.step(), 2);
                    }
                    let (wide, overflow) = if op == 0x69 {
                        let wide = a + b + carry;
                        (wide as i32, (!(a ^ b) & (a ^ wide) & 128) != 0)
                    } else {
                        let wide = a as i32 - b as i32 - (1 - carry as i32);
                        (wide, ((a ^ b) & (a ^ wide as u16) & 128) != 0)
                    };
                    let result = wide as u8;
                    let carry_out = if op == 0x69 { wide > 255 } else { wide >= 0 };
                    let flags = carry_out as u8
                        | ((result == 0) as u8) << 1
                        | (overflow as u8) << 6
                        | (result & 128);
                    assert_eq!(reg(&c, "A:"), result, "op={op:02x} a={a} b={b} c={carry}");
                    assert_eq!(reg(&c, "P:") & 0xc3, flags);
                }
            }
        }
    }
}

#[test]
fn mmc3_bank_data_and_counter_baseline() {
    let mut r = rom(4, 4, 2);
    mmc_write(&mut r, 0x8000, 6);
    mmc_write(&mut r, 0x8001, 3);
    assert_eq!(mmc_read(&mut r, 0x8000), 3);
    assert_eq!(mmc_read(&mut r, 0xc000), 6);
    assert_eq!(mmc_read(&mut r, 0xe000), 7);
    mmc_write(&mut r, 0xc000, 2);
    mmc_write(&mut r, 0xc001, 0);
    mmc_write(&mut r, 0xe001, 0);
    for _ in 0..2 {
        r.mapper.step_scanline();
        assert!(!r.mapper.is_asserting_irq());
    }
    r.mapper.step_scanline();
    assert!(r.mapper.is_asserting_irq());
    mmc_write(&mut r, 0xe000, 0);
    assert!(!r.mapper.is_asserting_irq());
}

#[test]
fn nestest_full_trace() {
    trace(true);
}

#[test]
fn brk_return_address() {
    let mut c = cpu(&[0x00, 0xea]);
    assert_eq!(c.step(), 7);
    assert_eq!(c.bus.read_byte(0x1fc), 0x02);
}

#[test]
fn irq_stacks_break_clear() {
    let mut c = cpu(&[0x58, 0xea]);
    c.step();
    irq(&mut c);
    c.step();
    assert_eq!(c.step(), 7);
    assert_eq!(c.bus.read_byte(0x1fb) & 0x30, 0x20);
}

#[test]
fn nmi_stacks_break_clear() {
    let mut c = cpu(&[0xea]);
    c.bus.ppu.write_register(0x2000, 0x80);
    for _ in 0..3 {
        c.bus.ppu.step();
    }
    c.step();
    assert_eq!(c.step(), 7);
    assert_eq!(c.bus.read_byte(0x1fb) & 0x30, 0x20);
}

#[test]
fn mmc3_irq_is_level() {
    let mut c = cpu(&[]);
    irq(&mut c);
    assert!(c.bus.ppu.rom.mapper.is_asserting_irq());
    assert!(c.bus.ppu.rom.mapper.is_asserting_irq());
}

#[test]
fn masked_mmc3_irq_survives_until_cli() {
    let mut c = cpu(&[0x78, 0x58, 0xea, 0xea]);
    irq(&mut c);
    for _ in 0..4 {
        c.step();
    }
    assert_eq!(c.pc, 0xe000);
}

#[test]
fn cli_defers_irq_for_one_instruction() {
    let mut c = cpu(&[0x58, 0xea, 0xea]);
    c.step();
    irq(&mut c);
    c.step();
    assert_eq!(c.pc, 0x202);
}

#[test]
fn rmw_oamdata_writes_old_then_new() {
    let mut c = cpu(&[0xee, 0x04, 0x20]); // INC $2004
    c.bus.write_byte(0x2003, 0);
    c.bus.write_byte(0x2004, 0x40);
    c.bus.write_byte(0x2003, 0);
    c.step();
    c.bus.write_byte(0x2003, 0);
    assert_eq!(c.bus.read_byte(0x2004), 0x40);
    c.bus.write_byte(0x2003, 1);
    assert_eq!(c.bus.read_byte(0x2004), 0x41);
}

#[test]
fn rts_wraps_address_space() {
    let mut c = cpu(&[0xa9, 0xff, 0x48, 0x48, 0x60]);
    for _ in 0..4 {
        c.step();
    }
    assert_eq!(c.pc, 0);
}

#[test]
fn consecutive_dma_parity() {
    let mut c = cpu(&[0x24, 0x00, 0xea, 0xea]);
    assert_eq!(c.step(), 3);
    c.bus.dma_transfer = true;
    assert_eq!(c.step(), 513);
    assert_eq!(c.step(), 2);
    c.bus.dma_transfer = true;
    assert_eq!(c.step(), 514);
}

#[test]
fn mmc3_prg_mode_is_immediate() {
    let mut r = rom(4, 4, 2);
    mmc_write(&mut r, 0x8000, 6);
    mmc_write(&mut r, 0x8001, 3);
    mmc_write(&mut r, 0x8000, 0x46);
    assert_eq!(mmc_read(&mut r, 0x8000), 6);
    assert_eq!(mmc_read(&mut r, 0xc000), 3);
}

#[test]
fn mmc3_chr_mode_is_immediate() {
    let mut r = rom(4, 4, 2);
    mmc_write(&mut r, 0x8000, 2);
    mmc_write(&mut r, 0x8001, 5);
    mmc_write(&mut r, 0x8000, 0x82);
    assert_eq!(mmc_read(&mut r, 0), 5);
}

#[test]
fn mmc3_chr_banks_wrap() {
    let mut r = rom(4, 4, 1);
    mmc_write(&mut r, 0x8000, 2);
    mmc_write(&mut r, 0x8001, 0xff);
    assert_eq!(mmc_read(&mut r, 0x1000), 7);
}

#[test]
fn mmc3_chr_ram() {
    let mut r = rom(4, 4, 0);
    mmc_write(&mut r, 0, 0x5a);
    assert_eq!(mmc_read(&mut r, 0), 0x5a);
}

#[test]
fn mmc3_prg_ram_write_protect() {
    let mut r = rom(4, 4, 1);
    mmc_write(&mut r, 0xa001, 0x80);
    mmc_write(&mut r, 0x6000, 0x12);
    mmc_write(&mut r, 0xa001, 0xc0);
    mmc_write(&mut r, 0x6000, 0x34);
    assert_eq!(mmc_read(&mut r, 0x6000), 0x12);
}

#[test]
fn mmc3_four_screen_is_hardwired() {
    let mut r = rom(4, 4, 1);
    r.cart.mirroring = Mirroring::FourScreen;
    mmc_write(&mut r, 0xa000, 1);
    assert_eq!(r.cart.mirroring, Mirroring::FourScreen);
}

#[test]
fn mmc3_no_irq_when_a12_stays_low() {
    let mut p = PPU::new(rom(4, 4, 1));
    p.write_register(0x2000, 0); // 8x8 sprites and background both use $0000
    p.write_register(0x2001, 0x18);
    mmc_write(&mut p.rom, 0xc000, 0);
    mmc_write(&mut p.rom, 0xe001, 0);
    for _ in 0..(262 * 341) {
        p.step();
        assert!(!p.rom.mapper.is_asserting_irq());
    }
}

#[test]
fn savestate_restores_mmc3_mirroring() {
    let mut c = cpu(&[]);
    mmc_write(&mut c.bus.ppu.rom, 0xa000, 0);
    let mut saved = Section::new("root");
    c.save(&mut saved);
    mmc_write(&mut c.bus.ppu.rom, 0xa000, 1);
    c.load(&mut saved).unwrap();
    assert_eq!(c.bus.ppu.rom.cart.mirroring, Mirroring::Vertical);
}

#[test]
fn savestate_restores_apu_status() {
    let mut c = cpu(&[]);
    c.bus.apu.write(0x4015, 1);
    c.bus.apu.write(0x4003, 8);
    let mut saved = Section::new("root");
    c.save(&mut saved);
    c.bus.apu.write(0x4015, 0);
    c.load(&mut saved).unwrap();
    assert_eq!(c.bus.apu.read(0x4015) & 1, 1);
}

#[test]
fn forced_blank_outputs_backdrop() {
    let mut p = PPU::new(rom(0, 1, 1));
    address(&mut p, 0x3f00);
    p.write_data_reg(0x21);
    address(&mut p, 0x2000);
    for _ in 0..(262 * 341 + 2) {
        p.step();
    }
    let rgb = nessy::ppu::COLOR_PALETTE[0x21];
    assert_eq!(&p.get_frame()[..3], &[rgb.0, rgb.1, rgb.2]);
}

#[test]
fn ppu_3000_mirror_writes() {
    let mut p = PPU::new(rom(0, 1, 1));
    address(&mut p, 0x3000);
    p.write_data_reg(0x42);
    address(&mut p, 0x2000);
    p.read_data_reg();
    assert_eq!(p.read_data_reg(), 0x42);
}

#[test]
fn palette_3f30_aliases_3f00() {
    let mut p = PPU::new(rom(0, 1, 1));
    address(&mut p, 0x3f30);
    p.write_data_reg(0x21);
    address(&mut p, 0x3f00);
    assert_eq!(p.read_data_reg(), 0x21);
}

#[test]
fn palette_read_refills_buffer() {
    let mut p = PPU::new(rom(0, 1, 1));
    address(&mut p, 0x2f00);
    p.write_data_reg(0x42);
    address(&mut p, 0x3f00);
    p.read_data_reg();
    address(&mut p, 0x2000);
    assert_eq!(p.read_data_reg(), 0x42);
}

#[test]
fn four_screen_nametable_capacity() {
    let mut r = rom(4, 4, 1);
    r.cart.mirroring = Mirroring::FourScreen;
    let mut p = PPU::new(r);
    address(&mut p, 0x2800);
    p.write_data_reg(0x42);
}

#[test]
fn apu_irq_is_level() {
    let mut a = APU::new(44100.0);
    for _ in 0..29830 {
        a.step();
    }
    assert!(a.is_asserting_irq());
    assert!(a.is_asserting_irq());
}

#[test]
fn five_step_has_no_frame_irq() {
    let mut a = APU::new(44100.0);
    a.write(0x4017, 0xc0);
    for _ in 0..40000 {
        a.step();
    }
    assert_eq!(a.read(0x4015) & 0x40, 0);
}

#[test]
fn dmc_completion_irq() {
    let mut a = APU::new(44100.0);
    a.write(0x4017, 0x40);
    a.write(0x4010, 0x8f);
    a.write(0x4012, 0);
    a.write(0x4013, 0);
    a.write(0x4015, 0x10);
    for _ in 0..100 {
        a.step();
        if a.pull_memory_read_request().is_some() {
            a.push_memory_read_response(0xff);
        }
    }
    assert_eq!(a.read(0x4015) & 0x80, 0x80);
}

#[test]
fn triangle_length_expires() {
    let mut a = APU::new(44100.0);
    a.write(0x4015, 4);
    a.write(0x4008, 1);
    a.write(0x400b, 0x18); // length 2
    for _ in 0..40000 {
        a.step();
    }
    assert_eq!(a.read(0x4015) & 4, 0);
}

#[test]
fn controller_two_bus_route() {
    let mut c = cpu(&[]);
    c.bus.joypad2.update(1);
    c.bus.write_byte(0x4016, 1);
    c.bus.write_byte(0x4016, 0);
    assert_eq!(c.bus.read_byte(0x4017) & 1, 1);
}

#[test]
fn malformed_rom_returns_error() {
    assert!(ROM::new(vec![]).is_err());
}

#[test]
fn malformed_savestate_returns_error() {
    assert!(SaveState::decode(&[]).is_err());
}

#[test]
fn nrom_ram_window_does_not_panic() {
    let mut r = rom(0, 1, 1);
    mmc_write(&mut r, 0x6800, 0x42);
}

#[test]
fn unrom_ram_window_does_not_panic() {
    let mut r = rom(2, 2, 0);
    mmc_write(&mut r, 0x6800, 0x42);
}

#[test]
fn jsr_from_stack_page_reads_low_operand_before_push() {
    let mut c = cpu(&[]);
    c.pc = 0x1fc;
    c.bus.write_byte(0x1fc, 0x20);
    c.bus.write_byte(0x1fd, 0x00);
    c.bus.write_byte(0x1fe, 0x03);
    c.step();
    assert_eq!(c.pc, 0x0300);
}

#[test]
fn ppustatus_write_is_ignored() {
    let mut p = PPU::new(rom(0, 1, 1));
    p.write_register(0x2002, 0);
}

#[test]
fn soft_reset_preserves_stack_position() {
    let mut c = cpu(&[0xa2, 0x80, 0x9a]);
    c.step();
    c.step();
    c.soft_reset();
    assert_eq!(reg(&c, "SP:"), 0x7d);
}

#[test]
fn controller_latches_buttons_until_next_strobe() {
    let mut c = cpu(&[]);
    c.bus.joypad1.update(0x55);
    c.bus.write_byte(0x4016, 1);
    c.bus.write_byte(0x4016, 0);
    c.bus.joypad1.update(0xaa);
    for bit in 0..8 {
        assert_eq!(c.bus.read_byte(0x4016) & 1, (0x55 >> bit) & 1);
    }
    assert_eq!(c.bus.read_byte(0x4016) & 1, 1);
}

#[test]
fn disabled_cartridge_ram_preserves_cpu_open_bus() {
    let mut c = cpu(&[]);
    c.bus.write_byte(0xa001, 0);
    c.bus.write_byte(0x0100, 0xa5);
    assert_eq!(c.bus.read_byte(0x6000), 0xa5);
}

#[test]
fn mmc1_ignores_second_rmw_write() {
    let mut r = rom(1, 4, 0);
    for (cycle, value) in [(10, 1), (11, 0), (20, 0), (30, 0), (40, 0), (50, 0)] {
        r.mapper.cpu_write(&mut r.cart, 0xe000, value, cycle);
    }
    assert_eq!(mmc_read(&mut r, 0x8000), 2);
}

#[test]
fn chr_rom_is_immutable_and_small_banks_wrap() {
    for mapper in [1, 2] {
        let mut r = rom(mapper, 2, 1);
        let original = mmc_read(&mut r, 0x1234);
        mmc_write(&mut r, 0x1234, 0xff);
        assert_eq!(mmc_read(&mut r, 0x1234), original);
        for _ in 0..5 {
            mmc_write(&mut r, 0xe000, 0xff);
        }
        let _ = mmc_read(&mut r, 0xffff);
    }
}

#[test]
fn malformed_rom_payloads_are_rejected() {
    for len in 0..16 {
        assert!(ROM::new(vec![0; len]).is_err());
    }
    let mut data = vec![0; 16];
    data[..4].copy_from_slice(b"NES\x1a");
    data[4] = 1;
    assert!(ROM::new(data.clone()).is_err());
    data.resize(16 + 0x4000, 0);
    data[7] = 8;
    assert!(ROM::new(data).is_err());
}

fn replay_fixture() -> nessy::nes::Nes {
    let mut r = rom(4, 4, 1);
    let base = r.cart.prg_rom_start;
    // Enable both rendering pipelines and pulse audio, then loop over RAM RMWs.
    let program = [
        0xa9, 0x18, 0x8d, 0x01, 0x20, 0xa9, 1, 0x8d, 0x15, 0x40, 0xa9, 0x3f, 0x8d, 0x00, 0x40,
        0xa9, 0x40, 0x8d, 0x02, 0x40, 0xa9, 0x08, 0x8d, 0x03, 0x40, 0xe6, 0x00, 0x4c, 0x19, 0xe0,
    ];
    r.cart.bytes[base + 0xe000..base + 0xe000 + program.len()].copy_from_slice(&program);
    r.cart.bytes[base + 0xfffc..base + 0xfffe].copy_from_slice(&[0, 0xe0]);
    nessy::nes::Nes::new(r, 44100.0)
}

#[test]
fn mid_frame_save_replays_video_audio_and_every_saved_component() {
    let mut nes = replay_fixture();
    for _ in 0..5000 {
        nes.step();
    }
    let saved = nes.save_state().encode();
    let mut first = [0.0; 2048];
    nes.next_samples(&mut first);
    let expected = nes.save_state().encode();
    let frame = nes.get_frame().to_vec();
    nes.load_state(&saved).unwrap();
    let mut second = [0.0; 2048];
    nes.next_samples(&mut second);
    assert_eq!(first, second);
    assert!(first.iter().any(|v| *v != 0.0));
    assert_eq!(nes.get_frame(), frame);
    assert_eq!(nes.save_state().encode(), expected);
}

#[test]
fn failed_load_is_atomic_even_after_cpu_and_ppu_changes() {
    let mut nes = replay_fixture();
    let mut bad = nes.save_state();
    bad.get_root_mut().get("MMC3").unwrap().data = nessy::savestate::ByteBuffer::new();
    for _ in 0..100 {
        nes.step();
    }
    let before = nes.save_state().encode();
    assert!(nes.load_state(&bad.encode()).is_err());
    assert_eq!(nes.save_state().encode(), before);
    for length in [0, 1, 5, 37, 38, 50, 100, before.len() - 1] {
        assert!(nes.load_state(&before[..length]).is_err());
        assert_eq!(nes.save_state().encode(), before);
    }
}

#[test]
fn audio_underrun_silences_tail_and_overflow_retains_recent_samples() {
    let mut a = APU::new(44100.0);
    let mut samples = [1.0; 16];
    a.fill(&mut samples);
    assert_eq!(samples, [0.0; 16]);
    for _ in 0..500_000 {
        a.step();
    }
    assert_eq!(a.remaining_samples(), 8191);
    a.fill(&mut samples);
    assert_eq!(a.remaining_samples(), 8175);
}
