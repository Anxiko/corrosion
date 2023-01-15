use super::ram::MappedRam;
use super::register_bank::RegisterBank;

pub struct Cpu {
	pub register_bank: RegisterBank,
	pub mapped_ram: MappedRam,
}

impl Cpu {
	pub fn new() -> Self {
		Self {
			register_bank: RegisterBank::new(),
			mapped_ram: MappedRam::new(),
		}
	}
}

impl Default for Cpu {
	fn default() -> Self {
		Self::new()
	}
}