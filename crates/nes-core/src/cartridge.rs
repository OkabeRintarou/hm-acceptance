//! iNES / NES 2.0 container parsing, independent of file I/O.
use crate::mapper::Mirroring;
use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    INes,
    Nes2,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timing {
    Ntsc,
    Pal,
    MultiRegion,
    Dendy,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleType {
    Nes,
    VsSystem,
    PlayChoice10,
    Extended(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CartridgeError {
    InvalidMagic,
    Truncated,
    UnsupportedMapper(u16),
    UnsupportedFormat,
    SizeOverflow,
    UnsupportedNromLayout(&'static str),
}
impl fmt::Display for CartridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => f.write_str("invalid NES file magic"),
            Self::Truncated => f.write_str("truncated NES header, trainer, or ROM payload"),
            Self::UnsupportedMapper(n) => {
                write!(f, "unsupported mapper {n}; only NROM (0) is supported")
            }
            Self::UnsupportedFormat => f.write_str("unrecognized iNES header format"),
            Self::SizeOverflow => f.write_str("ROM size cannot be represented on this platform"),
            Self::UnsupportedNromLayout(reason) => write!(f, "unsupported NROM layout: {reason}"),
        }
    }
}
impl Error for CartridgeError {}

/// Container metadata is exposed even for non-NTSC images. Console execution
/// accepts only standard NES NTSC (or multi-region) images, not PAL/Dendy/VS.
#[derive(Debug, Clone)]
pub struct Cartridge {
    pub format: Format,
    pub mapper: u16,
    pub submapper: u8,
    pub prg_rom: Vec<u8>,
    /// CHR ROM bytes, or zero-initialized CHR RAM when `chr_is_ram`.
    pub chr: Vec<u8>,
    pub chr_is_ram: bool,
    pub mirroring: Mirroring,
    pub battery: bool,
    pub trainer: Option<Vec<u8>>,
    pub prg_ram_size: usize,
    pub prg_nvram_size: usize,
    pub chr_ram_size: usize,
    pub chr_nvram_size: usize,
    pub console_type: ConsoleType,
    pub timing: Timing,
}

impl Cartridge {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CartridgeError> {
        use CartridgeError::*;
        if bytes.len() < 16 {
            return Err(Truncated);
        }
        if &bytes[..4] != b"NES\x1a" {
            return Err(InvalidMagic);
        }
        let h = &bytes[..16];
        let format = match h[7] & 0x0c {
            0 => Format::INes,
            8 => Format::Nes2,
            _ => return Err(UnsupportedFormat),
        };
        let nes2 = format == Format::Nes2;
        // Dirty iNES headers are tolerated; byte 7 mapper bits are still used.
        let mapper = u16::from(h[6] >> 4)
            | u16::from(h[7] & 0xf0)
            | if nes2 { u16::from(h[8] & 15) << 8 } else { 0 };
        if mapper != 0 {
            return Err(UnsupportedMapper(mapper));
        }
        let submapper = if nes2 { h[8] >> 4 } else { 0 };
        let prg_size = rom_size(h[4], if nes2 { h[9] & 15 } else { 0 }, 16 * 1024)?;
        let chr_size = rom_size(h[5], if nes2 { h[9] >> 4 } else { 0 }, 8 * 1024)?;
        let battery = h[6] & 2 != 0;
        let (prg_ram_size, prg_nvram_size, chr_ram_size, chr_nvram_size) = if nes2 {
            (
                ram_size(h[10] & 15),
                ram_size(h[10] >> 4),
                ram_size(h[11] & 15),
                ram_size(h[11] >> 4),
            )
        } else {
            let ram = usize::from(h[8].max(1)) * 8 * 1024;
            (
                if battery { 0 } else { ram },
                if battery { ram } else { 0 },
                if chr_size == 0 { 8192 } else { 0 },
                0,
            )
        };
        let trainer_size = if h[6] & 4 != 0 { 512 } else { 0 };
        let prg_start: usize = 16 + trainer_size;
        let chr_start = prg_start.checked_add(prg_size).ok_or(SizeOverflow)?;
        let end = chr_start.checked_add(chr_size).ok_or(SizeOverflow)?;
        if bytes.len() < end {
            return Err(Truncated);
        }
        // Container sizes are parsed fully before validating the board layout.
        if submapper != 0 {
            return Err(UnsupportedNromLayout("nonzero submapper"));
        }
        if !matches!(prg_size, 16384 | 32768) {
            return Err(UnsupportedNromLayout("PRG ROM must be 16 or 32 KiB"));
        }
        if !matches!(chr_size, 0 | 8192) {
            return Err(UnsupportedNromLayout("CHR ROM must be absent or 8 KiB"));
        }
        if prg_ram_size + prg_nvram_size > 8192 {
            return Err(UnsupportedNromLayout("PRG RAM exceeds 8 KiB"));
        }
        let chr_ram = chr_ram_size + chr_nvram_size;
        if chr_size == 0 && (chr_ram == 0 || chr_ram > 8192) {
            return Err(UnsupportedNromLayout(
                "CHR RAM must be 1..8192 bytes when CHR ROM is absent",
            ));
        }
        if chr_size != 0 && chr_ram != 0 {
            return Err(UnsupportedNromLayout("simultaneous CHR ROM and RAM"));
        }
        let console_type = match h[7] & 3 {
            0 => ConsoleType::Nes,
            1 => ConsoleType::VsSystem,
            2 => ConsoleType::PlayChoice10,
            _ => ConsoleType::Extended(if nes2 { h[13] & 15 } else { 0 }),
        };
        let timing = if nes2 {
            match h[12] & 3 {
                0 => Timing::Ntsc,
                1 => Timing::Pal,
                2 => Timing::MultiRegion,
                _ => Timing::Dendy,
            }
        } else if h[9] & 1 != 0 {
            Timing::Pal
        } else {
            Timing::Ntsc
        };
        Ok(Self {
            format,
            mapper,
            submapper,
            prg_rom: bytes[prg_start..chr_start].to_vec(),
            chr: if chr_size == 0 {
                vec![0; chr_ram]
            } else {
                bytes[chr_start..end].to_vec()
            },
            chr_is_ram: chr_size == 0,
            mirroring: if h[6] & 8 != 0 {
                Mirroring::FourScreen
            } else if h[6] & 1 != 0 {
                Mirroring::Vertical
            } else {
                Mirroring::Horizontal
            },
            battery,
            trainer: if trainer_size != 0 {
                Some(bytes[16..prg_start].to_vec())
            } else {
                None
            },
            prg_ram_size,
            prg_nvram_size,
            chr_ram_size,
            chr_nvram_size,
            console_type,
            timing,
        })
    }
}

fn ram_size(shift: u8) -> usize {
    if shift == 0 {
        0
    } else {
        64usize << shift
    }
}
fn rom_size(lsb: u8, msb: u8, unit: usize) -> Result<usize, CartridgeError> {
    if msb == 15 {
        let base = 1usize
            .checked_shl(u32::from(lsb >> 2))
            .ok_or(CartridgeError::SizeOverflow)?;
        base.checked_mul(usize::from((lsb & 3) * 2 + 1))
            .ok_or(CartridgeError::SizeOverflow)
    } else {
        ((usize::from(msb) << 8) | usize::from(lsb))
            .checked_mul(unit)
            .ok_or(CartridgeError::SizeOverflow)
    }
}
