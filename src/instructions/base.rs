use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use crate::instructions::changeset::{Change, ChangesetInstruction, SingleRegisterChange};

use super::{ACC_REGISTER, ExecutionError};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum ByteSource {
	Acc,
	SingleRegister { single_reg: SingleRegisters },
	Memory { address_register: DoubleRegisters },
}

impl ByteSource {
	fn read_from_acc() -> Self {
		Self::Acc
	}

	fn read_from_single(single_reg: SingleRegisters) -> Self {
		Self::SingleRegister { single_reg }
	}

	fn read_from_hl_address() -> Self {
		Self::Memory {
			address_register: DoubleRegisters::HL,
		}
	}

	pub(super) fn read(&self, cpu: &Cpu) -> Result<u8, ExecutionError> {
		match self {
			Self::Acc => Ok(cpu.register_bank.read_single_named(ACC_REGISTER)),
			Self::SingleRegister { single_reg } => {
				Ok(cpu.register_bank.read_single_named(*single_reg))
			}
			Self::Memory { address_register } => {
				let address = cpu.register_bank.read_double_named(*address_register);
				let result = cpu.mapped_ram.read(address)?;
				Ok(result)
			}
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum ByteDestination {
	Acc,
	SingleRegister { single_reg: SingleRegisters },
}

impl ByteDestination {
	fn write_to_acc() -> Self {
		Self::Acc
	}

	fn write_to_single(single_reg: SingleRegisters) -> Self {
		Self::SingleRegister { single_reg }
	}

	pub(super) fn change_destination(&self, value: u8) -> SingleRegisterChange {
		let reg = match self {
			Self::Acc => ACC_REGISTER,
			Self::SingleRegister { single_reg } => *single_reg,
		};

		SingleRegisterChange::new(reg, value)
	}
}

pub(super) trait ByteOperation {
	type C: Change;

	fn execute(
		&self,
		cpu: &Cpu,
		src: &ByteSource,
		dst: &ByteDestination,
	) -> Result<Self::C, ExecutionError>;
}

pub(super) struct BaseByteInstruction<O>
	where
		O: ByteOperation,
{
	src: ByteSource,
	dst: ByteDestination,
	op: O,
}

impl<O> BaseByteInstruction<O>
	where
		O: ByteOperation,
{
	pub(super) fn new(src: ByteSource, dst: ByteDestination, op: O) -> Self {
		Self { src, dst, op }
	}
}

impl<O> ChangesetInstruction for BaseByteInstruction<O>
	where
		O: ByteOperation,
{
	type C = O::C;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		self.op.execute(cpu, &self.src, &self.dst)
	}
}
