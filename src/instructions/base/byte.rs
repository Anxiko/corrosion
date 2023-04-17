use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Rom;
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use crate::instructions::{ACC_REGISTER, ExecutionError};
use crate::instructions::changeset::{
	Change, ChangesetExecutable, MemoryByteWriteChange, SingleRegisterChange,
};

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

	pub(crate) fn read(&self, cpu: &Cpu) -> Result<u8, ExecutionError> {
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
	SingleRegister(SingleRegisters),
	AddressImmediate(u16),
	AddressInRegister(DoubleRegisters),
	OffsetAddressInRegister { base: u16, offset: SingleRegisters },
}

impl ByteDestination {
	pub(crate) fn write_to_acc() -> Self {
		Self::SingleRegister(ACC_REGISTER)
	}

	pub(crate) fn change_destination(&self, value: u8) -> Box<dyn Change> {
		match self {
			Self::SingleRegister(single_reg) => {
				Box::new(SingleRegisterChange::new(*single_reg, value))
			}
			Self::AddressImmediate(address_immediate) => Box::new(
				MemoryByteWriteChange::write_to_immediate(*address_immediate, value),
			),
			Self::AddressInRegister(double_reg) => {
				Box::new(MemoryByteWriteChange::write_to_register(*double_reg, value))
			}
			Self::OffsetAddressInRegister { base, offset } => Box::new(
				MemoryByteWriteChange::write_to_offset(*base, *offset, value),
			),
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

#[derive(Debug)]
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

impl<O> ChangesetExecutable for UnaryByteInstruction<O>
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
		&self,
		cpu: &Cpu,
		left: &ByteSource,
		right: &ByteSource,
		dst: &ByteDestination,
	) -> Result<Self::C, ExecutionError>;
}

#[derive(Debug)]
pub(crate) struct BinaryByteInstruction<O: BinaryByteOperation> {
	left: ByteSource,
	right: ByteSource,
	dst: ByteDestination,
	op: O,
}

impl<O: BinaryByteOperation> BinaryByteInstruction<O> {
	pub(crate) fn new(left: ByteSource, right: ByteSource, dst: ByteDestination, op: O) -> Self {
		Self {
			left,
			right,
			dst,
			op,
		}
	}
}

impl<O: BinaryByteOperation> ChangesetExecutable for BinaryByteInstruction<O> {
	type C = O::C;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		self.op
			.compute_changes(cpu, &self.left, &self.right, &self.dst)
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::ram::{Ram, WORKING_RAM_START};

	use super::*;

	#[test]
	fn source_register() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(ACC_REGISTER, 0x12);

		let source = ByteSource::read_from_acc();

		assert_eq!(source.read(&cpu).unwrap(), 0x12);
	}

	#[test]
	fn source_address_in_register() {
		let mut cpu = Cpu::new();
		cpu.register_bank
			.write_double_named(DoubleRegisters::HL, WORKING_RAM_START);
		cpu.mapped_ram.write_byte(WORKING_RAM_START, 0x12).unwrap();

		let source = ByteSource::AddressInRegister(DoubleRegisters::HL);

		assert_eq!(source.read(&cpu).unwrap(), 0x12);
	}

	#[test]
	fn source_offset_in_register() {
		let mut cpu = Cpu::new();
		cpu.register_bank
			.write_single_named(SingleRegisters::B, 0x20);
		cpu.mapped_ram
			.write_byte(WORKING_RAM_START + 0x20, 0x12)
			.unwrap();

		let source = ByteSource::OffsetAddressInRegister {
			base: WORKING_RAM_START,
			offset: SingleRegisters::B,
		};

		assert_eq!(source.read(&cpu).unwrap(), 0x12);
	}

	#[test]
	fn source_address_in_immediate() {
		let mut cpu = Cpu::new();
		cpu.mapped_ram
			.write_byte(WORKING_RAM_START + 0x20, 0x12)
			.unwrap();

		let source = ByteSource::AddressInImmediate(WORKING_RAM_START + 0x20);

		assert_eq!(source.read(&cpu).unwrap(), 0x12);
	}

	#[test]
	fn source_immediate() {
		let cpu = Cpu::new();

		let source = ByteSource::Immediate(0x12);

		assert_eq!(source.read(&cpu).unwrap(), 0x12);
	}

	#[test]
	fn destination_register() {
		let dest = ByteDestination::write_to_acc();

		let actual = dest.change_destination(0x12);
		let expected: Box<dyn Change> = Box::new(SingleRegisterChange::new(ACC_REGISTER, 0x12));

		assert_eq!(actual, expected);
	}

	#[test]
	fn destination_address_in_register() {
		let dest = ByteDestination::AddressInRegister(DoubleRegisters::HL);

		let actual = dest.change_destination(0x12);
		let expected: Box<dyn Change> = Box::new(MemoryByteWriteChange::write_to_register(
			DoubleRegisters::HL,
			0x12,
		));

		assert_eq!(actual, expected);
	}

	#[test]
	fn destination_address_immediate() {
		let dest = ByteDestination::AddressImmediate(WORKING_RAM_START);

		let actual = dest.change_destination(0x12);
		let expected: Box<dyn Change> = Box::new(MemoryByteWriteChange::write_to_immediate(
			WORKING_RAM_START,
			0x12,
		));

		assert_eq!(actual, expected);
	}

	#[test]
	fn destination_offset_in_register() {
		let dest = ByteDestination::OffsetAddressInRegister {
			base: WORKING_RAM_START,
			offset: SingleRegisters::B,
		};

		let actual = dest.change_destination(0x12);
		let expected: Box<dyn Change> = Box::new(MemoryByteWriteChange::write_to_offset(
			WORKING_RAM_START,
			SingleRegisters::B,
			0x12,
		));

		assert_eq!(actual, expected);
	}
}
