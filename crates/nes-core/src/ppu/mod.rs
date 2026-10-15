//! Interface placeholder owned by nes-ppu-apu. Ticks frames but renders no pixels.
use crate::{FRAME_HEIGHT, FRAME_WIDTH};

/// Pattern tables, mirrored nametables and palette memory supplied by the bus.
pub trait PpuMemory {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, v: u8);
}

pub struct Ppu {
    pub frame_ready: bool,
    dots: usize,
    framebuffer: Vec<u8>,
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}
impl Ppu {
    pub fn new() -> Self {
        Self {
            frame_ready: false,
            dots: 0,
            framebuffer: vec![0; FRAME_WIDTH * FRAME_HEIGHT * 4],
        }
    }
    pub fn cpu_read(&mut self, _reg: u16, _mem: &mut dyn PpuMemory) -> u8 {
        0
    }
    pub fn cpu_write(&mut self, _reg: u16, _v: u8, _mem: &mut dyn PpuMemory) {}
    pub fn tick(&mut self, _mem: &mut dyn PpuMemory) {
        self.dots += 1;
        if self.dots == 341 * 262 {
            self.dots = 0;
            self.frame_ready = true;
        }
    }
    pub fn take_nmi(&mut self) -> bool {
        false
    }
    pub fn framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }
    pub fn oam_dma_write(&mut self, _byte: u8) {}
}
