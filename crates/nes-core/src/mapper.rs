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

/// Fixed NROM board. PRG ROM is read-only; smaller RAM declarations mirror
/// within the 8 KiB window. Battery RAM persistence belongs to the frontend.
pub struct Nrom {
    cartridge: crate::cartridge::Cartridge,
    prg_ram: Vec<u8>,
}
impl Nrom {
    pub fn new(cartridge: crate::cartridge::Cartridge) -> Self {
        assert_eq!(cartridge.mapper, 0, "NROM requires mapper 0");
        assert!(matches!(cartridge.prg_rom.len(), 16384 | 32768));
        assert!(!cartridge.chr.is_empty());
        let size = cartridge.prg_ram_size + cartridge.prg_nvram_size;
        // Trainer boards require storage at $7000, even if no RAM is declared.
        let mut prg_ram = vec![
            0;
            if cartridge.trainer.is_some() {
                8192
            } else {
                size
            }
        ];
        if let Some(trainer) = &cartridge.trainer {
            prg_ram[0x1000..0x1200].copy_from_slice(trainer);
        }
        Self { cartridge, prg_ram }
    }
    pub fn prg_ram(&self) -> &[u8] {
        &self.prg_ram
    }
    pub fn prg_ram_mut(&mut self) -> &mut [u8] {
        &mut self.prg_ram
    }
}
impl Mapper for Nrom {
    fn cpu_read(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x6000..=0x7fff if !self.prg_ram.is_empty() => {
                Some(self.prg_ram[usize::from(addr - 0x6000) % self.prg_ram.len()])
            }
            0x8000..=0xffff => Some(
                self.cartridge.prg_rom[usize::from(addr - 0x8000) % self.cartridge.prg_rom.len()],
            ),
            _ => None,
        }
    }
    fn cpu_write(&mut self, addr: u16, v: u8) {
        if (0x6000..=0x7fff).contains(&addr) && !self.prg_ram.is_empty() {
            let i = usize::from(addr - 0x6000) % self.prg_ram.len();
            self.prg_ram[i] = v;
        }
    }
    fn ppu_read(&mut self, addr: u16) -> u8 {
        self.cartridge.chr[usize::from(addr & 0x1fff) % self.cartridge.chr.len()]
    }
    fn ppu_write(&mut self, addr: u16, v: u8) {
        if self.cartridge.chr_is_ram {
            let i = usize::from(addr & 0x1fff) % self.cartridge.chr.len();
            self.cartridge.chr[i] = v;
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.cartridge.mirroring
    }
}
