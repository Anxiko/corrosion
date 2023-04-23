use crate::hardware::ime::Ime;
use crate::hardware::ram::Rom;
use crate::hardware::register_bank::{ProgramCounter, StackPointer};
use crate::instructions::ExecutionError;

use super::ram::MappedMemory;
use super::register_bank::RegisterBank;

#[derive(Debug, PartialEq, Clone)]
pub struct Cpu {
	pub(crate) register_bank: RegisterBank,
	pub(crate) mapped_ram: MappedMemory,
	pub(crate) pc: ProgramCounter,
	pub(crate) sp: StackPointer,
	pub(crate) ime: Ime,
}

impl Cpu {
	pub fn new() -> Self {
		Self {
			register_bank: RegisterBank::new(),
			mapped_ram: MappedMemory::new(),
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

	pub fn current_pc(&self) -> u16 {
		self.pc.read()
	}
}

impl Default for Cpu {
	fn default() -> Self {
		Self::new()
	}
}
