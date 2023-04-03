use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use crate::instructions::{ACC_REGISTER, ExecutionError};
use crate::instructions::changeset::{Change, ChangesetInstruction, MemoryByteWriteChange, SingleRegisterChange};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ByteSource {
	SingleRegister(SingleRegisters),
	AddressInRegister(DoubleRegisters),
	OffsetAddressInRegister { base: u16, offset: SingleRegisters },
	AddressInImmediate(u16),
	Immediate(u8),
}

impl ByteSource {
	pub(crate) fn read_from_acc() -> Self {
		Self::SingleRegister(ACC_REGISTER)
	}

	pub(in crate::instructions) fn read(&self, cpu: &Cpu) -> Result<u8, ExecutionError> {
		match self {
			Self::SingleRegister(single_reg) => {
				Ok(cpu.register_bank.read_single_named(*single_reg))
			}
			Self::AddressInRegister(address_register) => {
				let address = cpu.register_bank.read_double_named(*address_register);
				let result = cpu.mapped_ram.read_byte(address)?;
				Ok(result)
			}
			Self::OffsetAddressInRegister { base, offset } => {
				let offset = cpu.register_bank.read_single_named(*offset);
				let address = base.wrapping_add(offset.into());
				let result = cpu.mapped_ram.read_byte(address)?;
				Ok(result)
			}
			Self::AddressInImmediate(address_immediate) => {
				let result = cpu.mapped_ram.read_byte(*address_immediate)?;
				Ok(result)
			}
			Self::Immediate(value) => Ok(*value),
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ByteDestination {
	Acc,
	SingleRegister(SingleRegisters),
	AddressImmediate(u16),
	AddressInRegister(DoubleRegisters),
	OffsetAddressInRegister { base: u16, offset: SingleRegisters },
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

	pub(crate) fn change_destination(&self, value: u8) -> Box<dyn Change> {
		match self {
			Self::Acc => Box::new(SingleRegisterChange::new(ACC_REGISTER, value)),
			Self::SingleRegister(single_reg) => Box::new(SingleRegisterChange::new(*single_reg, value)),
			Self::AddressImmediate(address_immediate) => {
				Box::new(MemoryByteWriteChange::write_to_immediate(*address_immediate, value))
			}
			Self::AddressInRegister(double_reg) => {
				Box::new(MemoryByteWriteChange::write_to_register(*double_reg, value))
			}
			Self::OffsetAddressInRegister { base, offset } => {
				Box::new(MemoryByteWriteChange::write_to_offset(*base, *offset, value))
			}
		}
	}
}

pub(crate) trait UnaryByteOperation {
	type C: Change;

	fn execute(
		&self,
		cpu: &Cpu,
		src: &ByteSource,
		dst: &ByteDestination,
	) -> Result<Self::C, ExecutionError>;
}

pub(crate) struct UnaryByteInstruction<O>
	where
		O: UnaryByteOperation,
{
	src: ByteSource,
	dst: ByteDestination,
	op: O,
}

impl<O> UnaryByteInstruction<O>
	where
		O: UnaryByteOperation,
{
	pub(crate) fn new(src: ByteSource, dst: ByteDestination, op: O) -> Self {
		Self { src, dst, op }
	}
}

impl<O> ChangesetInstruction for UnaryByteInstruction<O>
	where
		O: UnaryByteOperation,
{
	type C = O::C;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		self.op.execute(cpu, &self.src, &self.dst)
	}
}

pub(crate) trait BinaryByteOperation {
	type C: Change;
	fn compute_changes(
		&self, cpu: &Cpu, left: &ByteSource, right: &ByteSource, dst: &ByteDestination,
	) -> Result<Self::C, ExecutionError>;
}

pub(crate) struct BinaryByteInstruction<O: BinaryByteOperation> {
	left: ByteSource,
	right: ByteSource,
	dst: ByteDestination,
	op: O,
}

impl<O: BinaryByteOperation> BinaryByteInstruction<O> {
	pub(crate) fn new(left: ByteSource, right: ByteSource, dst: ByteDestination, op: O) -> Self {
		Self { left, right, dst, op }
	}
}

impl<O: BinaryByteOperation> ChangesetInstruction for BinaryByteInstruction<O> {
	type C = O::C;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		self.op.compute_changes(cpu, &self.left, &self.right, &self.dst)
	}
}
