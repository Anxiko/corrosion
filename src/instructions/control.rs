use crate::hardware::cpu::Cpu;
use crate::instructions::{ExecutionError, Instruction};

pub(crate) struct NopInstruction {}

impl NopInstruction {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl Instruction for NopInstruction {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		Ok(())
	}
}