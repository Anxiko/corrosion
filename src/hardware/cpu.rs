use crate::hardware::register_bank::ProgramCounter;
use crate::instructions::ExecutionError;
use super::ram::MappedRam;
use super::register_bank::RegisterBank;

pub struct Cpu {
	pub register_bank: RegisterBank,
	pub mapped_ram: MappedRam,
	pub(crate) pc: ProgramCounter,
}

impl Cpu {
	pub fn new() -> Self {
		Self {
			register_bank: RegisterBank::new(),
			mapped_ram: MappedRam::new(),
			pc: ProgramCounter::new(),
		}
	}

	pub(crate) fn next_pc(&mut self) -> u16 {
		let result = self.pc.read();
		self.pc.increment();

		result
	}
}

impl Default for Cpu {
	fn default() -> Self {
		Self::new()
	}
}