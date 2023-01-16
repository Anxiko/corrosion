use crate::hardware::cpu::Cpu;
use crate::hardware::ram::RamError;

pub(crate) mod arithmetic;
pub(crate) mod logical;

trait Instruction {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError>;
}

#[derive(Debug)]
pub(crate) enum ExecutionError {
	RamError(RamError)
}

impl From<RamError> for ExecutionError {
	fn from(ram_error: RamError) -> Self {
		Self::RamError(ram_error)
	}
}