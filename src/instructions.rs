use crate::hardware::cpu::Cpu;
use crate::hardware::ram::RamError;
use crate::hardware::register_bank::SingleRegisters;

pub(crate) mod arithmetic;
pub(crate) mod base;
pub(crate) mod changeset;
pub(crate) mod flags;
pub(crate) mod logical;
pub(crate) mod shifting;
pub(crate) mod load;
pub(crate) mod single_bit;
pub(crate) mod control;
pub(crate) mod jump;

pub(crate) trait Instruction {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError>;
}

#[derive(Debug)]
pub(crate) enum ExecutionError {
	RamError(RamError),
}

impl From<RamError> for ExecutionError {
	fn from(ram_error: RamError) -> Self {
		Self::RamError(ram_error)
	}
}

pub(crate) const ACC_REGISTER: SingleRegisters = SingleRegisters::A;
