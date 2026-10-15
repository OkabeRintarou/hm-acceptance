mod common;
use nes_core::{
    bus::{PpuAddressSpace, SystemBus},
    cpu::CpuBus,
    input::Controller,
    mapper::Nrom,
    ppu::PpuMemory,
    Cartridge, Console,
};

#[test]
fn ram_mirrors_open_bus_and_controller_routing() {
    let mut bus = SystemBus::new(Cartridge::from_bytes(&common::rom(1, 0)).unwrap(), 48000);
    bus.write(0x0005, 0xa5);
    for addr in [0x0005, 0x0805, 0x1005, 0x1805] {
        assert_eq!(bus.read(addr), 0xa5);
    }
    assert_eq!(bus.read(0x5000), 0xa5);
    bus.controllers[0].set_buttons(0x55);
    bus.controllers[1].set_buttons(0xaa);
    bus.write(0x4016, 1);
    bus.write(0x4016, 0);
    for i in 0..8 {
        assert_eq!(bus.read(0x4016) & 1, (0x55 >> i) & 1);
        assert_eq!(bus.read(0x4017) & 1, (0xaa >> i) & 1);
    }
    assert_eq!(bus.read(0x4016) & 1, 1);
    // $4017 writes target APU, not controller strobe.
    bus.write(0x4017, 1);
    assert_eq!(bus.read(0x4016) & 1, 1);
}
#[test]
fn strobe_reads_live_a_and_latches_on_release() {
    let mut c = Controller::default();
    c.set_buttons(0x81);
    c.write(1);
    assert_eq!(c.read(), 1);
    assert_eq!(c.read(), 1);
    c.set_buttons(0x82);
    assert_eq!(c.read(), 0);
    c.write(0);
    c.set_buttons(0);
    assert_eq!((0..8).map(|i| c.read() << i).fold(0, |a, b| a | b), 0x82);
    assert_eq!(c.read(), 1);
}
#[test]
fn oam_dma_stalls_513_or_514_cycles_and_waits_for_read() {
    for precycles in [0, 1] {
        let mut bus = SystemBus::new(Cartridge::from_bytes(&common::rom(1, 0)).unwrap(), 48000);
        for _ in 0..precycles {
            bus.read(0);
        }
        bus.write(0x4014, 0x00);
        let before = bus.cycles;
        bus.read(0x8000);
        let stall = if before & 1 == 0 { 513 } else { 514 };
        assert_eq!(bus.cycles - before, stall + 1);
    }
    let mut bus = SystemBus::new(Cartridge::from_bytes(&common::rom(1, 0)).unwrap(), 48000);
    bus.write(0x4014, 0);
    bus.write(0x4014, 1);
    assert_eq!(bus.cycles, 2); // consecutive RMW writes do not trigger two DMAs
    bus.read(0);
    assert_eq!(bus.cycles, 516);
}
#[test]
fn nametable_and_palette_mirroring() {
    for (flags, aliases) in [(0, [0, 0, 1, 1]), (1, [0, 1, 0, 1]), (8, [0, 1, 2, 3])] {
        let mut bytes = common::rom(1, 0);
        bytes[6] = flags;
        let mut mem =
            PpuAddressSpace::new(Box::new(Nrom::new(Cartridge::from_bytes(&bytes).unwrap())));
        for table in 0..4 {
            mem.write(0x2000 + table * 1024, 10 + aliases[usize::from(table)]);
        }
        for table in 0..4 {
            assert_eq!(
                mem.read(0x2000 + table * 1024),
                10 + aliases[usize::from(table)]
            );
        }
        mem.write(0x2001, 0x53);
        assert_eq!(mem.read(0x3001), 0x53);
        mem.write(0x3f10, 0xff);
        assert_eq!(mem.read(0x3f00), 0x3f);
        assert_eq!(mem.read(0x7f30), 0x3f);
        mem.write(0x3f04, 0x21);
        assert_eq!(mem.read(0x3f14), 0x21);
    }
}
#[test]
fn generated_rom_executes_through_console_and_advances_frame() {
    // LDA #$42; STA $00; INC $00; JMP $8004. No external binary fixtures.
    let bytes = common::test_program(&[0xa9, 0x42, 0x85, 0x00, 0xe6, 0x00, 0x4c, 0x04, 0x80]);
    let mut console = Console::new(Cartridge::from_bytes(&bytes).unwrap(), 48000);
    assert_eq!(console.cpu.pc, 0x8000);
    assert_eq!(console.cpu.sp, 0xfd);
    assert_eq!(console.bus.cycles, 7);
    let before = console.bus.cycles;
    console.step_frame();
    assert!(console.bus.cycles - before > 27000);
    assert!(console.bus.cycles - before < 31000);
    assert_eq!(console.framebuffer().len(), 256 * 240 * 4);
    let old_sp = console.cpu.sp;
    console.reset();
    assert_eq!(console.cpu.sp, old_sp.wrapping_sub(3));
    assert_eq!(console.cpu.pc, 0x8000);
}
#[test]
fn ntsc_only_execution_is_enforced() {
    let mut bytes = common::test_program(&[0x4c, 0, 0x80]);
    bytes[9] = 1;
    assert!(std::panic::catch_unwind(|| Console::new(
        Cartridge::from_bytes(&bytes).unwrap(),
        48000
    ))
    .is_err());
}

#[test]
fn dma_fetches_all_256_bytes_and_mapper_clocks_every_stall_cycle() {
    use nes_core::mapper::{Mapper, Mirroring};
    use std::{cell::RefCell, rc::Rc};
    struct Spy {
        reads: Rc<RefCell<Vec<u16>>>,
        clocks: Rc<RefCell<u64>>,
    }
    impl Mapper for Spy {
        fn cpu_read(&mut self, addr: u16) -> Option<u8> {
            self.reads.borrow_mut().push(addr);
            Some(addr as u8)
        }
        fn cpu_write(&mut self, _addr: u16, _v: u8) {}
        fn ppu_read(&mut self, _addr: u16) -> u8 {
            0
        }
        fn ppu_write(&mut self, _addr: u16, _v: u8) {}
        fn mirroring(&self) -> Mirroring {
            Mirroring::Horizontal
        }
        fn cpu_clock(&mut self) {
            *self.clocks.borrow_mut() += 1;
        }
    }
    let reads = Rc::new(RefCell::new(Vec::new()));
    let clocks = Rc::new(RefCell::new(0));
    let mut bus = SystemBus::with_mapper(
        Box::new(Spy {
            reads: reads.clone(),
            clocks: clocks.clone(),
        }),
        48000,
    );
    bus.write(0x4014, 0x80);
    bus.read(0);
    assert_eq!(*reads.borrow(), (0x8000..=0x80ff).collect::<Vec<u16>>());
    assert_eq!(*clocks.borrow(), bus.cycles);
}
