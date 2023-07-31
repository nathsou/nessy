extern crate nessy;

use std::collections::HashMap;

use nessy::{
    controller::{Joypad, JoypadStatus},
    cpu::rom::ROM,
    Nes, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use sdl2::{
    audio::{AudioCallback, AudioSpecDesired},
    event::Event,
    keyboard::Keycode,
    pixels::PixelFormatEnum,
    EventPump,
};

const SCALE_FACTOR: usize = 2;
const SAMPLE_RATE: f64 = 44_100.0;
const WIDTH: usize = SCREEN_WIDTH;
const HEIGHT: usize = SCREEN_HEIGHT;

fn build_controller_map() -> HashMap<Keycode, JoypadStatus> {
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

struct APUCallback<'a> {
    nes: &'a mut Nes,
    frame: &'a mut [u8],
    avoid_underruns: bool,
}

impl<'a> AudioCallback for APUCallback<'a> {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        self.nes
            .fill_audio_buffer(out, self.frame, self.avoid_underruns);
    }
}

fn handle_events(
    event_pump: &mut EventPump,
    controller: &mut Joypad,
    controller_map: &HashMap<Keycode, JoypadStatus>,
    paused: &mut bool,
) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => {
                std::process::exit(0);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            }
            | Event::KeyDown {
                keycode: Some(Keycode::Tab),
                ..
            } => {
                *paused = !*paused;
            }
            Event::KeyDown { keycode, .. } => {
                if let Some(&button) = keycode.and_then(|k| controller_map.get(&k)) {
                    controller.status.insert(button);
                }
            }
            Event::KeyUp { keycode, .. } => {
                if let Some(&button) = keycode.and_then(|k| controller_map.get(&k)) {
                    controller.status.remove(button);
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
        let bytes = std::fs::read(rom_path).unwrap();
        let rom = ROM::new(bytes).unwrap();
        let mut nes = Nes::new(rom, SAMPLE_RATE);

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();
        let desired_audio_spec = AudioSpecDesired {
            freq: Some(SAMPLE_RATE as i32),
            channels: Some(1), // mono,
            samples: Some(1024),
        };

        let window = video_subsystem
            .window(
                "nessy",
                (WIDTH * SCALE_FACTOR) as u32,
                (HEIGHT * SCALE_FACTOR) as u32,
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
            .create_texture_target(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)
            .unwrap();

        let mut event_pump = sdl_context.event_pump().unwrap();
        let controller_map = build_controller_map();
        let mut frame = [0; WIDTH * HEIGHT * 3];

        let audio_device = audio_subsystem
            .open_playback(None, &desired_audio_spec, |_| APUCallback {
                nes: &mut nes,
                frame: &mut frame,
                avoid_underruns: false,
            })
            .unwrap();

        audio_device.resume();

        let mut paused = false;

        loop {
            handle_events(
                &mut event_pump,
                nes.get_joypad1_mut(),
                &controller_map,
                &mut paused,
            );

            if !paused {
                nes.next_frame_inaccurate(&mut frame);
            }

            // nes.get_frame(&mut frame);
            texture.update(None, &frame, WIDTH * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();

            canvas.present();
        }
    }
}
