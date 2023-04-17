use super::BOOTSTRAP_RAM_SIZE;

pub(super) static BOOTSTRAP_DATA: &[u8; BOOTSTRAP_RAM_SIZE] = include_bytes!("DMG_ROM.bin");
