use std::fmt::{Display, Formatter};

use crate::instructions::base::byte::ByteDestination;
use crate::instructions::changeset::{BitFlagsChange, ChangeList};
use crate::instructions::ACC_REGISTER;

#[derive(Debug, Copy, Clone)]
pub enum ShiftDirection {
	Left,
	Right,
}

impl ShiftDirection {
	fn shift_in_mask(self) -> u8 {
		match self {
			Self::Left => 0x01,
			Self::Right => 0x80,
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub enum ShiftType {
	Rotate,
	RotateWithCarry,
	LogicalShift,
	ArithmeticShift,
}

#[derive(Debug)]
pub struct ByteShiftOperation {
	direction: ShiftDirection,
	type_: ShiftType,
}

impl ByteShiftOperation {
	pub(crate) fn new(direction: ShiftDirection, type_: ShiftType) -> Self {
		Self { direction, type_ }
	}

	fn shift_result(&self, value: u8) -> (u8, bool) {
		match self.direction {
			ShiftDirection::Left => (value << 1, value & 0x80 != 0),
			ShiftDirection::Right => (value >> 1, value & 0x01 != 0),
		}
	}

	fn shift_in(&self, shifted_out: bool, old_carry: bool) -> Option<bool> {
		match self.type_ {
			ShiftType::Rotate => Some(shifted_out),
			ShiftType::RotateWithCarry => Some(old_carry),
			_ => None,
		}
	}

	fn preserve_sign_bit(&self) -> bool {
		matches!(
			(self.type_, self.direction),
			(ShiftType::ArithmeticShift, ShiftDirection::Right)
		)
	}

	fn zero_flag_for_result(&self, destination: &ByteDestination, result: u8) -> bool {
		match destination {
			ByteDestination::SingleRegister(ACC_REGISTER) => false,
			_ => result == 0,
		}
	}

	pub(super) fn compute_changes(&self, value: u8, old_carry: bool, dst: &ByteDestination) -> ChangeList {
		let old_sign = value & 80 != 0;
		let (mut result, shifted_out) = self.shift_result(value);

		let shift_in_bit = self.shift_in(shifted_out, old_carry).unwrap_or(false);
		if shift_in_bit {
			result |= self.direction.shift_in_mask();
		}

		if self.preserve_sign_bit() {
			result = (result & 0x7F) | u8::from(old_sign) << 7;
		}

		let new_carry = shifted_out;
		let new_zero = self.zero_flag_for_result(dst, result);

		let result_change = dst.change_destination(result);
		let bit_flags_change = BitFlagsChange::zero_all()
			.with_carry_flag(new_carry)
			.with_zero_flag(new_zero);

		ChangeList::new(vec![result_change, Box::new(bit_flags_change)])
	}
}

impl Display for ByteShiftOperation {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let direction = match self.direction {
			ShiftDirection::Left => "l",
			ShiftDirection::Right => "r",
		};

		match self.type_ {
			ShiftType::Rotate => {
				write!(f, "r{direction}c")
			}
			ShiftType::RotateWithCarry => {
				write!(f, "r{direction}")
			}
			ShiftType::ArithmeticShift => {
				write!(f, "s{direction}a")
			}
			ShiftType::LogicalShift => {
				write!(f, "s{direction}l")
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::register_bank::SingleRegisters;
	use crate::instructions::changeset::SingleRegisterChange;

	use super::*;

	#[test]
	fn zero_flag() {
		assert_eq!(
			ByteShiftOperation::new(ShiftDirection::Right, ShiftType::Rotate).compute_changes(
				0,
				false,
				&ByteDestination::write_to_acc(),
			),
			ChangeList::new(vec![
				Box::new(SingleRegisterChange::new(ACC_REGISTER, 0)),
				Box::new(BitFlagsChange::zero_all().with_zero_flag(false).with_carry_flag(false)),
			])
		);

		assert_eq!(
			ByteShiftOperation::new(ShiftDirection::Right, ShiftType::Rotate).compute_changes(
				0,
				false,
				&ByteDestination::SingleRegister(SingleRegisters::B),
			),
			ChangeList::new(vec![
				Box::new(SingleRegisterChange::new(SingleRegisters::B, 0)),
				Box::new(BitFlagsChange::zero_all().with_zero_flag(true).with_carry_flag(false)),
			])
		);
	}

	#[test]
	fn rotate() {
		assert_eq!(
			ByteShiftOperation::new(ShiftDirection::Right, ShiftType::Rotate).compute_changes(
				0b1100_1010,
				false,
				&ByteDestination::write_to_acc(),
			),
			ChangeList::new(vec![
				Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b0110_0101)),
				Box::new(BitFlagsChange::zero_all().with_zero_flag(false).with_carry_flag(false)),
			])
		);

		assert_eq!(
			ByteShiftOperation::new(ShiftDirection::Left, ShiftType::Rotate).compute_changes(
				0b1100_1010,
				false,
				&ByteDestination::write_to_acc(),
			),
			ChangeList::new(vec![
				Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b1001_0101)),
				Box::new(BitFlagsChange::zero_all().with_zero_flag(false).with_carry_flag(true)),
			])
		);
	}

	#[test]
	fn rotate_with_carry() {
		assert_eq!(
			ByteShiftOperation::new(ShiftDirection::Right, ShiftType::RotateWithCarry).compute_changes(
				0b0011_1010,
				true,
				&ByteDestination::write_to_acc()
			),
			ChangeList::new(vec![
				Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b1001_1101)),
				Box::new(BitFlagsChange::zero_all().with_zero_flag(false).with_carry_flag(false)),
			])
		);

		assert_eq!(
			ByteShiftOperation::new(ShiftDirection::Left, ShiftType::RotateWithCarry).compute_changes(
				0b1001_1101,
				false,
				&ByteDestination::write_to_acc()
			),
			ChangeList::new(vec![
				Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b0011_1010)),
				Box::new(BitFlagsChange::zero_all().with_zero_flag(false).with_carry_flag(true)),
			])
		);
	}

	#[test]
	fn shift_logical() {
		assert_eq!(
			ByteShiftOperation::new(ShiftDirection::Right, ShiftType::LogicalShift).compute_changes(
				0b0011_1010,
				false,
				&ByteDestination::write_to_acc()
			),
			ChangeList::new(vec![
				Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b0001_1101)),
				Box::new(BitFlagsChange::zero_all().with_zero_flag(false).with_carry_flag(false)),
			])
		);

		assert_eq!(
			ByteShiftOperation::new(ShiftDirection::Left, ShiftType::LogicalShift).compute_changes(
				0b1001_1101,
				false,
				&ByteDestination::write_to_acc(),
			),
			ChangeList::new(vec![
				Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b0011_1010)),
				Box::new(BitFlagsChange::zero_all().with_zero_flag(false).with_carry_flag(true)),
			])
		);
	}

	#[test]
	fn shift_arithmetic() {
		assert_eq!(
			ByteShiftOperation::new(ShiftDirection::Right, ShiftType::ArithmeticShift).compute_changes(
				0b1100_1010,
				false,
				&ByteDestination::write_to_acc()
			),
			ChangeList::new(vec![
				Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b1110_0101)),
				Box::new(BitFlagsChange::zero_all().with_zero_flag(false).with_carry_flag(false)),
			])
		);

		assert_eq!(
			ByteShiftOperation::new(ShiftDirection::Left, ShiftType::ArithmeticShift).compute_changes(
				0b1000_1101,
				false,
				&ByteDestination::write_to_acc()
			),
			ChangeList::new(vec![
				Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b0001_1010)),
				Box::new(BitFlagsChange::zero_all().with_zero_flag(false).with_carry_flag(true)),
			])
		);
	}
}
