use std::fmt::{Display, Formatter};

use crate::hardware::cpu::Cpu;
use crate::instructions::{Executable, ExecutionError};
use crate::instructions::changeset::{ChangeIme, ChangesetExecutable};

#[derive(Debug)]
pub(crate) struct NopInstruction {}

impl NopInstruction {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl Executable for NopInstruction {
	fn execute(&self, _cpu: &mut Cpu) -> Result<(), ExecutionError> {
		Ok(())
	}
}

impl Display for NopInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "nop")
	}
}

#[derive(Debug)]
pub(crate) struct StopInstruction {}

impl StopInstruction {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl Executable for StopInstruction {
	fn execute(&self, _cpu: &mut Cpu) -> Result<(), ExecutionError> {
		todo!("Implement STOP instruction")
	}
}

impl Display for StopInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "stop")
	}
}

#[derive(Debug)]
pub(crate) struct HaltInstruction;

impl HaltInstruction {
	pub(crate) fn new() -> Self {
		Self
	}
}

impl Executable for HaltInstruction {
	fn execute(&self, _cpu: &mut Cpu) -> Result<(), ExecutionError> {
		todo!("Implement HALT instruction")
	}
}

impl Display for HaltInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "halt")
	}
}

#[derive(Debug)]
pub(crate) struct SetImeInstruction {
	value: bool,
}

impl SetImeInstruction {
	pub(crate) fn new(value: bool) -> Self {
		Self { value }
	}
}

impl SetImeInstruction {
	fn as_str(&self) -> &str {
		match self.value {
			true => "ei",
			false => "di"
		}
	}
}

impl ChangesetExecutable for SetImeInstruction {
	type C = ChangeIme;

	fn compute_change(&self, _cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		Ok(ChangeIme::new(self.value))
	}
}

impl Display for SetImeInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let str = self.as_str();
		write!(f, "{str}")
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::cpu::Cpu;

	use super::*;

	#[test]
	fn change_ime() {
		let cpu = Cpu::new();

		let actual = SetImeInstruction::new(true).compute_change(&cpu).unwrap();
		let expected = ChangeIme::new(true);

		assert_eq!(actual, expected);
	}
}
