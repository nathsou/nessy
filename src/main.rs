mod bus;
mod console;
mod cpu;
mod ppu;
use bus::controller::{Joypad, JoypadStatus};
use console::Console;
use cpu::rom::ROM;
use ppu::screen::Screen;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;
use std::collections::hash_map::HashMap;
use std::time::{Duration, Instant};

const SCALE_FACTOR: usize = 3;
const TARGET_FPS: f64 = 60.0;

fn get_controller_map() -> HashMap<Keycode, JoypadStatus> {
    let mut controller_map = HashMap::new();
    controller_map.insert(Keycode::W, JoypadStatus::UP);
    controller_map.insert(Keycode::S, JoypadStatus::DOWN);
    controller_map.insert(Keycode::A, JoypadStatus::LEFT);
    controller_map.insert(Keycode::D, JoypadStatus::RIGHT);
    controller_map.insert(Keycode::L, JoypadStatus::A);
    controller_map.insert(Keycode::K, JoypadStatus::B);
    controller_map.insert(Keycode::Return, JoypadStatus::START);
    controller_map.insert(Keycode::Space, JoypadStatus::SELECT);
    controller_map
}

fn handle_events(
    event_pump: &mut EventPump,
    controller: &mut Joypad,
    controller_map: &HashMap<Keycode, JoypadStatus>,
) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                std::process::exit(0);
            }
            Event::KeyDown { keycode, .. } => {
                if let Some(keycode) = keycode {
                    if let Some(button) = controller_map.get(&keycode) {
                        controller.update_button_state(*button, true);
                    }
                }
            }
            Event::KeyUp { keycode, .. } => {
                if let Some(keycode) = keycode {
                    if let Some(button) = controller_map.get(&keycode) {
                        controller.update_button_state(*button, false);
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let args = std::env::args().take(2).collect::<Vec<_>>();

    if args.len() < 2 {
        eprintln!("usage: nessy rom.nes");
    } else {
        let rom_path = &args[1];
        let rom = ROM::load(rom_path).unwrap();
        let mut console = Console::new(rom);

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(
                "nessy",
                (Screen::WIDTH * SCALE_FACTOR) as u32,
                (Screen::HEIGHT * SCALE_FACTOR) as u32,
            )
            .position_centered()
            .build()
            .expect("could not initialize video subsystem");

        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .expect("could not make a canvas");

        canvas
            .set_scale(SCALE_FACTOR as f32, SCALE_FACTOR as f32)
            .unwrap();

        let creator = canvas.texture_creator();
        let mut texture = creator
            .create_texture_target(
                PixelFormatEnum::RGB24,
                Screen::WIDTH as u32,
                Screen::HEIGHT as u32,
            )
            .unwrap();

        let mut event_pump = sdl_context.event_pump().unwrap();
        let controller_map = get_controller_map();
        let target_frame_duration = Duration::from_secs_f64(1.0 / TARGET_FPS);
        let mut last_frame = Instant::now();

        loop {
            let (frame, controller) = console.next_frame();

            handle_events(&mut event_pump, controller, &controller_map);

            texture.update(None, frame, Screen::WIDTH * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();

            let now = Instant::now();
            let frame_duration = now - last_frame;
            last_frame = now;

            if frame_duration < target_frame_duration {
                std::thread::sleep(target_frame_duration - frame_duration);
            }

            // let fps = 1.0 / frame_duration.as_secs_f64();
            // println!("FPS: {fps:.2}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Bus;
    use crate::cpu::CPU;

    #[test]
    fn test_nestest_dump() {
        let rom = ROM::load("src/tests/nestest.nes").expect("nestest.nes not found");
        let logs = include_str!("tests/nestest.log").lines();
        let bus = Bus::new(rom);
        let mut cpu = CPU::new(bus);
        cpu.pc = 0xc000;

        for log_line in logs {
            if cpu.pc == 0xc6bd {
                // illegal opcodes after this point
                break;
            }

            println!("{log_line}");

            let expected_pc = &log_line[0..4];
            let actual_pc = format!("{:04X}", cpu.pc);
            assert_eq!(expected_pc, actual_pc, "PC mismatch");

            let expected_regs = &log_line[48..73];
            let actual_regs = format!("{cpu:?}");
            assert_eq!(expected_regs, actual_regs, "Registers mismatch");

            cpu.step();
        }
    }

    #[test]
    #[ignore]
    fn test_smb_dump() {
        let rom = ROM::load("roms/smb.nes").expect("smb.nes not found");
        let dump = std::fs::read_to_string("smb.log").expect("smb.log not found");
        let bus = Bus::new(rom);
        let mut cpu = CPU::new(bus);
        let mut i = 0;
        let lines = dump.lines().collect::<Vec<_>>();

        while i < lines.len() {
            let line = lines[i];

            if line.starts_with('!') {
                let joypad1 = u8::from_str_radix(&line[1..], 2).unwrap();
                cpu.bus.controller.update(joypad1);
                i += 1;
            } else {
                let actual = cpu.state_fmt();
                let prev_pc = cpu.pc;
                let cycles = cpu.step();
                cpu.bus.advance_ppu(cycles);

                if prev_pc != cpu.pc {
                    let with_cycles = format!(
                        "{actual} CY {cycles} {}|{}",
                        cpu.bus.ppu.scanline, cpu.bus.ppu.cycle
                    );

                    assert_eq!(with_cycles, line);
                    i += 1;
                }
            }
        }
    }
}
