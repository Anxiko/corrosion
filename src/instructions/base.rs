use crate::hardware::alu::add_u8;
use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use crate::instructions::changeset::{BitFlagsChange, Change, ChangeList, ChangesetInstruction, DoubleRegisterChange, MemoryByteWriteChange, MemoryDoubleByteWriteChange, SingleRegisterChange, SpChange};

use super::{ACC_REGISTER, ExecutionError};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ByteSource {
	Acc,
	SingleRegister(SingleRegisters),
	AddressInRegister(DoubleRegisters),
	AddressInImmediate(u16),
	Immediate(u8),
}

impl ByteSource {
	fn read_from_acc() -> Self {
		Self::Acc
	}

	pub(crate) fn read_from_single(single_reg: SingleRegisters) -> Self {
		Self::SingleRegister(single_reg)
	}

	pub(crate) fn read_from_register_address(address_register: DoubleRegisters) -> Self {
		Self::AddressInRegister(address_register)
	}

	fn read_from_immediate_address(address_immediate: u16) -> Self {
		Self::AddressInImmediate(address_immediate)
	}

	fn read_from_immediate(value: u8) -> Self {
		Self::Immediate(value)
	}

	pub(super) fn read(&self, cpu: &Cpu) -> Result<u8, ExecutionError> {
		match self {
			Self::Acc => Ok(cpu.register_bank.read_single_named(ACC_REGISTER)),
			Self::SingleRegister(single_reg) => {
				Ok(cpu.register_bank.read_single_named(*single_reg))
			}
			Self::AddressInRegister(address_register) => {
				let address = cpu.register_bank.read_double_named(*address_register);
				let result = cpu.mapped_ram.read_byte(address)?;
				Ok(result)
			},
			Self::AddressInImmediate(address_immediate) => {
				let result = cpu.mapped_ram.read_byte(*address_immediate)?;
				Ok(result)
			},
			Self::Immediate(value) => Ok(*value),
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ByteDestination {
	Acc,
	SingleRegister(SingleRegisters),
	MemoryImmediate(u16),
	AddressInRegister(DoubleRegisters)
}

impl ByteDestination {
	fn write_to_acc() -> Self {
		Self::Acc
	}

	fn write_to_single(single_reg: SingleRegisters) -> Self {
		Self::SingleRegister(single_reg)
	}

	fn write_to_address_register(double_reg: DoubleRegisters) -> Self {
		Self::AddressInRegister(double_reg)
	}

	pub(super) fn change_destination(&self, value: u8) -> Box<dyn Change> {
		match self {
			Self::Acc => Box::new(SingleRegisterChange::new(ACC_REGISTER, value)),
			Self::SingleRegister(single_reg) => Box::new(SingleRegisterChange::new(*single_reg, value)),
			Self::MemoryImmediate(address_immediate) => {
				Box::new(MemoryByteWriteChange::write_to_immediate(*address_immediate, value))
			},
			Self::AddressInRegister(double_reg) => {
				Box::new(MemoryByteWriteChange::write_to_register(*double_reg, value))
			}
		}
	}
}

pub(crate) trait ByteOperation {
	type C: Change;

	fn execute(
		&self,
		cpu: &Cpu,
		src: &ByteSource,
		dst: &ByteDestination,
	) -> Result<Self::C, ExecutionError>;
}

pub(crate) struct BaseByteInstruction<O>
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
	pub(crate) fn new(src: ByteSource, dst: ByteDestination, op: O) -> Self {
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
pub(crate) enum DoubleByteSource {
	DoubleRegister(DoubleRegisters),
	Immediate(u16),
	StackPointer,
	AddressInRegister(DoubleRegisters)
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
			},
			Self::AddressInRegister(double_register) => {
				let address = cpu.register_bank.read_double_named(*double_register);
				let value = cpu.mapped_ram.read_double_byte(address)?;
				Ok(value)
			}
		}
	}
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum DoubleByteDestination {
	DoubleRegister(DoubleRegisters),
	StackPointer,
	AddressInImmediate(u16),
	AddressInRegister(DoubleRegisters)
}

impl DoubleByteDestination {
	pub(crate) fn change_destination(&self, value: u16) -> Box<dyn Change> {
		match self {
			Self::DoubleRegister(double_register) => {
				Box::new(DoubleRegisterChange::new(*double_register, value))
			},
			Self::StackPointer => {
				Box::new(SpChange::new(value))
			},
			Self::AddressInImmediate(address) => {
				Box::new(MemoryDoubleByteWriteChange::write_to_immediate_address(*address, value))
			}
			_ => todo!()
		}
	}
}

pub(crate) trait DoubleByteOperation {
	type C: Change;

	fn execute(
		&self,
		cpu: &Cpu,
		src: &DoubleByteSource,
		dst: &DoubleByteDestination,
	) -> Result<Self::C, ExecutionError>;
}

pub(crate) struct BaseDoubleByteInstruction<O>
	where O: DoubleByteOperation {
	src: DoubleByteSource,
	dst: DoubleByteDestination,
	op: O,
}

impl<O> BaseDoubleByteInstruction<O> where O: DoubleByteOperation {
	pub(crate) fn new(src: DoubleByteSource, dst: DoubleByteDestination, op: O) -> Self {
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

pub(crate) trait BinaryDoubleOperation {
	type C: Change;

	fn compute_changes(
		&self, cpu: &Cpu, left: &DoubleByteSource, right: &DoubleByteSource, dst: &DoubleByteDestination,
	) -> Result<Self::C, ExecutionError>;
}

pub(crate) trait BinaryOperation {
	type C: Change;
	fn compute_changes(
		&self, cpu: &Cpu, left: &ByteSource, right: &ByteSource, dst: &ByteDestination,
	) -> Result<Self::C, ExecutionError>;
}

pub(crate) struct BinaryInstruction<O: BinaryOperation> {
	left: ByteSource,
	right: ByteSource,
	dst: ByteDestination,
	op: O,
}

impl<O: BinaryOperation> BinaryInstruction<O> {
	pub(crate) fn new(left: ByteSource, right: ByteSource, dst: ByteDestination, op: O) -> Self {
		Self { left, right, dst, op }
	}
}

impl<O: BinaryOperation> ChangesetInstruction for BinaryInstruction<O> {
	type C = O::C;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		self.op.compute_changes(cpu, &self.left, &self.right, &self.dst)
	}
}