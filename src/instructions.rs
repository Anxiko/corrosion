use crate::hardware::cpu::Cpu;
use crate::hardware::ram::RamError;

pub(crate) mod arithmetic;

trait Instruction {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), InstructionError>;
}

pub(crate) enum InstructionError {
	RamError(RamError)
}

impl From<RamError> for InstructionError {
	fn from(ram_error: RamError) -> Self {
		Self::RamError(ram_error)
	}
}