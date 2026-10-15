//! Ricoh 2A03 CPU interface: every read/write must advance one CPU cycle.
pub trait CpuBus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, v: u8);
    fn take_nmi(&mut self) -> bool {
        false
    }
    fn irq_pending(&self) -> bool {
        false
    }
}
#[derive(Default)]
pub struct Cpu;
