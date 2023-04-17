use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::BitFlags;
use crate::instructions::base::byte::{
	ByteDestination, ByteSource, UnaryByteInstruction, UnaryByteOperation,
};
use crate::instructions::changeset::{Change, ChangeList};
use crate::instructions::shifting::operation::ByteShiftOperation;
use crate::instructions::ExecutionError;

pub(crate) mod operation;

impl UnaryByteOperation for ByteShiftOperation {
	type C = ChangeList;

	fn execute(
		&self,
		cpu: &Cpu,
		src: &ByteSource,
		dst: &ByteDestination,
	) -> Result<Self::C, ExecutionError> {
		let value = src.read(cpu)?;
		let old_carry = cpu.register_bank.read_bit_flag(BitFlags::Carry);

		let changes = self.compute_changes(value, old_carry, dst);
		Ok(changes)
	}
}

pub(crate) type ByteShiftInstruction = UnaryByteInstruction<ByteShiftOperation>;

#[derive(Debug)]
pub(crate) struct ByteSwapOperation {}

impl ByteSwapOperation {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl UnaryByteOperation for ByteSwapOperation {
	type C = Box<dyn Change>;

	fn execute(
		&self,
		cpu: &Cpu,
		src: &ByteSource,
		dst: &ByteDestination,
	) -> Result<Self::C, ExecutionError> {
		let byte = src.read(cpu)?;
		let high = byte & 0xF0;
		let low = byte & 0x0F;

		let result = (high >> 4) | (low << 4);
		Ok(dst.change_destination(result))
	}
}

pub(crate) type ByteSwapInstruction = UnaryByteInstruction<ByteSwapOperation>;

#[cfg(test)]
mod tests {
	use crate::hardware::register_bank::SingleRegisters;
	use crate::instructions::changeset::{
		BitFlagsChange, ChangesetExecutable, SingleRegisterChange,
	};
	use crate::instructions::shifting::operation::{ShiftDirection, ShiftType};
	use crate::instructions::ACC_REGISTER;

	use super::*;

	#[test]
	fn shift_instruction() {
		let mut cpu = Cpu::new();
		cpu.register_bank
			.write_single_named(ACC_REGISTER, 0b0011_0101);
		cpu.register_bank.write_bit_flag(BitFlags::Carry, true);

		let instruction = ByteShiftInstruction::new(
			ByteSource::read_from_acc(),
			ByteDestination::write_to_acc(),
			ByteShiftOperation::new(ShiftDirection::Left, ShiftType::RotateWithCarry),
		);

		let expected = ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b0110_1011)),
			Box::new(
				BitFlagsChange::zero_all()
					.with_carry_flag(false)
					.with_zero_flag(false),
			),
		]);
		let actual = instruction.compute_change(&cpu).expect("Compute changes");

		assert_eq!(actual, expected);
	}

	#[test]
	fn swap_operation() {
		let mut cpu = Cpu::new();
		cpu.register_bank
			.write_single_named(SingleRegisters::B, 0b0011_0101);

		let operation = ByteSwapOperation::new();

		let actual = operation
			.execute(
				&cpu,
				&ByteSource::SingleRegister(SingleRegisters::B),
				&ByteDestination::write_to_acc(),
			)
			.expect("Operation to execute");

		let expected: Box<dyn Change> =
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b0101_0011));

		assert_eq!(actual, expected);
	}
}
