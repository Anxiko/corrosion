use crate::hardware::alu::{add_u8, delta_u8};
use crate::hardware::cpu::Cpu;
use crate::instructions::base::double_byte::{BinaryDoubleByteInstruction, BinaryDoubleByteOperation, DoubleByteDestination, DoubleByteSource, UnaryDoubleByteInstruction, UnaryDoubleByteOperation};
use crate::instructions::changeset::{BitFlagsChange, Change, ChangeList, ChangesetInstruction};
use crate::instructions::ExecutionError;
use crate::instructions::shared::IndexUpdateType;

pub(crate) struct BinaryDoubleByteAddOperation;

impl BinaryDoubleByteAddOperation {
	pub(crate) fn new() -> Self {
		Self
	}
}

impl BinaryDoubleByteOperation for BinaryDoubleByteAddOperation {
	type C = ChangeList;

	fn compute_changes(
		&self, cpu: &Cpu, left: &DoubleByteSource, right: &DoubleByteSource, dst: &DoubleByteDestination,
	) -> Result<Self::C, ExecutionError> {
		let left_value = left.read(cpu)?;
		let right_value = right.read(cpu)?;

		let high_left_value = left_value.to_le_bytes()[1];
		let high_right_value = left_value.to_le_bytes()[1];

		let (result, _overflow) = left_value.overflowing_add(right_value);

		let high_alu_result = add_u8(high_left_value, high_right_value);

		Ok(ChangeList::new(vec![
			dst.change_destination(result),
			Box::new(
				BitFlagsChange::keep_all()
					.with_subtraction_flag(false)
					.with_half_carry_flag(high_alu_result.half_carry)
					.with_carry_flag(high_alu_result.carry)
			),
		]))
	}
}


pub(crate) type BinaryDoubleByteAddInstruction = BinaryDoubleByteInstruction<BinaryDoubleByteAddOperation>;

pub(crate) struct IncOrDecDoubleByteOperation {
	type_: IndexUpdateType,
}

impl IncOrDecDoubleByteOperation {
	pub(crate) fn new(type_: IndexUpdateType) -> Self {
		Self { type_ }
	}
}

impl UnaryDoubleByteOperation for IncOrDecDoubleByteOperation {
	type C = Box<dyn Change>;

	fn execute(&self, cpu: &Cpu, src: &DoubleByteSource, dst: &DoubleByteDestination) -> Result<Self::C, ExecutionError> {
		let value = src.read(cpu)?;
		let delta = i16::from(self.type_.to_delta());
		let result = value.wrapping_add_signed(delta);

		Ok(dst.change_destination(result))
	}
}

pub(crate) type IncOrDecDoubleInstruction = UnaryDoubleByteInstruction<IncOrDecDoubleByteOperation>;

pub(crate) struct AddSignedByteToDoubleByte {
	src: DoubleByteSource,
	dst: DoubleByteDestination,
	delta: i8,
}

impl AddSignedByteToDoubleByte {
	pub(crate) fn new(src: DoubleByteSource, dst: DoubleByteDestination, delta: i8) -> Self {
		Self { src, dst, delta }
	}

	pub(crate) fn add_to_sp(delta: i8) -> Self {
		Self::new(DoubleByteSource::StackPointer, DoubleByteDestination::StackPointer, delta)
	}
}

impl ChangesetInstruction for AddSignedByteToDoubleByte {
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

#[cfg(test)]
mod tests {
	use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
	use crate::instructions::changeset::{DoubleRegisterChange, SpChange};

	use super::*;

	#[test]
	fn double_byte_add() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_double_named(DoubleRegisters::HL, 0x1234);
		cpu.sp.write(0x4321);

		let instruction = BinaryDoubleByteAddInstruction::new(
			DoubleByteSource::DoubleRegister(DoubleRegisters::HL),
			DoubleByteSource::StackPointer,
			DoubleByteDestination::DoubleRegister(DoubleRegisters::HL),
			BinaryDoubleByteAddOperation::new(),
		);

		let actual = instruction.compute_change(&cpu).unwrap();
		let expected = ChangeList::new(vec![
			Box::new(DoubleRegisterChange::new(DoubleRegisters::HL, 0x1234 + 0x4321)),
			Box::new(
				BitFlagsChange::keep_all()
					.with_subtraction_flag(false)
					.with_half_carry_flag(false)
					.with_carry_flag(false)
			),
		]);

		assert_eq!(actual, expected);
	}

	#[test]
	fn inc() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_double_named(DoubleRegisters::HL, 0x1234);

		let instruction = IncOrDecDoubleInstruction::new(
			DoubleByteSource::DoubleRegister(DoubleRegisters::HL),
			DoubleByteDestination::DoubleRegister(DoubleRegisters::HL),
			IncOrDecDoubleByteOperation::new(IndexUpdateType::Increment),
		);

		let actual = instruction.compute_change(&cpu).unwrap();
		let expected: Box<dyn Change> = Box::new(DoubleRegisterChange::new(
			DoubleRegisters::HL,
			0x1234 + 1,
		));

		assert_eq!(actual, expected);
	}

	#[test]
	fn add_byte_to_double_byte() {
		let mut cpu = Cpu::new();
		cpu.sp.write(0x1234);
		cpu.register_bank.write_single_named(SingleRegisters::B, -0x35i8 as u8);

		let instruction = AddSignedByteToDoubleByte::add_to_sp(
			-0x35
		);

		let actual = instruction.compute_change(&cpu).unwrap();
		let expected = ChangeList::new(vec![
			Box::new(SpChange::new(0x11FF)),
			Box::new(
				BitFlagsChange::zero_all()
					.with_half_carry_flag(true)
					.with_carry_flag(true)
			),
		]);

		assert_eq!(actual, expected);
	}
}