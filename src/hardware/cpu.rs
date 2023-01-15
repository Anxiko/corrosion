use super::register_bank::RegisterBank;

pub struct Cpu {
	pub register_bank: RegisterBank,
}

impl Cpu {
	pub fn new() -> Self {
		Self {
			register_bank: RegisterBank::new()
		}
	}
}

impl Default for Cpu {
	fn default() -> Self {
		Self::new()
	}
}