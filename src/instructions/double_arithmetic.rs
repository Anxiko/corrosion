use crate::hardware::alu::{add_u8, delta_u8};
use crate::hardware::cpu::Cpu;
use crate::instructions::base::double_byte::{UnaryDoubleByteInstruction, BinaryDoubleByteOperation, DoubleByteDestination, UnaryDoubleByteOperation, DoubleByteSource};
use crate::instructions::changeset::{BitFlagsChange, Change, ChangeList, ChangesetInstruction};
use crate::instructions::ExecutionError;

pub(crate) struct BinaryDoubleAddOperation;

impl BinaryDoubleAddOperation {
	pub(crate) fn new() -> Self {
		Self
	}
}

impl BinaryDoubleByteOperation for BinaryDoubleAddOperation {
	type C = Box<dyn Change>;

	fn compute_changes(
		&self, cpu: &Cpu, left: &DoubleByteSource, right: &DoubleByteSource, dst: &DoubleByteDestination,
	) -> Result<Self::C, ExecutionError> {
		let left_value = left.read(cpu)?;
		let right_value = right.read(cpu)?;

		let high_left_value = left_value.to_le_bytes()[1];
		let high_right_value = left_value.to_le_bytes()[1];

		let (result, _overflow) = left_value.overflowing_add(right_value);

		let high_alu_result = add_u8(high_left_value, high_right_value);

		Ok(Box::new(ChangeList::new(vec![
			dst.change_destination(result),
			Box::new(
				BitFlagsChange::keep_all()
					.with_subtraction_flag(false)
					.with_half_carry_flag(high_alu_result.half_carry)
					.with_carry_flag(high_alu_result.carry)
			),
		])))
	}
}

pub(crate) struct BinaryDoubleInstruction<O: BinaryDoubleByteOperation> {
	left: DoubleByteSource,
	right: DoubleByteSource,
	dst: DoubleByteDestination,
	op: O,
}

impl<O: BinaryDoubleByteOperation> BinaryDoubleInstruction<O> {
	pub(crate) fn new(left: DoubleByteSource, right: DoubleByteSource, dst: DoubleByteDestination, op: O) -> Self {
		Self { left, right, dst, op }
	}
}

impl<O> ChangesetInstruction for BinaryDoubleInstruction<O> where
	O: BinaryDoubleByteOperation {
	type C = O::C;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		self.op.compute_changes(cpu, &self.left, &self.right, &self.dst)
	}
}

pub(crate) type BinaryDoubleAddInstruction = BinaryDoubleInstruction<BinaryDoubleAddOperation>;


#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum IncOrDecDoubleType {
	Increment,
	Decrement,
}

impl IncOrDecDoubleType {
	fn delta(&self) -> i16 {
		match self {
			Self::Increment => 1,
			Self::Decrement => -1
		}
	}
}

pub(crate) struct IncOrDecDoubleOperation {
	type_: IncOrDecDoubleType,
}

impl IncOrDecDoubleOperation {
	pub(crate) fn new(type_: IncOrDecDoubleType) -> Self {
		Self { type_ }
	}

	pub(crate) fn increment() -> Self {
		Self::new(IncOrDecDoubleType::Increment)
	}

	pub(crate) fn decrement() -> Self {
		Self::new(IncOrDecDoubleType::Decrement)
	}
}

impl UnaryDoubleByteOperation for IncOrDecDoubleOperation {
	type C = Box<dyn Change>;

	fn execute(&self, cpu: &Cpu, src: &DoubleByteSource, dst: &DoubleByteDestination) -> Result<Self::C, ExecutionError> {
		let value = src.read(cpu)?;
		let delta = self.type_.delta();
		let result = value.wrapping_add_signed(delta);

		Ok(dst.change_destination(result))
	}
}

pub(crate) type IncOrDecDoubleInstruction = UnaryDoubleByteInstruction<IncOrDecDoubleOperation>;

pub(crate) struct AddSignedByteToDouble {
	src: DoubleByteSource,
	dst: DoubleByteDestination,
	delta: i8,
}

impl AddSignedByteToDouble {
	pub(crate) fn new(src: DoubleByteSource, dst: DoubleByteDestination, delta: i8) -> Self {
		Self { src, dst, delta }
	}

	pub(crate) fn add_to_sp(delta: i8) -> Self {
		Self::new(DoubleByteSource::StackPointer, DoubleByteDestination::StackPointer, delta)
	}
}

impl ChangesetInstruction for AddSignedByteToDouble {
	type C = ChangeList;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let value = self.src.read(cpu)?;
		let value_lower = value.to_le_bytes()[0];

		let result = value.wrapping_add_signed(self.delta.into());
		let lower_result = delta_u8(value_lower, self.delta);

		let bitflag_changes = BitFlagsChange::from(lower_result)
			.with_zero_flag(false)
			.with_subtraction_flag(false);

		Ok(ChangeList::new(vec![
			self.dst.change_destination(result),
			Box::new(bitflag_changes),
		]))
	}
}