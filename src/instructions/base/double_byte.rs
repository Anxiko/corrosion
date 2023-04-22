use std::fmt::{Display, Formatter};

use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::DoubleRegisters;
use crate::instructions::changeset::{
	Change, ChangesetExecutable, DoubleRegisterChange, MemoryDoubleByteWriteChange, SpChange,
};
use crate::instructions::ExecutionError;

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum DoubleByteSource {
	DoubleRegister(DoubleRegisters),
	Immediate(u16),
	StackPointer,
}

impl DoubleByteSource {
	pub(crate) fn read(&self, cpu: &Cpu) -> Result<u16, ExecutionError> {
		match self {
			Self::DoubleRegister(double_register) => {
				Ok(cpu.register_bank.read_double_named(*double_register))
			}
			Self::Immediate(immediate) => Ok(*immediate),
			Self::StackPointer => Ok(cpu.sp.read()),
		}
	}
}

impl Display for DoubleByteSource {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DoubleRegister(r) => {
				write!(f, "{r}")
			}
			Self::Immediate(i) => {
				write!(f, "{i:#06X}")
			}
			Self::StackPointer => {
				write!(f, "SP")
			}
		}
	}
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum DoubleByteDestination {
	DoubleRegister(DoubleRegisters),
	StackPointer,
	AddressInImmediate(u16),
}

impl DoubleByteDestination {
	pub(crate) fn change_destination(&self, value: u16) -> Box<dyn Change> {
		match self {
			Self::DoubleRegister(double_register) => {
				Box::new(DoubleRegisterChange::new(*double_register, value))
			}
			Self::StackPointer => Box::new(SpChange::new(value)),
			Self::AddressInImmediate(address) => Box::new(
				MemoryDoubleByteWriteChange::write_to_immediate(*address, value),
			),
		}
	}
}

impl Display for DoubleByteDestination {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DoubleRegister(r) => {
				write!(f, "{r}")
			}
			Self::StackPointer => {
				write!(f, "SP")
			}
			Self::AddressInImmediate(a) => {
				write!(f, "({a:#06X})")
			}
		}
	}
}

pub(crate) trait UnaryDoubleByteOperation {
	type C: Change;

	fn execute(
		&self,
		cpu: &Cpu,
		src: &DoubleByteSource,
		dst: &DoubleByteDestination,
	) -> Result<Self::C, ExecutionError>;
}

#[derive(Debug)]
pub(crate) struct UnaryDoubleByteInstruction<O>
	where
		O: UnaryDoubleByteOperation,
{
	src: DoubleByteSource,
	dst: DoubleByteDestination,
	op: O,
}

impl<O> UnaryDoubleByteInstruction<O>
	where
		O: UnaryDoubleByteOperation,
{
	pub(crate) fn new(src: DoubleByteSource, dst: DoubleByteDestination, op: O) -> Self {
		Self { src, dst, op }
	}
}

impl<O> ChangesetExecutable for UnaryDoubleByteInstruction<O>
	where
		O: UnaryDoubleByteOperation,
{
	type C = O::C;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		self.op.execute(cpu, &self.src, &self.dst)
	}
}

impl<O: UnaryDoubleByteOperation + Display> Display for UnaryDoubleByteInstruction<O> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} {} <- {}", self.op, self.dst, self.src)
	}
}

pub(crate) trait BinaryDoubleByteOperation {
	type C: Change;

	fn compute_changes(
		&self,
		cpu: &Cpu,
		left: &DoubleByteSource,
		right: &DoubleByteSource,
		dst: &DoubleByteDestination,
	) -> Result<Self::C, ExecutionError>;
}

#[derive(Debug)]
pub(crate) struct BinaryDoubleByteInstruction<O: BinaryDoubleByteOperation> {
	left: DoubleByteSource,
	right: DoubleByteSource,
	dst: DoubleByteDestination,
	op: O,
}

impl<O: BinaryDoubleByteOperation> BinaryDoubleByteInstruction<O> {
	pub(crate) fn new(
		left: DoubleByteSource,
		right: DoubleByteSource,
		dst: DoubleByteDestination,
		op: O,
	) -> Self {
		Self {
			left,
			right,
			dst,
			op,
		}
	}
}

impl<O> ChangesetExecutable for BinaryDoubleByteInstruction<O>
	where
		O: BinaryDoubleByteOperation,
{
	type C = O::C;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		self.op
			.compute_changes(cpu, &self.left, &self.right, &self.dst)
	}
}

impl<O: BinaryDoubleByteOperation + Display> Display for BinaryDoubleByteInstruction<O> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} {} <- {}, {}", self.op, self.dst, self.left, self.right)
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::ram::{Ram, WORKING_RAM_START};

	use super::*;

	#[test]
	fn source_register() {
		let mut cpu = Cpu::new();
		cpu.register_bank
			.write_double_named(DoubleRegisters::HL, 0x1234);

		let source = DoubleByteSource::DoubleRegister(DoubleRegisters::HL);

		assert_eq!(source.read(&cpu).unwrap(), 0x1234);
	}

	#[test]
	fn source_immediate() {
		let mut cpu = Cpu::new();
		cpu.mapped_ram
			.write_byte(WORKING_RAM_START + 0x20, 0x12)
			.unwrap();

		let source = DoubleByteSource::Immediate(0x1234);

		assert_eq!(source.read(&cpu).unwrap(), 0x1234);
	}

	#[test]
	fn source_sp() {
		let mut cpu = Cpu::new();
		cpu.sp.write(0x1234);

		let source = DoubleByteSource::StackPointer;

		assert_eq!(source.read(&cpu).unwrap(), 0x1234);
	}

	#[test]
	fn destination_register() {
		let dest = DoubleByteDestination::DoubleRegister(DoubleRegisters::HL);

		let actual = dest.change_destination(0x1234);
		let expected: Box<dyn Change> =
			Box::new(DoubleRegisterChange::new(DoubleRegisters::HL, 0x1234));

		assert_eq!(actual, expected);
	}

	#[test]
	fn destination_address_immediate() {
		let dest = DoubleByteDestination::AddressInImmediate(WORKING_RAM_START);

		let actual = dest.change_destination(0x1234);
		let expected: Box<dyn Change> = Box::new(MemoryDoubleByteWriteChange::write_to_immediate(
			WORKING_RAM_START,
			0x1234,
		));

		assert_eq!(actual, expected);
	}

	#[test]
	fn destination_stack_pointer() {
		let dest = DoubleByteDestination::StackPointer;

		let actual = dest.change_destination(0x1234);
		let expected: Box<dyn Change> = Box::new(SpChange::new(0x1234));

		assert_eq!(actual, expected);
	}
}
