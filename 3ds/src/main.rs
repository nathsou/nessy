extern crate nessy;

use ctru::prelude::*;
use ctru::services::gfx::{Flush, Screen, Side, Swap, TopScreen3D};
use nessy::{
    controller::{Joypad, JoypadStatus},
    cpu::rom::ROM,
    Nes, FRAME_BUFFER_BYTE_SIZE, SCREEN_HEIGHT as NES_SCREEN_HEIGHT,
    SCREEN_WIDTH as NES_SCREEN_WIDTH,
};

const TOP_SCREEN_WIDTH: usize = 400; // px
const TOP_SCREEN_HEIGHT: usize = 240; // p
const BOTTOM_SCREEN_WIDTH: usize = 320; // px
const BOTTOM_SCREEN_HEIGHT: usize = 240; // px
const SAMPLE_RATE: f64 = 44_100.0; // Hz
const ROM_BYTES: &[u8] = include_bytes!("../assets/Super Mario Bros.nes");
const LEFT_X_OFFSET_TOP: usize = (TOP_SCREEN_WIDTH - NES_SCREEN_WIDTH) / 2;
const LEFT_X_OFFSET_BOTTOM: usize = (BOTTOM_SCREEN_WIDTH - NES_SCREEN_WIDTH) / 2;

fn is_key_active(hid: &Hid, key: KeyPad) -> bool {
    hid.keys_held().contains(key)
}

fn main() {
    let mut nes_frame_buffer = [0u8; FRAME_BUFFER_BYTE_SIZE];
    ctru::use_panic_handler();

    let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
    let mut hid = Hid::new().expect("Couldn't obtain HID controller");
    let apt = Apt::new().expect("Couldn't obtain APT controller");
    // let _console = Console::new(gfx.bottom_screen.borrow_mut());

    // println!("\x1b[0;3HPress L + R to exit.");

    let mut top_screen = gfx.top_screen.borrow_mut();
    top_screen.set_double_buffering(false);
    top_screen.swap_buffers();

    let rom = ROM::new(ROM_BYTES.to_vec()).expect("Couldn't load ROM");
    let mut nes = Nes::new(rom, SAMPLE_RATE);
    let mut top_frame_buffer = [0u8; TOP_SCREEN_WIDTH * TOP_SCREEN_HEIGHT * 3];
    // let mut last_frame = std::time::Instant::now();

    // Main loop
    while apt.main_loop() {
        // let frame_duration = last_frame.elapsed();
        // last_frame = std::time::Instant::now();
        // Scan all the inputs. This should be done once for each frame
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::L | KeyPad::R) {
            break;
        }

        let mut joypad1_state = JoypadStatus::empty();

        if is_key_active(&hid, KeyPad::START) {
            joypad1_state.insert(JoypadStatus::START);
        }

        if is_key_active(&hid, KeyPad::SELECT) {
            joypad1_state.insert(JoypadStatus::SELECT);
        }

        if is_key_active(&hid, KeyPad::A) || is_key_active(&hid, KeyPad::Y) {
            joypad1_state.insert(JoypadStatus::A);
        }

        if is_key_active(&hid, KeyPad::B) || is_key_active(&hid, KeyPad::X) {
            joypad1_state.insert(JoypadStatus::B);
        }

        if is_key_active(&hid, KeyPad::DPAD_UP) || is_key_active(&hid, KeyPad::CPAD_UP) {
            joypad1_state.insert(JoypadStatus::UP);
        }

        if is_key_active(&hid, KeyPad::DPAD_DOWN) || is_key_active(&hid, KeyPad::CPAD_DOWN) {
            joypad1_state.insert(JoypadStatus::DOWN);
        }

        if is_key_active(&hid, KeyPad::DPAD_LEFT) || is_key_active(&hid, KeyPad::CPAD_LEFT) {
            joypad1_state.insert(JoypadStatus::LEFT);
        }

        if is_key_active(&hid, KeyPad::DPAD_RIGHT) || is_key_active(&hid, KeyPad::CPAD_RIGHT) {
            joypad1_state.insert(JoypadStatus::RIGHT);
        }

        nes.get_joypad1_mut().update(joypad1_state.bits());
        // let t0 = std::time::Instant::now();
        let offset = LEFT_X_OFFSET_TOP * NES_SCREEN_HEIGHT * 3;
        // nes.next_frame_inaccurate(
        //     &mut top_frame_buffer[offset..offset + NES_SCREEN_WIDTH * NES_SCREEN_HEIGHT * 3],
        // );

        nes.next_frame_inaccurate(&mut nes_frame_buffer);

        // let frame_time = t0.elapsed().as_millis() as usize;
        // let d0 = t0.elapsed();
        // let t1 = std::time::Instant::now();
        // nes.get_frame(&mut nes_frame_buffer);
        // let d1 = t1.elapsed();

        // let t2 = std::time::Instant::now();
        // // rotate the frame buffer 90 degrees
        for y in 0..NES_SCREEN_HEIGHT {
            for x in 0..NES_SCREEN_WIDTH {
                let src_index = (y * NES_SCREEN_WIDTH + x) * 3;
                let dst_index =
                    ((LEFT_X_OFFSET_TOP + x) * NES_SCREEN_HEIGHT + (NES_SCREEN_HEIGHT - y - 1)) * 3;
                // rgb -> bgr
                top_frame_buffer[dst_index] = nes_frame_buffer[src_index + 2];
                top_frame_buffer[dst_index + 1] = nes_frame_buffer[src_index + 1];
                top_frame_buffer[dst_index + 2] = nes_frame_buffer[src_index];
            }
        }

        // top_frame_buffer[..(NES_SCREEN_WIDTH * NES_SCREEN_HEIGHT * 3)]
        //     .copy_from_slice(&nes_frame_buffer[..]);

        // let d2 = t2.elapsed();

        // let t3 = std::time::Instant::now();
        unsafe {
            top_screen
                .raw_framebuffer()
                .ptr
                .copy_from(top_frame_buffer.as_ptr(), top_frame_buffer.len());
        }

        top_screen.flush_buffers();

        // let d3 = t3.elapsed();

        // println!(
        //     "nf {} gf {}, rt {} after {} frame {}",
        //     d0.as_millis(),
        //     d1.as_millis(),
        //     d2.as_millis(),
        //     d3.as_millis(),
        //     frame_duration.as_millis()
        // );

        //Wait for VBlank
        // gfx.wait_for_vblank();
    }
}
