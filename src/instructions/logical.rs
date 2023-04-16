use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::SingleRegisters;
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::ACC_REGISTER;
use crate::instructions::base::byte::{BinaryByteInstruction, UnaryByteInstruction, UnaryByteOperation};
use crate::instructions::base::byte::{BinaryByteOperation, ByteDestination, ByteSource};
use crate::instructions::changeset::{BitFlagsChange, Change, ChangeList, SingleRegisterChange};

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