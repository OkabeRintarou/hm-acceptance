#![allow(dead_code)]
/// Locally generated iNES fixture; no downloaded ROMs or copyrighted payloads.
pub fn rom(prg_banks: u8, chr_banks: u8) -> Vec<u8> {
    let mut bytes = vec![0; 16 + usize::from(prg_banks) * 16384 + usize::from(chr_banks) * 8192];
    bytes[..4].copy_from_slice(b"NES\x1a");
    bytes[4] = prg_banks;
    bytes[5] = chr_banks;
    bytes
}
pub fn test_program(program: &[u8]) -> Vec<u8> {
    let mut bytes = rom(1, 0);
    bytes[16..16 + program.len()].copy_from_slice(program);
    // All vectors point to $8000 in the mirrored 16 KiB ROM.
    for offset in [0x3ffa, 0x3ffc, 0x3ffe] {
        bytes[16 + offset..18 + offset].copy_from_slice(&0x8000u16.to_le_bytes());
    }
    bytes
}
