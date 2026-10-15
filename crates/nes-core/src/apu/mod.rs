//! Interface placeholder owned by nes-ppu-apu. Produces no samples yet.
#[derive(Default)]
pub struct Apu;
impl Apu {
    pub fn new(_sample_rate: u32) -> Self {
        Self
    }
    pub fn cpu_read(&mut self, _addr: u16) -> u8 {
        0
    }
    pub fn cpu_write(&mut self, _addr: u16, _v: u8) {}
    pub fn tick(&mut self) {}
    pub fn irq_pending(&self) -> bool {
        false
    }
    pub fn take_samples(&mut self, _out: &mut Vec<f32>) {}
    pub fn dmc_request(&mut self) -> Option<u16> {
        None
    }
    pub fn dmc_provide(&mut self, _byte: u8) {}
}
