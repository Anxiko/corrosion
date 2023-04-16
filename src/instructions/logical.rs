use crate::hardware::cpu::Cpu;
use crate::instructions::base::byte::{BinaryByteInstruction, UnaryByteInstruction, UnaryByteOperation};
use crate::instructions::base::byte::{BinaryByteOperation, ByteDestination, ByteSource};
use crate::instructions::changeset::{BitFlagsChange, Change, ChangeList};
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
			Box::new(BitFlagsChange::zero_all().with_zero_flag(result == 0)),
		]))
	}
}

pub(crate) type BinaryLogicalInstruction = BinaryByteInstruction<BinaryLogicalOperation>;

pub(crate) struct LogicalNegateOperation;

impl UnaryByteOperation for LogicalNegateOperation {
	type C = Box<dyn Change>;

	fn execute(&self, cpu: &Cpu, src: &ByteSource, dst: &ByteDestination) -> Result<Self::C, ExecutionError> {
		let value = src.read(cpu)?;
		let new_value = !value;

		Ok(dst.change_destination(new_value))
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
	use crate::instructions::ACC_REGISTER;
	use crate::instructions::changeset::{ChangesetInstruction, SingleRegisterChange};

	use super::*;

	#[test]
	fn negate() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(ACC_REGISTER, 0b11001010);

		let instruction = LogicalNegateInstruction::negate_acc();

		let actual = instruction.compute_change(&cpu).unwrap();
		let expected: Box<dyn Change> = Box::new(SingleRegisterChange::new(ACC_REGISTER, !0b11001010));

		assert_eq!(actual, expected);
	}
}