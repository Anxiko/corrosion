use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use crate::hardware::cpu::Cpu;
use crate::hardware::ram::RamError;
use crate::hardware::register_bank::SingleRegisters;

pub(crate) mod arithmetic;
pub(crate) mod base;
pub(crate) mod changeset;
pub(crate) mod control;
pub(crate) mod double_arithmetic;
pub(crate) mod flags;
pub(crate) mod flow;
pub(crate) mod load;
pub(crate) mod logical;
pub(crate) mod shared;
pub(crate) mod shifting;
pub(crate) mod single_bit;

pub trait Executable {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError>;
}

pub trait Instruction: Executable + Debug + Display {}

impl<T> Instruction for T where T: Executable + Debug + Display {}

#[derive(Debug)]
pub enum ExecutionError {
	RamError(RamError),
	InvalidOpcode(u8),
}

impl Display for ExecutionError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::RamError(ram_error) => write!(f, "{ram_error}"),
			Self::InvalidOpcode(opcode) => write!(f, "Invalid opcode {opcode:#06X}"),
		}
	}
}

impl Error for ExecutionError {}

impl From<RamError> for ExecutionError {
	fn from(ram_error: RamError) -> Self {
		Self::RamError(ram_error)
	}
}

pub(crate) const ACC_REGISTER: SingleRegisters = SingleRegisters::A;
