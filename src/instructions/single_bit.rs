use std::fmt::{Display, Formatter};

use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Rom;
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use crate::instructions::changeset::{
	BitFlagsChange, Change, ChangesetExecutable, MemoryByteWriteChange, SingleRegisterChange,
};
use crate::instructions::ExecutionError;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum SingleBitOperand {
	SingleRegister(SingleRegisters),
	MemoryAddress,
}

impl Display for SingleBitOperand {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::SingleRegister(r) => write!(f, "{r}"),
			Self::MemoryAddress => write!(f, "(HL)")
		}
	}
}

impl SingleBitOperand {
	fn read_byte(&self, cpu: &Cpu) -> Result<u8, ExecutionError> {
		match self {
			Self::SingleRegister(reg) => Ok(cpu.register_bank.read_single_named(*reg)),
			Self::MemoryAddress => {
				let address = cpu.register_bank.read_double_named(DoubleRegisters::HL);
				let byte = cpu.mapped_ram.read_byte(address)?;
				Ok(byte)
			}
		}
	}

	fn write_change(&self, byte: u8) -> Box<dyn Change> {
		match self {
			Self::SingleRegister(reg) => Box::new(SingleRegisterChange::new(*reg, byte)),
			Self::MemoryAddress => Box::new(MemoryByteWriteChange::write_to_register(
				DoubleRegisters::HL,
				byte,
			)),
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum SingleBitOperation {
	Test,
	Write(bool),
}

impl SingleBitOperation {
	fn as_str(&self) -> &str {
		match self {
			Self::Test => "bit",
			Self::Write(false) => "res",
			Self::Write(true) => "set",
		}
	}
}

impl Display for SingleBitOperation {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let str = self.as_str();
		write!(f, "{str}")
	}
}

impl SingleBitOperation {
	fn compute_change(&self, byte: u8, bitmask: u8, operand: &SingleBitOperand) -> Box<dyn Change> {
		match self {
			Self::Test => {
				// Since we're setting the zero flag, the zero flag is set (zero == true) if the bit is zero
				let test = byte & bitmask == 0;

				let flags_change = BitFlagsChange::keep_all()
					.with_zero_flag(test)
					.with_subtraction_flag(false)
					.with_half_carry_flag(true);

				Box::new(flags_change)
			}
			Self::Write(bit) => {
				let result = if *bit {
					byte | bitmask
				} else {
					byte & (!bitmask)
				};
				operand.write_change(result)
			}
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct SingleBitInstruction {
	operand: SingleBitOperand,
	operation: SingleBitOperation,
	bit_shift: u8,
}

impl SingleBitInstruction {
	pub(crate) fn new(
		operand: SingleBitOperand,
		operation: SingleBitOperation,
		bit_shift: u8,
	) -> Self {
		Self {
			operand,
			operation,
			bit_shift: bit_shift & 0x07,
		}
	}

	fn get_bit(&self) -> u8 {
		1 << self.bit_shift
	}
}

impl Display for SingleBitInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} {}, {}", self.operation, self.bit_shift, self.operand)
	}
}

impl ChangesetExecutable for SingleBitInstruction {
	type C = Box<dyn Change>;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let byte = self.operand.read_byte(cpu)?;
		let bitmask = self.get_bit();

		Ok(self.operation.compute_change(byte, bitmask, &self.operand))
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::ram::{Ram, WORKING_RAM_START};

	use super::*;

	#[test]
	fn test_bit() {
		let mut cpu = Cpu::new();

		cpu.register_bank
			.write_single_named(SingleRegisters::B, 0b11001010);
		cpu.register_bank
			.write_double_named(DoubleRegisters::HL, WORKING_RAM_START);
		cpu.mapped_ram
			.write_byte(WORKING_RAM_START, 0b11001010)
			.expect("Write to RAM");

		let cpu = cpu;

		let instruction = SingleBitInstruction::new(
			SingleBitOperand::SingleRegister(SingleRegisters::B),
			SingleBitOperation::Test,
			3,
		);

		let actual = instruction.compute_change(&cpu).expect("Compute changes");
		let expected: Box<dyn Change> = Box::new(
			BitFlagsChange::keep_all()
				.with_zero_flag(false)
				.with_subtraction_flag(false)
				.with_half_carry_flag(true),
		);

		assert_eq!(actual, expected);

		let instruction =
			SingleBitInstruction::new(SingleBitOperand::MemoryAddress, SingleBitOperation::Test, 4);

		let actual = instruction.compute_change(&cpu).expect("Compute changes");
		let expected: Box<dyn Change> = Box::new(
			BitFlagsChange::keep_all()
				.with_zero_flag(true)
				.with_subtraction_flag(false)
				.with_half_carry_flag(true),
		);

		assert_eq!(actual, expected);
	}

	#[test]
	fn write_bit() {
		let mut cpu = Cpu::new();

		cpu.register_bank
			.write_single_named(SingleRegisters::B, 0b11001010);
		cpu.register_bank
			.write_double_named(DoubleRegisters::HL, WORKING_RAM_START);
		cpu.mapped_ram
			.write_byte(WORKING_RAM_START, 0b11001010)
			.expect("Write to RAM");

		let cpu = cpu;

		let instruction = SingleBitInstruction::new(
			SingleBitOperand::SingleRegister(SingleRegisters::B),
			SingleBitOperation::Write(true),
			0,
		);

		let actual = instruction.compute_change(&cpu).expect("Compute changes");
		let expected: Box<dyn Change> =
			Box::new(SingleRegisterChange::new(SingleRegisters::B, 0b11001011));

		assert_eq!(actual, expected);

		let instruction = SingleBitInstruction::new(
			SingleBitOperand::MemoryAddress,
			SingleBitOperation::Write(false),
			7,
		);

		let actual = instruction.compute_change(&cpu).expect("Compute changes");
		let expected: Box<dyn Change> = Box::new(MemoryByteWriteChange::write_to_register(
			DoubleRegisters::HL,
			0b01001010,
		));

		assert_eq!(actual, expected);
	}
}
