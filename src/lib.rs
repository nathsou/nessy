pub mod apu;
pub mod bus;
pub mod cpu;
pub mod nes;
pub mod ppu;
pub mod savestate;

pub use bus::controller;
pub use nes::Nes;

pub const SCREEN_WIDTH: usize = 256; // px
pub const SCREEN_HEIGHT: usize = 240; // px
pub const FRAME_BUFFER_BYTE_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT * 3; // bytes
