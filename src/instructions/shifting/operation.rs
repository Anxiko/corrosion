use std::thread::sleep;
use crate::hardware::register_bank::SingleRegisters;

#[derive(Copy, Clone)]
pub(super) enum ShiftDirection {
	Left,
	Right,
}

impl ShiftDirection {
	fn shift_in_mask(self) -> u8 {
		match self {
			Self::Left => 0x01,
			Self::Right => 0x80
		}
	}
}

#[derive(Copy, Clone)]
pub(super) enum ShiftType {
	Rotate,
	RotateWithCarry { old_carry: bool },
	LogicalShift,
	ArithmeticShift,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) enum ShiftDestination {
	Acc,
	Single(SingleRegisters),
}

pub(super) struct ShiftOperation {
	src: u8,
	destination: ShiftDestination,
	direction: ShiftDirection,
	type_: ShiftType,
}

impl ShiftOperation {
	pub(super) fn calculate(&self) -> ShiftOperationResult {
		let old_sign = self.src & 80 != 0;
		let (mut result, shifted_out) = self.shift_result();

		let shift_in_bit = self.shift_in(shifted_out).unwrap_or(false);
		if shift_in_bit {
			result |= self.direction.shift_in_mask();
		}

		if self.preserve_sign_bit() {
			result = (result & 0x7F) | u8::from(old_sign) << 7;
		}

		ShiftOperationResult {
			result,
			destination: self.destination,
			new_carry: shifted_out,
			new_zero: self.zero_flag_for_result(result),
		}
	}

	fn shift_result(&self) -> (u8, bool) {
		match self.direction {
			ShiftDirection::Left => {
				(self.src << 1, self.src & 0x80 != 0)
			}
			ShiftDirection::Right => {
				(self.src >> 1, self.src & 0x01 != 0)
			}
		}
	}

	fn zero_flag_for_result(&self, result: u8) -> bool {
		match self.destination {
			ShiftDestination::Acc => false,
			_ => result == 0
		}
	}

	fn shift_in(&self, shifted_out: bool) -> Option<bool> {
		match self.type_ {
			ShiftType::Rotate => {
				Some(shifted_out)
			}
			ShiftType::RotateWithCarry { old_carry } => {
				Some(old_carry)
			}
			_ => None
		}
	}

	fn preserve_sign_bit(&self) -> bool {
		matches!((self.type_, self.direction), (ShiftType::ArithmeticShift, ShiftDirection::Right))
	}
}

#[derive(PartialEq, Debug)]
pub(super) struct ShiftOperationResult {
	result: u8,
	destination: ShiftDestination,
	new_carry: bool,
	new_zero: bool,
}

#[test]
fn zero_flag() {
	assert_eq!(
		ShiftOperation {
			src: 0,
			destination: ShiftDestination::Acc,
			direction: ShiftDirection::Right,
			type_: ShiftType::Rotate,
		}.calculate(),
		ShiftOperationResult {
			result: 0,
			destination: ShiftDestination::Acc,
			new_zero: false,
			new_carry: false,
		}
	);

	assert_eq!(
		ShiftOperation {
			src: 0,
			destination: ShiftDestination::Single(SingleRegisters::B),
			direction: ShiftDirection::Right,
			type_: ShiftType::Rotate,
		}.calculate(),
		ShiftOperationResult {
			result: 0,
			destination: ShiftDestination::Single(SingleRegisters::B),
			new_zero: true,
			new_carry: false,
		}
	);
}

#[test]
fn rotate() {
	assert_eq!(
		ShiftOperation {
			src: 0b1100_1010,
			destination: ShiftDestination::Acc,
			direction: ShiftDirection::Right,
			type_: ShiftType::Rotate,
		}.calculate(),
		ShiftOperationResult {
			result: 0b0110_0101,
			destination: ShiftDestination::Acc,
			new_carry: false,
			new_zero: false,
		}
	);

	assert_eq!(
		ShiftOperation {
			src: 0b1100_1010,
			destination: ShiftDestination::Single(SingleRegisters::B),
			direction: ShiftDirection::Left,
			type_: ShiftType::Rotate,
		}.calculate(),
		ShiftOperationResult {
			result: 0b1001_0101,
			destination: ShiftDestination::Single(SingleRegisters::B),
			new_carry: true,
			new_zero: false,
		}
	)
}