#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleLower,
    SingleUpper,
    FourScreen,
}

pub trait Mapper {
    fn cpu_read(&mut self, addr: u16) -> Option<u8>;
    fn cpu_write(&mut self, addr: u16, v: u8);
    fn ppu_read(&mut self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, v: u8);
    fn mirroring(&self) -> Mirroring;
    fn irq_pending(&self) -> bool {
        false
    }
    fn cpu_clock(&mut self) {}
}
