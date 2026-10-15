//! Reusable NTSC NES core. No windowing, audio device, or file I/O dependencies.
pub mod apu;
pub mod bus;
pub mod cartridge;
pub mod console;
pub mod cpu;
pub mod input;
pub mod mapper;
pub mod ppu;

pub use cartridge::{Cartridge, CartridgeError};
pub use console::Console;
pub use mapper::{Mapper, Mirroring};

pub const CPU_HZ: u32 = 1_789_773;
pub const FRAME_WIDTH: usize = 256;
pub const FRAME_HEIGHT: usize = 240;
