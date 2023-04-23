use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub enum RamError {
	InvalidAddress(u16),
	UnmappedRegion(u16),
	WriteOnRom(u16),
}

impl RamError {
	pub(crate) fn adjust_for_offset(self, offset: u16) -> Self {
		match self {
			Self::InvalidAddress(address) => Self::InvalidAddress(address + offset),
			Self::UnmappedRegion(address) => Self::UnmappedRegion(address + offset),
			Self::WriteOnRom(address) => Self::WriteOnRom(address + offset),
		}
	}
}

impl Display for RamError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::UnmappedRegion(address) => {
				write!(f, "No mapped RAM region for {address:#06X}")
			}
			Self::InvalidAddress(address) => {
				write!(f, "Attempted to access invalid address {address:#06X}")
			}
			Self::WriteOnRom(address) => {
				write!(f, "Attempted write to ROM address {address:#06X}")
			}
		}
	}
}