use crate::hardware::cpu::Cpu;
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::changeset::{ChangeIme, ChangesetInstruction};

pub(crate) struct NopInstruction {}

impl NopInstruction {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl Instruction for NopInstruction {
	fn execute(&self, _cpu: &mut Cpu) -> Result<(), ExecutionError> {
		Ok(())
	}
}

pub(crate) struct StopInstruction {}

impl StopInstruction {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl Instruction for StopInstruction {
	fn execute(&self, _cpu: &mut Cpu) -> Result<(), ExecutionError> {
		todo!("Implement STOP instruction")
	}
}

pub(crate) struct HaltInstruction;

impl HaltInstruction {
	pub(crate) fn new() -> Self {
		Self
	}
}

impl Instruction for HaltInstruction {
	fn execute(&self, _cpu: &mut Cpu) -> Result<(), ExecutionError> {
		todo!("Implement HALT instruction")
	}
}

pub(crate) struct SetImeInstruction {
	value: bool,
}

impl SetImeInstruction {
	pub(crate) fn new(value: bool) -> Self {
		Self { value }
	}
}

impl ChangesetInstruction for SetImeInstruction {
	type C = ChangeIme;

	fn compute_change(&self, _cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		Ok(ChangeIme::new(self.value))
	}
}