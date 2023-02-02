use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use crate::instructions::changeset::{Change, ChangesetInstruction, MemoryByteWriteChange, SingleRegisterChange};

use super::{ACC_REGISTER, ExecutionError};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum ByteSource {
	Acc,
	SingleRegister { single_reg: SingleRegisters },
	MemoryRegister { address_register: DoubleRegisters },
	MemoryImmediate { address_immediate: u16 },
	Immediate { value: u8 },
}

impl ByteSource {
	fn read_from_acc() -> Self {
		Self::Acc
	}

	fn read_from_single(single_reg: SingleRegisters) -> Self {
		Self::SingleRegister { single_reg }
	}

	fn read_from_register_address(address_register: DoubleRegisters) -> Self {
		Self::MemoryRegister {
			address_register,
		}
	}

	fn read_from_immediate_address(address_immediate: u16) -> Self {
		Self::MemoryImmediate { address_immediate }
	}

	fn read_from_immediate(value: u8) -> Self {
		Self::Immediate { value }
	}

	pub(super) fn read(&self, cpu: &Cpu) -> Result<u8, ExecutionError> {
		match self {
			Self::Acc => Ok(cpu.register_bank.read_single_named(ACC_REGISTER)),
			Self::SingleRegister { single_reg } => {
				Ok(cpu.register_bank.read_single_named(*single_reg))
			}
			Self::MemoryRegister { address_register } => {
				let address = cpu.register_bank.read_double_named(*address_register);
				let result = cpu.mapped_ram.read_byte(address)?;
				Ok(result)
			},
			Self::MemoryImmediate { address_immediate } => {
				let result = cpu.mapped_ram.read_byte(*address_immediate)?;
				Ok(result)
			},
			Self::Immediate { value } => Ok(*value),
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum ByteDestination {
	Acc,
	SingleRegister { single_reg: SingleRegisters },
	MemoryImmediate { address_immediate: u16 },
	MemoryRegister { double_reg: DoubleRegisters }
}

impl ByteDestination {
	fn write_to_acc() -> Self {
		Self::Acc
	}

	fn write_to_single(single_reg: SingleRegisters) -> Self {
		Self::SingleRegister { single_reg }
	}

	fn write_to_address_register(double_reg: DoubleRegisters) -> Self {
		Self::MemoryRegister { double_reg }
	}

	pub(super) fn change_destination(&self, value: u8) -> Box<dyn Change> {
		match self {
			Self::Acc => Box::new(SingleRegisterChange::new(ACC_REGISTER, value)),
			Self::SingleRegister { single_reg } => Box::new(SingleRegisterChange::new(*single_reg, value)),
			Self::MemoryImmediate { address_immediate } => {
				Box::new(MemoryByteWriteChange::write_to_immediate(*address_immediate, value))
			},
			Self::MemoryRegister { double_reg } => {
				Box::new(MemoryByteWriteChange::write_to_register(*double_reg, value))
			}
		}
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

#[derive(Debug, PartialEq, Copy, Clone)]
pub(super) enum DoubleByteSource {
	DoubleRegister(DoubleRegisters),
	Immediate(u16),
	StackPointer,
}

impl DoubleByteSource {
	fn read_from_double_register(double_register: DoubleRegisters) -> Self {
		Self::DoubleRegister(double_register)
	}

	fn read_from_immediate(immediate: u16) -> Self {
		Self::Immediate(immediate)
	}

	fn read_from_sp() -> Self {
		Self::StackPointer
	}

	pub(super) fn read(&self, cpu: &Cpu) -> Result<u16, ExecutionError> {
		match self {
			Self::DoubleRegister(double_register) => {
				Ok(cpu.register_bank.read_double_named(*double_register))
			},
			Self::Immediate(immediate) => {
				Ok(*immediate)
			},
			Self::StackPointer => {
				Ok(cpu.sp.read())
			}
		}
	}
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(super) enum DoubleByteDestination {
	DoubleRegister(DoubleRegisters),
	StackPointer,
	MemoryImmediate(u16),
}

impl DoubleByteDestination {
	fn write_to_double_register(double_register: DoubleRegisters) -> Self {
		Self::DoubleRegister(double_register)
	}

	fn write_to_sp() -> Self {
		Self::StackPointer
	}

	fn write_to_memory_immediate_address(address: u16) -> Self {
		Self::MemoryImmediate(address)
	}

	pub(super) fn change_destination(&self, value: u16) -> Result<Box<dyn Change>, ExecutionError> {
		todo!()
	}
}

pub(super) trait DoubleByteOperation {
	type C: Change;

	fn execute(
		&self,
		cpu: &Cpu,
		src: &DoubleByteSource,
		dst: &DoubleByteDestination,
	) -> Result<Self::C, ExecutionError>;
}

pub(super) struct BaseDoubleByteInstruction<O>
	where O: DoubleByteOperation {
	src: DoubleByteSource,
	dst: DoubleByteDestination,
	op: O,
}

impl<O> BaseDoubleByteInstruction<O> where O: DoubleByteOperation {
	fn new(src: DoubleByteSource, dst: DoubleByteDestination, op: O) -> Self {
		Self { src, dst, op }
	}
}

impl<O> ChangesetInstruction for BaseDoubleByteInstruction<O>
	where O: DoubleByteOperation {
	type C = O::C;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		self.op.execute(cpu, &self.src, &self.dst)
	}
}