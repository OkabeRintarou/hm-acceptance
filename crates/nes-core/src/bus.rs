//! NTSC system bus. Devices and PPU address space have disjoint ownership so
//! register calls can safely borrow the PPU and mapper/VRAM at the same time.
use crate::{
    apu::Apu,
    cartridge::Cartridge,
    cpu::CpuBus,
    input::Controller,
    mapper::{Mapper, Mirroring, Nrom},
    ppu::{Ppu, PpuMemory},
};

pub struct PpuAddressSpace {
    pub mapper: Box<dyn Mapper>,
    // First 2 KiB is console CIRAM; additional 2 KiB supports four-screen boards.
    nametables: [u8; 4096],
    palette: [u8; 32],
}
impl PpuAddressSpace {
    pub fn new(mapper: Box<dyn Mapper>) -> Self {
        Self {
            mapper,
            nametables: [0; 4096],
            palette: [0; 32],
        }
    }
    fn nametable_index(&self, addr: u16) -> usize {
        let offset = usize::from((addr - 0x2000) & 0x0fff);
        let table = offset / 1024;
        let physical = match self.mapper.mirroring() {
            Mirroring::Horizontal => table / 2,
            Mirroring::Vertical => table & 1,
            Mirroring::SingleLower => 0,
            Mirroring::SingleUpper => 1,
            Mirroring::FourScreen => table,
        };
        physical * 1024 + offset % 1024
    }
    fn palette_index(addr: u16) -> usize {
        let i = usize::from(addr & 31);
        if i >= 16 && i & 3 == 0 {
            i - 16
        } else {
            i
        }
    }
}
impl PpuMemory for PpuAddressSpace {
    fn read(&mut self, addr: u16) -> u8 {
        let addr = addr & 0x3fff;
        match addr {
            0..=0x1fff => self.mapper.ppu_read(addr),
            0x2000..=0x3eff => self.nametables[self.nametable_index(addr)],
            _ => self.palette[Self::palette_index(addr)],
        }
    }
    fn write(&mut self, addr: u16, v: u8) {
        let addr = addr & 0x3fff;
        match addr {
            0..=0x1fff => self.mapper.ppu_write(addr, v),
            0x2000..=0x3eff => {
                let i = self.nametable_index(addr);
                self.nametables[i] = v;
            }
            _ => self.palette[Self::palette_index(addr)] = v & 0x3f,
        }
    }
}

pub struct SystemBus {
    pub ppu: Ppu,
    pub apu: Apu,
    pub memory: PpuAddressSpace,
    pub controllers: [Controller; 2],
    /// Elapsed hardware cycles, including OAM DMA and DMC stalls.
    pub cycles: u64,
    ram: [u8; 2048],
    open_bus: u8,
    nmi_pending: bool,
    dma_page: Option<u8>,
}
impl SystemBus {
    pub fn new(cartridge: Cartridge, sample_rate: u32) -> Self {
        Self::with_mapper(Box::new(Nrom::new(cartridge)), sample_rate)
    }
    pub fn with_mapper(mapper: Box<dyn Mapper>, sample_rate: u32) -> Self {
        Self {
            ppu: Ppu::new(),
            apu: Apu::new(sample_rate),
            memory: PpuAddressSpace::new(mapper),
            controllers: [Controller::default(), Controller::default()],
            cycles: 0,
            ram: [0; 2048],
            open_bus: 0,
            nmi_pending: false,
            dma_page: None,
        }
    }
    pub fn reset_devices(&mut self, sample_rate: u32) {
        self.ppu = Ppu::new();
        self.apu = Apu::new(sample_rate);
        self.nmi_pending = false;
        self.dma_page = None;
        for controller in &mut self.controllers {
            controller.write(1);
            controller.write(0);
        }
    }
    fn clock(&mut self) {
        self.cycles += 1;
        self.apu.tick();
        self.memory.mapper.cpu_clock();
        for _ in 0..3 {
            self.ppu.tick(&mut self.memory);
            self.nmi_pending |= self.ppu.take_nmi();
        }
    }
    fn mapped_read(&mut self, addr: u16) -> u8 {
        let value = match addr {
            0..=0x1fff => self.ram[usize::from(addr & 0x7ff)],
            0x2000..=0x3fff => self.ppu.cpu_read(addr, &mut self.memory),
            0x4015 => self.apu.cpu_read(addr),
            0x4016 | 0x4017 => {
                (self.open_bus & 0xe0) | self.controllers[usize::from(addr & 1)].read()
            }
            0x4000..=0x4014 | 0x4018..=0x401f => self.open_bus,
            _ => self.memory.mapper.cpu_read(addr).unwrap_or(self.open_bus),
        };
        self.open_bus = value;
        value
    }
    fn read_cycle(&mut self, addr: u16) -> u8 {
        let value = self.mapped_read(addr);
        self.clock();
        value
    }
    fn service_dma(&mut self, halt_addr: u16) {
        if let Some(page) = self.dma_page.take() {
            // DMA halts on a CPU read cycle, not on consecutive RMW writes.
            let align = self.cycles & 1 != 0;
            self.read_cycle(halt_addr);
            if align {
                self.read_cycle(halt_addr);
            }
            for offset in 0..=255u16 {
                let byte = self.read_cycle((u16::from(page) << 8) | offset);
                self.ppu.oam_dma_write(byte);
                self.open_bus = byte;
                self.clock();
            }
        }
        // Initial DMC arbitration model: four cycles per fetch on next CPU read.
        // Exact DMA collision shortening and repeated controller reads are deferred.
        if let Some(addr) = self.apu.dmc_request() {
            for _ in 0..3 {
                self.read_cycle(halt_addr);
            }
            let sample = self.read_cycle(addr);
            self.apu.dmc_provide(sample);
        }
    }
}
impl CpuBus for SystemBus {
    fn read(&mut self, addr: u16) -> u8 {
        self.service_dma(addr);
        self.read_cycle(addr)
    }
    fn write(&mut self, addr: u16, v: u8) {
        self.open_bus = v;
        match addr {
            0..=0x1fff => self.ram[usize::from(addr & 0x7ff)] = v,
            0x2000..=0x3fff => self.ppu.cpu_write(addr, v, &mut self.memory),
            0x4014 => self.dma_page = Some(v),
            0x4016 => {
                for controller in &mut self.controllers {
                    controller.write(v);
                }
            }
            0x4000..=0x4013 | 0x4015 | 0x4017 => self.apu.cpu_write(addr, v),
            0x4018..=0x401f => {}
            _ => self.memory.mapper.cpu_write(addr, v),
        }
        self.clock();
    }
    fn take_nmi(&mut self) -> bool {
        std::mem::take(&mut self.nmi_pending)
    }
    fn irq_pending(&self) -> bool {
        self.apu.irq_pending() || self.memory.mapper.irq_pending()
    }
}
