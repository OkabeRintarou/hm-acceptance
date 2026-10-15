mod common;
use common::rom;
use nes_core::{
    cartridge::{Cartridge, CartridgeError as E, ConsoleType, Format, Timing},
    mapper::{Mapper, Mirroring, Nrom},
};

#[test]
fn ines_defaults_and_metadata() {
    let mut bytes = rom(1, 0);
    bytes[6] = 3;
    let c = Cartridge::from_bytes(&bytes).unwrap();
    assert_eq!(c.format, Format::INes);
    assert_eq!(c.mapper, 0);
    assert_eq!(c.submapper, 0);
    assert_eq!(c.prg_rom.len(), 16384);
    assert_eq!(c.chr.len(), 8192);
    assert!(c.chr_is_ram);
    assert_eq!(c.mirroring, Mirroring::Vertical);
    assert!(c.battery);
    assert_eq!(c.prg_ram_size, 0);
    assert_eq!(c.prg_nvram_size, 8192);
    assert_eq!(c.chr_ram_size, 8192);
    assert_eq!(c.console_type, ConsoleType::Nes);
    assert_eq!(c.timing, Timing::Ntsc);
}
#[test]
fn trainer_is_separated_and_loaded_at_7000() {
    let mut bytes = rom(2, 1);
    bytes[6] = 0x0c;
    bytes.splice(16..16, [0xa5; 512]);
    bytes[16 + 512] = 0x42;
    let c = Cartridge::from_bytes(&bytes).unwrap();
    assert_eq!(c.mirroring, Mirroring::FourScreen);
    assert_eq!(c.trainer.as_ref().unwrap().len(), 512);
    assert_eq!(c.prg_rom[0], 0x42);
    let mut m = Nrom::new(c);
    assert_eq!(m.cpu_read(0x7000), Some(0xa5));
    assert_eq!(m.cpu_read(0x71ff), Some(0xa5));
}
#[test]
fn nes2_linear_sizes_and_ram_shifts() {
    let mut bytes = rom(2, 0);
    bytes[7] = 8;
    bytes[10] = 0x61;
    bytes[11] = 7;
    bytes[12] = 2;
    let c = Cartridge::from_bytes(&bytes).unwrap();
    assert_eq!(c.format, Format::Nes2);
    assert_eq!(c.prg_rom.len(), 32768);
    assert_eq!(c.prg_ram_size, 128);
    assert_eq!(c.prg_nvram_size, 4096);
    assert_eq!(c.chr_ram_size, 8192);
    assert_eq!(c.timing, Timing::MultiRegion);
}
#[test]
fn nes2_exponent_sizes() {
    let mut bytes = rom(1, 1);
    bytes[7] = 8;
    bytes[9] = 0xff;
    bytes[4] = 14 << 2;
    bytes[5] = 13 << 2;
    let c = Cartridge::from_bytes(&bytes).unwrap();
    assert_eq!(c.prg_rom.len(), 16384);
    assert_eq!(c.chr.len(), 8192);
    assert!(!c.chr_is_ram);
    // E=13, M=1 -> 2^13 * 3 bytes, correctly decoded then rejected as non-NROM.
    bytes[4] = (13 << 2) | 1;
    bytes.resize(16 + 24576 + 8192, 0);
    assert!(matches!(
        Cartridge::from_bytes(&bytes),
        Err(E::UnsupportedNromLayout(_))
    ));
}
#[test]
fn nes2_msb_and_exponent_overflow_are_checked() {
    let mut bytes = rom(1, 1);
    bytes[7] = 8;
    bytes[9] = 1;
    assert!(matches!(Cartridge::from_bytes(&bytes), Err(E::Truncated)));
    bytes[9] = 15;
    bytes[4] = 255;
    assert!(matches!(
        Cartridge::from_bytes(&bytes),
        Err(E::SizeOverflow)
    ));
}
#[test]
fn malformed_and_unsupported_headers() {
    assert!(matches!(Cartridge::from_bytes(&[]), Err(E::Truncated)));
    assert!(matches!(
        Cartridge::from_bytes(&[0; 16]),
        Err(E::InvalidMagic)
    ));
    let mut bytes = rom(1, 1);
    assert!(matches!(
        Cartridge::from_bytes(&bytes[..bytes.len() - 1]),
        Err(E::Truncated)
    ));
    bytes[6] = 4;
    assert!(matches!(Cartridge::from_bytes(&bytes), Err(E::Truncated)));
    bytes[6] = 0x10;
    assert!(matches!(
        Cartridge::from_bytes(&bytes),
        Err(E::UnsupportedMapper(1))
    ));
    bytes[6] = 0;
    bytes[7] = 8;
    bytes[8] = 3;
    assert!(matches!(
        Cartridge::from_bytes(&bytes),
        Err(E::UnsupportedMapper(768))
    ));
    bytes[8] = 0x10;
    assert!(matches!(
        Cartridge::from_bytes(&bytes),
        Err(E::UnsupportedNromLayout(_))
    ));
    bytes[7] = 4;
    assert!(matches!(
        Cartridge::from_bytes(&bytes),
        Err(E::UnsupportedFormat)
    ));
}
#[test]
fn regions_and_console_types_are_exposed() {
    let mut bytes = rom(1, 1);
    bytes[9] = 1;
    assert_eq!(Cartridge::from_bytes(&bytes).unwrap().timing, Timing::Pal);
    bytes[7] = 9;
    bytes[9] = 0;
    bytes[12] = 3;
    let c = Cartridge::from_bytes(&bytes).unwrap();
    assert_eq!(c.timing, Timing::Dendy);
    assert_eq!(c.console_type, ConsoleType::VsSystem);
    bytes[7] = 10;
    assert_eq!(
        Cartridge::from_bytes(&bytes).unwrap().console_type,
        ConsoleType::PlayChoice10
    );
    bytes[7] = 11;
    bytes[13] = 7;
    assert_eq!(
        Cartridge::from_bytes(&bytes).unwrap().console_type,
        ConsoleType::Extended(7)
    );
}
#[test]
fn nrom_16k_mirrors_32k_does_not_and_rom_ignores_writes() {
    for banks in [1, 2] {
        let mut bytes = rom(banks, 1);
        bytes[16] = 0x11;
        if banks == 2 {
            bytes[16 + 16384] = 0x22;
        }
        let mut m = Nrom::new(Cartridge::from_bytes(&bytes).unwrap());
        assert_eq!(m.cpu_read(0x8000), Some(0x11));
        assert_eq!(
            m.cpu_read(0xc000),
            Some(if banks == 1 { 0x11 } else { 0x22 })
        );
        m.cpu_write(0x8000, 0xff);
        assert_eq!(m.cpu_read(0x8000), Some(0x11));
        assert_eq!(m.cpu_read(0x5000), None);
        m.cpu_write(0x6000, 0x53);
        m.cpu_write(0x7fff, 0x29);
        assert_eq!(m.cpu_read(0x6000), Some(0x53));
        assert_eq!(m.cpu_read(0x7fff), Some(0x29));
        m.ppu_write(0, 0xff);
        assert_eq!(m.ppu_read(0), 0);
    }
}
#[test]
fn chr_ram_is_writable_and_optional_nes2_prg_ram_is_unmapped() {
    let mut bytes = rom(1, 0);
    bytes[7] = 8;
    bytes[11] = 7;
    let mut m = Nrom::new(Cartridge::from_bytes(&bytes).unwrap());
    assert_eq!(m.cpu_read(0x6000), None);
    m.cpu_write(0x6000, 9);
    assert_eq!(m.cpu_read(0x6000), None);
    m.ppu_write(0x1fff, 0x99);
    assert_eq!(m.ppu_read(0x1fff), 0x99);
}
#[test]
fn nonsensical_nrom_sizes_are_typed_errors() {
    assert!(matches!(
        Cartridge::from_bytes(&rom(0, 1)),
        Err(E::UnsupportedNromLayout(_))
    ));
    assert!(matches!(
        Cartridge::from_bytes(&rom(3, 1)),
        Err(E::UnsupportedNromLayout(_))
    ));
    assert!(matches!(
        Cartridge::from_bytes(&rom(1, 2)),
        Err(E::UnsupportedNromLayout(_))
    ));
    let mut bytes = rom(1, 0);
    bytes[7] = 8;
    assert!(matches!(
        Cartridge::from_bytes(&bytes),
        Err(E::UnsupportedNromLayout(_))
    ));
    bytes[11] = 8;
    assert!(matches!(
        Cartridge::from_bytes(&bytes),
        Err(E::UnsupportedNromLayout(_))
    ));
}
