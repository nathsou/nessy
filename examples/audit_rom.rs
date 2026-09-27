//! Headless runner for test ROMs using blargg's $6000 result protocol.
//! Usage: cargo run --release --example audit_rom -- path/to/test.nes [...]
//! Reports unsupported protocols/timeouts separately from pass/fail results.
use nessy::{
    bus::Bus,
    cpu::{rom::ROM, CPU},
};
use std::{
    env,
    panic::{catch_unwind, AssertUnwindSafe},
};

fn peek(cpu: &mut CPU, addr: u16) -> u8 {
    let rom = &mut cpu.bus.ppu.rom;
    rom.mapper.read(&mut rom.cart, addr)
}

fn run(path: &str, budget: u64) -> bool {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            println!("{path}: READ ERROR: {error}");
            return false;
        }
    };
    let rom = match ROM::new(bytes) {
        Ok(rom) => rom,
        Err(error) => {
            println!("{path}: UNSUPPORTED/INVALID ROM: {error:?}");
            return false;
        }
    };
    let mut cpu = CPU::new(Bus::new(rom, 44100.0));
    let mut cycles = 0u64;
    let mut running = false;
    let result = catch_unwind(AssertUnwindSafe(|| {
        while cycles < budget {
            let elapsed = cpu.step();
            cycles += elapsed as u64;
            if cycles % 1024 >= elapsed as u64 {
                continue;
            }
            let signature = [
                peek(&mut cpu, 0x6001),
                peek(&mut cpu, 0x6002),
                peek(&mut cpu, 0x6003),
            ];
            if signature != [0xde, 0xb0, 0x61] {
                continue;
            }
            let status = peek(&mut cpu, 0x6000);
            if status == 0x80 {
                running = true;
            }
            if running && status != 0x80 {
                return Some(status);
            }
        }
        None
    }));
    let outcome = match result {
        Ok(Some(0)) => "PASS".to_owned(),
        Ok(Some(0x81)) => "RESET REQUESTED (not completed)".to_owned(),
        Ok(Some(code)) => format!("FAIL {code}"),
        Ok(None) => format!("TIMEOUT (protocol started: {running})"),
        Err(_) => "PANIC".to_owned(),
    };
    let text: Vec<u8> = (0x6004..0x6800)
        .map(|addr| peek(&mut cpu, addr))
        .take_while(|&b| b != 0)
        .collect();
    println!(
        "{path}: {outcome}; cycles={cycles}; pc={:04X}; text={:?}",
        cpu.pc,
        String::from_utf8_lossy(&text)
    );
    matches!(result, Ok(Some(0)))
}

fn main() {
    let paths: Vec<_> = env::args().skip(1).collect();
    assert!(!paths.is_empty(), "provide one or more test ROM paths");
    let budget = env::var("NESSY_AUDIT_MAX_CYCLES")
        .ok()
        .map(|s| s.parse().unwrap())
        .unwrap_or(100_000_000);
    let mut passed = 0;
    for path in &paths {
        if run(path, budget) {
            passed += 1;
        }
    }
    println!("{passed}/{} ROMs passed", paths.len());
    if passed != paths.len() {
        std::process::exit(1);
    }
}
