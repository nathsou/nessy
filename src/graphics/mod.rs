use crate::cpu::memory::Memory;
use crate::cpu::CPU;
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{Texture, WindowCanvas};

fn color_mapping(byte: u8) -> Color {
    match byte {
        0 => Color::BLACK,
        1 => Color::WHITE,
        2 | 9 => Color::GREY,
        3 | 10 => Color::RED,
        4 | 11 => Color::GREEN,
        5 | 12 => Color::BLUE,
        6 | 13 => Color::MAGENTA,
        7 | 14 => Color::YELLOW,
        _ => Color::CYAN,
    }
}

fn render(
    canvas: &mut WindowCanvas,
    cpu: &CPU,
    buf: &mut [u8; 32 * 32 * 3],
    texture: &mut Texture,
) {
    let mut updated = false;
    let mut idx = 0;

    for i in 0x200..0x600 {
        let (r, g, b) = color_mapping(cpu.bus.read_byte(i)).rgb();
        if buf[idx] != r || buf[idx + 1] != g || buf[idx + 2] != b {
            buf[idx] = r;
            buf[idx + 1] = g;
            buf[idx + 2] = b;
            updated = true;
        }

        idx += 3;
    }

    if updated {
        texture.update(None, buf, 32 * 3).unwrap();
        canvas.copy(texture, None, None).unwrap();
        canvas.present();
    }
}

const SCALE_FACTOR: u32 = 20;

pub fn begin(cpu: &mut CPU) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("nessy", 32 * SCALE_FACTOR, 32 * SCALE_FACTOR)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("could not make a canvas");

    canvas.set_scale(SCALE_FACTOR as f32, SCALE_FACTOR as f32)?;

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 32, 32)
        .unwrap();

    let mut event_pump = sdl_context.event_pump()?;
    let mut frame_buffer = [0u8; 32 * 32 * 3];
    let mut rng = rand::thread_rng();

    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    cpu.bus.write_byte(0xff, 0x77);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    cpu.bus.write_byte(0xff, 0x73);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    cpu.bus.write_byte(0xff, 0x61);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    cpu.bus.write_byte(0xff, 0x64);
                }
                _ => {}
            }
        }

        // Update
        cpu.bus.write_byte(0xfe, rng.gen_range(1..16));
        cpu.step();

        // Render
        render(&mut canvas, cpu, &mut frame_buffer, &mut texture);

        // Time management!
        ::std::thread::sleep(std::time::Duration::new(0, 30_000));
    }

    Ok(())
}
