use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{RegisterFlags, SingleRegisters};
use crate::instructions::{ACC_REGISTER, ExecutionError, Instruction};

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

impl ShiftDestination {
	fn as_single_named_register(self) -> SingleRegisters {
		match self {
			Self::Acc => ACC_REGISTER,
			Self::Single(register) => register
		}
	}
}

pub(super) struct ShiftOperation {
	value: u8,
	destination: ShiftDestination,
	direction: ShiftDirection,
	type_: ShiftType,
}

impl ShiftOperation {
	pub(super) fn new(value: u8, destination: ShiftDestination, direction: ShiftDirection, type_: ShiftType) -> Self {
		Self {
			value,
			destination,
			direction,
			type_,
		}
	}
}

impl ShiftOperation {
	pub(super) fn calculate(&self) -> ShiftOperationResult {
		let old_sign = self.value & 80 != 0;
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
				(self.value << 1, self.value & 0x80 != 0)
			}
			ShiftDirection::Right => {
				(self.value >> 1, self.value & 0x01 != 0)
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

impl ShiftOperationResult {
	fn commit(&self, cpu: &mut Cpu) {
		cpu.register_bank.write_single_named(self.destination.as_single_named_register(), self.result);
		cpu.register_bank.write_bit_flag(RegisterFlags::Carry, self.new_carry);
		cpu.register_bank.write_bit_flag(RegisterFlags::Zero, self.new_zero);
		cpu.register_bank.write_bit_flag(RegisterFlags::HalfCarry, false);
		cpu.register_bank.write_bit_flag(RegisterFlags::Subtraction, false);
	}
}

pub(super) trait AsShiftOperation {
	fn as_shift_operation(&self, cpu: &mut Cpu) -> ShiftOperation;
}

impl<T> Instruction for T
	where T: AsShiftOperation
{
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let shift_operation = self.as_shift_operation(cpu);
		let operation_result = shift_operation.calculate();
		operation_result.commit(cpu);

		Ok(())
	}
}

#[test]
fn zero_flag() {
	assert_eq!(
		ShiftOperation {
			value: 0,
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
			value: 0,
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
			value: 0b1100_1010,
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
			value: 0b1100_1010,
			destination: ShiftDestination::Acc,
			direction: ShiftDirection::Left,
			type_: ShiftType::Rotate,
		}.calculate(),
		ShiftOperationResult {
			result: 0b1001_0101,
			destination: ShiftDestination::Acc,
			new_carry: true,
			new_zero: false,
		}
	)
}

#[test]
fn rotate_with_carry() {
	assert_eq!(
		ShiftOperation {
			value: 0b0011_1010,
			destination: ShiftDestination::Acc,
			type_: ShiftType::RotateWithCarry { old_carry: true },
			direction: ShiftDirection::Right,
		}.calculate(),
		ShiftOperationResult {
			result: 0b1001_1101,
			destination: ShiftDestination::Acc,
			new_carry: false,
			new_zero: false,
		}
	);

	assert_eq!(
		ShiftOperation {
			value: 0b1001_1101,
			destination: ShiftDestination::Acc,
			type_: ShiftType::RotateWithCarry { old_carry: false },
			direction: ShiftDirection::Left,
		}.calculate(),
		ShiftOperationResult {
			result: 0b0011_1010,
			destination: ShiftDestination::Acc,
			new_carry: true,
			new_zero: false,
		}
	);
}

#[test]
fn shift_logical() {
	assert_eq!(
		ShiftOperation {
			value: 0b0011_1010,
			destination: ShiftDestination::Acc,
			type_: ShiftType::LogicalShift,
			direction: ShiftDirection::Right,
		}.calculate(),
		ShiftOperationResult {
			result: 0b0001_1101,
			destination: ShiftDestination::Acc,
			new_carry: false,
			new_zero: false,
		}
	);

	assert_eq!(
		ShiftOperation {
			value: 0b1001_1101,
			destination: ShiftDestination::Acc,
			type_: ShiftType::LogicalShift,
			direction: ShiftDirection::Left,
		}.calculate(),
		ShiftOperationResult {
			result: 0b0011_1010,
			destination: ShiftDestination::Acc,
			new_carry: true,
			new_zero: false,
		}
	);
}

#[test]
fn shift_arithmetic() {
	assert_eq!(
		ShiftOperation {
			value: 0b1100_1010,
			destination: ShiftDestination::Acc,
			type_: ShiftType::ArithmeticShift,
			direction: ShiftDirection::Right,
		}.calculate(),
		ShiftOperationResult {
			result: 0b1110_0101,
			destination: ShiftDestination::Acc,
			new_carry: false,
			new_zero: false,
		}
	);

	assert_eq!(
		ShiftOperation {
			value: 0b1000_1101,
			destination: ShiftDestination::Acc,
			type_: ShiftType::ArithmeticShift,
			direction: ShiftDirection::Left,
		}.calculate(),
		ShiftOperationResult {
			result: 0b0001_1010,
			destination: ShiftDestination::Acc,
			new_carry: true,
			new_zero: false,
		}
	);
}