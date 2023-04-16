use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::DoubleRegisters;
use crate::instructions::changeset::{Change, ChangesetInstruction, DoubleRegisterChange, MemoryDoubleByteWriteChange, SpChange};
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
			Self::Immediate(immediate) => {
				Ok(*immediate)
			}
			Self::StackPointer => {
				Ok(cpu.sp.read())
			}
		}
	}
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum DoubleByteDestination {
	DoubleRegister(DoubleRegisters),
	StackPointer,
	AddressInImmediate(u16),
	AddressInRegister(DoubleRegisters),
}

impl DoubleByteDestination {
	pub(crate) fn change_destination(&self, value: u16) -> Box<dyn Change> {
		match self {
			Self::DoubleRegister(double_register) => {
				Box::new(DoubleRegisterChange::new(*double_register, value))
			}
			Self::StackPointer => {
				Box::new(SpChange::new(value))
			}
			Self::AddressInImmediate(address) => {
				Box::new(MemoryDoubleByteWriteChange::write_to_immediate(*address, value))
			}
			Self::AddressInRegister(double_register) => {
				Box::new(MemoryDoubleByteWriteChange::write_to_source(
					DoubleByteSource::DoubleRegister(*double_register),
					value,
				))
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

pub(crate) struct UnaryDoubleByteInstruction<O>
	where O: UnaryDoubleByteOperation {
	src: DoubleByteSource,
	dst: DoubleByteDestination,
	op: O,
}

impl<O> UnaryDoubleByteInstruction<O> where O: UnaryDoubleByteOperation {
	pub(crate) fn new(src: DoubleByteSource, dst: DoubleByteDestination, op: O) -> Self {
		Self { src, dst, op }
	}
}

impl<O> ChangesetInstruction for UnaryDoubleByteInstruction<O>
	where O: UnaryDoubleByteOperation {
	type C = O::C;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		self.op.execute(cpu, &self.src, &self.dst)
	}
}

pub(crate) trait BinaryDoubleByteOperation {
	type C: Change;

	fn compute_changes(
		&self, cpu: &Cpu, left: &DoubleByteSource, right: &DoubleByteSource, dst: &DoubleByteDestination,
	) -> Result<Self::C, ExecutionError>;
}

pub(crate) struct BinaryDoubleByteInstruction<O: BinaryDoubleByteOperation> {
	left: DoubleByteSource,
	right: DoubleByteSource,
	dst: DoubleByteDestination,
	op: O,
}

impl<O: BinaryDoubleByteOperation> BinaryDoubleByteInstruction<O> {
	pub(crate) fn new(left: DoubleByteSource, right: DoubleByteSource, dst: DoubleByteDestination, op: O) -> Self {
		Self { left, right, dst, op }
	}
}

impl<O> ChangesetInstruction for BinaryDoubleByteInstruction<O> where
	O: BinaryDoubleByteOperation {
	type C = O::C;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		self.op.compute_changes(cpu, &self.left, &self.right, &self.dst)
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::ram::{Ram, WORKING_RAM_START};

	use super::*;

	#[test]
	fn source_register() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_double_named(DoubleRegisters::HL, 0x1234);

		let source = DoubleByteSource::DoubleRegister(DoubleRegisters::HL);

		assert_eq!(source.read(&cpu).unwrap(), 0x1234);
	}

	#[test]
	fn source_immediate() {
		let mut cpu = Cpu::new();
		cpu.mapped_ram.write_byte(WORKING_RAM_START + 0x20, 0x12).unwrap();

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
		let expected: Box<dyn Change> = Box::new(DoubleRegisterChange::new(DoubleRegisters::HL, 0x1234));

		assert_eq!(actual, expected);
	}

	#[test]
	fn destination_address_in_register() {
		let dest = DoubleByteDestination::AddressInRegister(DoubleRegisters::HL);

		let actual = dest.change_destination(0x1234);
		let expected: Box<dyn Change> = Box::new(MemoryDoubleByteWriteChange::write_to_source(
			DoubleByteSource::DoubleRegister(DoubleRegisters::HL), 0x1234,
		));

		assert_eq!(actual, expected);
	}

	#[test]
	fn destination_address_immediate() {
		let dest = DoubleByteDestination::AddressInImmediate(WORKING_RAM_START);

		let actual = dest.change_destination(0x1234);
		let expected: Box<dyn Change> = Box::new(MemoryDoubleByteWriteChange::write_to_immediate(
			WORKING_RAM_START, 0x1234,
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