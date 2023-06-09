mod bus;
mod cpu;
mod graphics;
mod ppu;
use bus::controller::JoypadStatus;
use bus::Bus;
use cpu::rom::ROM;
use cpu::CPU;
use ppu::screen::Screen;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::collections::hash_map::HashMap;

const SCALE_FACTOR: usize = 2;

fn get_controller_map() -> HashMap<Keycode, JoypadStatus> {
    let mut controller_map = HashMap::new();
    controller_map.insert(Keycode::W, JoypadStatus::UP);
    controller_map.insert(Keycode::S, JoypadStatus::DOWN);
    controller_map.insert(Keycode::A, JoypadStatus::LEFT);
    controller_map.insert(Keycode::D, JoypadStatus::RIGHT);
    controller_map.insert(Keycode::K, JoypadStatus::A);
    controller_map.insert(Keycode::L, JoypadStatus::B);
    controller_map.insert(Keycode::Return, JoypadStatus::START);
    controller_map.insert(Keycode::Space, JoypadStatus::SELECT);
    controller_map
}

fn main() {
    let args = std::env::args().take(2).collect::<Vec<_>>();

    if args.len() < 2 {
        eprintln!("usage: nessy rom.nes");
    } else {
        let rom_path = &args[1];
        let rom = ROM::load(rom_path).unwrap();
        let bus = Bus::new(rom);
        let mut cpu = CPU::new(bus);

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
        let mut last_frame_timestamp = std::time::Instant::now();
        let target_frame_duration = std::time::Duration::from_millis(1000 / 60);

        'running: loop {
            // handle events
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        break 'running;
                    }
                    Event::KeyDown { keycode, .. } => {
                        if let Some(keycode) = keycode {
                            if let Some(button) = controller_map.get(&keycode) {
                                cpu.bus.controller.update_button_state(*button, true);
                            }
                        }
                    }
                    Event::KeyUp { keycode, .. } => {
                        if let Some(keycode) = keycode {
                            if let Some(button) = controller_map.get(&keycode) {
                                cpu.bus.controller.update_button_state(*button, false);
                            }
                        }
                    }
                    _ => {}
                }
            }

            // update
            let vblank_before = cpu.bus.ppu.is_vblank();
            let cpu_cycles = cpu.step();
            cpu.bus.advance_ppu(cpu_cycles);
            let vblank_after = cpu.bus.ppu.is_vblank();

            if !vblank_before && vblank_after {
                cpu.bus.ppu.render_frame();
                texture
                    .update(None, &cpu.bus.ppu.screen.pixels, 256 * 3)
                    .unwrap();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();

                let now = std::time::Instant::now();
                let frame_duration = now - last_frame_timestamp;
                if frame_duration < target_frame_duration {
                    std::thread::sleep(target_frame_duration - frame_duration);
                }

                let fps = 1.0 / frame_duration.as_secs_f64();
                last_frame_timestamp = now;
                println!("FPS: {fps}");
            }
        }
    }
}

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
