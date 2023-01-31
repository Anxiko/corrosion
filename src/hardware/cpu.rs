use crate::hardware::register_bank::{ProgramCounter, StackPointer};

use super::ram::MappedRam;
use super::register_bank::RegisterBank;

#[derive(Debug, PartialEq, Clone)]
pub struct Cpu {
	pub register_bank: RegisterBank,
	pub mapped_ram: MappedRam,
	pub(crate) pc: ProgramCounter,
	pub(crate) sp: StackPointer,
}

impl Cpu {
	pub fn new() -> Self {
		Self {
			register_bank: RegisterBank::new(),
			mapped_ram: MappedRam::new(),
			pc: ProgramCounter::new(),
			sp: StackPointer::new(),
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
