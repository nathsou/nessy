mod bus;
mod cpu;
mod graphics;
mod ppu;
use bus::Bus;
use cpu::rom::ROM;
use cpu::CPU;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

const SCALE_FACTOR: usize = 2;

fn main() {
    // let args = env::args().take(2).collect::<Vec<_>>();

    // if args.len() < 2 {
    //     eprintln!("usage: nessy rom.nes");
    // } else {
    // let rom_path = &args[1];
    // let rom = ROM::from(rom_path).unwrap();

    let rom = ROM::load("roms/DK.nes").unwrap();
    let bus = Bus::new(rom);
    // bus.ppu.show_tile_bank(0);
    let mut cpu = CPU::new(bus);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "nessy",
            (256 * SCALE_FACTOR) as u32,
            (240 * SCALE_FACTOR) as u32,
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
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

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
        }

        // ::std::thread::sleep(std::time::Duration::new(0, 100_000_000));
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
