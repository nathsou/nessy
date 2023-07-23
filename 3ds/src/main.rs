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
const ROM_BYTES: &[u8] = include_bytes!("../assets/Kirby's Adventure.nes");
const LEFT_X_OFFSET_TOP: usize = (TOP_SCREEN_WIDTH - NES_SCREEN_WIDTH) / 2;
const LEFT_X_OFFSET_BOTTOM: usize = (BOTTOM_SCREEN_WIDTH - NES_SCREEN_WIDTH) / 2;
static LOGO: &[u8] = include_bytes!("../assets/logo.rgb");

#[inline]
fn is_key_active(hid: &Hid, key: KeyPad) -> bool {
    hid.keys_down().contains(key) || hid.keys_held().contains(key)
}

fn main() {
    let mut nes_frame_buffer: [u8; FRAME_BUFFER_BYTE_SIZE] = [0; FRAME_BUFFER_BYTE_SIZE];
    ctru::use_panic_handler();

    let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
    let mut hid = Hid::new().expect("Couldn't obtain HID controller");
    let apt = Apt::new().expect("Couldn't obtain APT controller");
    // let _console = Console::new(gfx.bottom_screen.borrow_mut());

    // println!("\x1b[0;3HPress L + R to exit.");

    let mut top_screen = gfx.top_screen.borrow_mut();
    let mut bottom_screen = gfx.bottom_screen.borrow_mut();
    bottom_screen.set_double_buffering(false);
    bottom_screen.swap_buffers();

    // We don't need double buffering in this example.
    // In this way we can draw our image only once on screen.
    top_screen.set_double_buffering(false);

    let rom = ROM::new(ROM_BYTES.to_vec()).expect("Couldn't load ROM");
    let mut nes = Nes::new(rom, SAMPLE_RATE);
    let mut nes_frame_buffer = [0u8; FRAME_BUFFER_BYTE_SIZE];
    let mut top_frame_buffer = [0u8; TOP_SCREEN_WIDTH * TOP_SCREEN_HEIGHT * 3];
    let mut bottom_frame_buffer = [0u8; BOTTOM_SCREEN_WIDTH * BOTTOM_SCREEN_HEIGHT * 3];

    let bottom_offset = LEFT_X_OFFSET_BOTTOM * BOTTOM_SCREEN_HEIGHT * 3;
    bottom_frame_buffer[bottom_offset..(LOGO.len() + bottom_offset)].copy_from_slice(LOGO);

    // Main loop
    while apt.main_loop() {
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

        if is_key_active(&hid, KeyPad::A) {
            joypad1_state.insert(JoypadStatus::A);
        }

        if is_key_active(&hid, KeyPad::B) {
            joypad1_state.insert(JoypadStatus::B);
        }

        if is_key_active(&hid, KeyPad::DPAD_UP) {
            joypad1_state.insert(JoypadStatus::UP);
        }

        if is_key_active(&hid, KeyPad::DPAD_DOWN) {
            joypad1_state.insert(JoypadStatus::DOWN);
        }

        if is_key_active(&hid, KeyPad::DPAD_LEFT) {
            joypad1_state.insert(JoypadStatus::LEFT);
        }

        if is_key_active(&hid, KeyPad::DPAD_RIGHT) {
            joypad1_state.insert(JoypadStatus::RIGHT);
        }

        nes.get_joypad1_mut().update(joypad1_state.bits());

        nes.next_frame();
        nes_frame_buffer.copy_from_slice(&nes.get_frame());

        // rotate the frame buffer 90 degrees
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

        unsafe {
            top_screen
                .raw_framebuffer()
                .ptr
                .copy_from(top_frame_buffer.as_ptr(), top_frame_buffer.len());

            bottom_screen
                .raw_framebuffer()
                .ptr
                .copy_from(bottom_frame_buffer.as_ptr(), bottom_frame_buffer.len());
        }

        // Flush and swap framebuffers
        top_screen.flush_buffers();
        bottom_screen.flush_buffers();
        // top_screen.swap_buffers();

        //Wait for VBlank
        gfx.wait_for_vblank();
    }
}
