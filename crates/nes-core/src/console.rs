use crate::{apu::Apu, cartridge::Cartridge, ppu::Ppu};

pub struct Console {
    ppu: Ppu,
    apu: Apu,
    _cartridge: Cartridge,
}
impl Console {
    pub fn new(cartridge: Cartridge, sample_rate: u32) -> Self {
        Self {
            ppu: Ppu::new(),
            apu: Apu::new(sample_rate),
            _cartridge: cartridge,
        }
    }
    pub fn step_frame(&mut self) {}
    pub fn framebuffer(&self) -> &[u8] {
        self.ppu.framebuffer()
    }
    pub fn drain_audio(&mut self, out: &mut Vec<f32>) {
        self.apu.take_samples(out);
    }
    pub fn set_buttons(&mut self, _port: usize, _state: u8) {}
    pub fn reset(&mut self) {}
}
