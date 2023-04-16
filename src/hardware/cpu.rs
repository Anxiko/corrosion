use crate::hardware::ime::Ime;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::{ProgramCounter, StackPointer};
use crate::instructions::ExecutionError;

use super::ram::MappedRam;
use super::register_bank::RegisterBank;

#[derive(Debug, PartialEq, Clone)]
pub struct Cpu {
	pub register_bank: RegisterBank,
	pub mapped_ram: MappedRam,
	pub(crate) pc: ProgramCounter,
	pub(crate) sp: StackPointer,
	pub(crate) ime: Ime,
}

impl Cpu {
	pub fn new() -> Self {
		Self {
			register_bank: RegisterBank::new(),
			mapped_ram: MappedRam::new(),
			pc: ProgramCounter::new(),
			sp: StackPointer::new(),
			ime: Ime::new(),
		}
	}

	pub(crate) fn next_pc(&mut self) -> u16 {
		let result = self.pc.read();
		self.pc.increment();

		result
	}

	pub(crate) fn next_byte(&mut self) -> Result<u8, ExecutionError> {
		let pc = self.next_pc();
		let byte = self.mapped_ram.read_byte(pc)?;
		Ok(byte)
	}
}

impl Default for Cpu {
	fn default() -> Self {
		Self::new()
	}
}
