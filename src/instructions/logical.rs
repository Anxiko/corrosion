use crate::hardware::cpu::Cpu;
use crate::instructions::base::byte::{BinaryByteInstruction, UnaryByteInstruction, UnaryByteOperation};
use crate::instructions::base::byte::{BinaryByteOperation, ByteDestination, ByteSource};
use crate::instructions::changeset::{BitFlagsChange, ChangeList};
use crate::instructions::ExecutionError;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum BinaryLogicalOperationType {
	And,
	Or,
	Xor,
}

impl BinaryLogicalOperationType {
	fn compute(&self, left: u8, right: u8) -> u8 {
		match self {
			Self::And => left & right,
			Self::Or => left | right,
			Self::Xor => left ^ right
		}
	}

	fn is_and(&self) -> bool {
		matches!(self, Self::And)
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct BinaryLogicalOperation {
	type_: BinaryLogicalOperationType,
}

impl BinaryLogicalOperation {
	pub(crate) fn new(type_: BinaryLogicalOperationType) -> Self {
		Self { type_ }
	}
}

impl BinaryByteOperation for BinaryLogicalOperation {
	type C = ChangeList;

	fn compute_changes(
		&self, cpu: &Cpu, left: &ByteSource, right: &ByteSource, dst: &ByteDestination,
	) -> Result<Self::C, ExecutionError> {
		let left_value = left.read(cpu)?;
		let right_value = right.read(cpu)?;
		let result = self.type_.compute(left_value, right_value);

		Ok(ChangeList::new(vec![
			dst.change_destination(result),
			Box::new(
				BitFlagsChange::zero_all()
					.with_zero_flag(result == 0)
					.with_half_carry_flag(self.type_.is_and())
			),
		]))
	}
}

pub(crate) type BinaryLogicalInstruction = BinaryByteInstruction<BinaryLogicalOperation>;

pub(crate) struct LogicalNegateOperation;

impl UnaryByteOperation for LogicalNegateOperation {
	type C = ChangeList;

	fn execute(&self, cpu: &Cpu, src: &ByteSource, dst: &ByteDestination) -> Result<Self::C, ExecutionError> {
		let value = src.read(cpu)?;
		let new_value = !value;

		Ok(ChangeList::new(vec![
			dst.change_destination(new_value),
			Box::new(
				BitFlagsChange::keep_all()
					.with_subtraction_flag(true)
					.with_half_carry_flag(true)
			),
		]))
	}
}

pub(crate) type LogicalNegateInstruction = UnaryByteInstruction<LogicalNegateOperation>;

impl LogicalNegateInstruction {
	pub(crate) fn negate_acc() -> Self {
		Self::new(
			ByteSource::read_from_acc(),
			ByteDestination::write_to_acc(),
			LogicalNegateOperation,
		)
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::register_bank::SingleRegisters;
	use crate::instructions::ACC_REGISTER;
	use crate::instructions::changeset::{ChangesetInstruction, SingleRegisterChange};

	use super::*;

	#[test]
	fn negate() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(ACC_REGISTER, 0b11001010);

		let instruction = LogicalNegateInstruction::negate_acc();

		let actual = instruction.compute_change(&cpu).unwrap();
		let expected = ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, !0b11001010)),
			Box::new(BitFlagsChange::keep_all().with_subtraction_flag(true).with_half_carry_flag(true)),
		]);

		assert_eq!(actual, expected);
	}

	#[test]
	fn and() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(ACC_REGISTER, 0b11001010);
		cpu.register_bank.write_single_named(SingleRegisters::B, 0b10101100);

		let instruction = BinaryLogicalInstruction::new(
			ByteSource::SingleRegister(ACC_REGISTER),
			ByteSource::SingleRegister(SingleRegisters::B),
			ByteDestination::SingleRegister(ACC_REGISTER),
			BinaryLogicalOperation::new(BinaryLogicalOperationType::And),
		);

		let actual = instruction.compute_change(&cpu).unwrap();
		let expected = ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b10001000)),
			Box::new(
				BitFlagsChange::zero_all()
					.with_half_carry_flag(true)
			),
		]);

		assert_eq!(actual, expected);
	}

	#[test]
	fn or() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(ACC_REGISTER, 0b11001010);
		cpu.register_bank.write_single_named(SingleRegisters::B, 0b10101100);

		let instruction = BinaryLogicalInstruction::new(
			ByteSource::SingleRegister(ACC_REGISTER),
			ByteSource::SingleRegister(SingleRegisters::B),
			ByteDestination::SingleRegister(ACC_REGISTER),
			BinaryLogicalOperation::new(BinaryLogicalOperationType::Or),
		);

		let actual = instruction.compute_change(&cpu).unwrap();
		let expected = ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b11101110)),
			Box::new(
				BitFlagsChange::zero_all()
			),
		]);

		assert_eq!(actual, expected);
	}

	#[test]
	fn xor() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(ACC_REGISTER, 0b1100_1010);
		cpu.register_bank.write_single_named(SingleRegisters::B, 0b1010_1100);

		let instruction = BinaryLogicalInstruction::new(
			ByteSource::SingleRegister(ACC_REGISTER),
			ByteSource::SingleRegister(SingleRegisters::B),
			ByteDestination::SingleRegister(ACC_REGISTER),
			BinaryLogicalOperation::new(BinaryLogicalOperationType::Xor),
		);

		let actual = instruction.compute_change(&cpu).unwrap();
		let expected = ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b0110_0110)),
			Box::new(
				BitFlagsChange::zero_all()
			),
		]);

		assert_eq!(actual, expected);
	}
}