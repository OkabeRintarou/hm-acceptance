use crate::mapper::Mirroring;

#[derive(Debug)]
pub enum CartridgeError {
    InvalidMagic,
    Truncated,
    UnsupportedMapper(u16),
    UnsupportedFormat,
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr: Vec<u8>,
    pub mirroring: Mirroring,
}

impl Cartridge {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CartridgeError> {
        if bytes.len() < 16 {
            return Err(CartridgeError::Truncated);
        }
        if &bytes[..4] != b"NES\x1a" {
            return Err(CartridgeError::InvalidMagic);
        }
        Err(CartridgeError::UnsupportedFormat)
    }
}
