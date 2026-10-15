use crate::{
    bus::SystemBus,
    cartridge::{Cartridge, ConsoleType, Timing},
    cpu::{Cpu, CpuError},
};

/// Frontend boundary. No host timing, devices, or file I/O are owned here.
pub struct Console {
    pub cpu: Cpu,
    pub bus: SystemBus,
    sample_rate: u32,
}
impl Console {
    /// Panics for nonstandard consoles or PAL/Dendy-only images. Multi-region
    /// images run in NTSC mode; parsing metadata never enables other systems.
    pub fn new(cartridge: Cartridge, sample_rate: u32) -> Self {
        assert_eq!(
            cartridge.console_type,
            ConsoleType::Nes,
            "only standard NES consoles are supported"
        );
        assert!(
            matches!(cartridge.timing, Timing::Ntsc | Timing::MultiRegion),
            "only NTSC timing is supported"
        );
        let mut console = Self {
            cpu: Cpu::new(),
            bus: SystemBus::new(cartridge, sample_rate),
            sample_rate,
        };
        console.cpu.reset(&mut console.bus);
        console
    }
    /// Executes complete instructions until the PPU signals a frame. Hardware
    /// time can overshoot the boundary by one instruction or DMA transfer.
    /// Panics on unsupported/JAM opcodes; use `try_step_frame` to handle errors.
    pub fn step_frame(&mut self) {
        self.try_step_frame()
            .expect("CPU stopped while stepping frame");
    }
    pub fn try_step_frame(&mut self) -> Result<(), CpuError> {
        self.bus.ppu.frame_ready = false;
        while !self.bus.ppu.frame_ready {
            self.cpu.step(&mut self.bus)?;
        }
        Ok(())
    }
    pub fn framebuffer(&self) -> &[u8] {
        self.bus.ppu.framebuffer()
    }
    pub fn drain_audio(&mut self, out: &mut Vec<f32>) {
        self.bus.apu.take_samples(out);
    }
    pub fn set_buttons(&mut self, port: usize, state: u8) {
        if let Some(controller) = self.bus.controllers.get_mut(port) {
            controller.set_buttons(state);
        }
    }
    /// Soft reset preserves CPU registers and RAM, reinitializes PPU/APU, then
    /// performs the seven-cycle CPU reset sequence (SP decrements three times).
    pub fn reset(&mut self) {
        self.bus.reset_devices(self.sample_rate);
        self.cpu.reset(&mut self.bus);
    }
}
