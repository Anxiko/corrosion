use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{BitFlags, SingleRegisters};
use crate::instructions::{ACC_REGISTER, ExecutionError, Instruction};
use crate::instructions::base::{ByteDestination, ByteOperation, ByteSource};
use crate::instructions::changeset::{BitFlagsChangeset, Change, ChangeList, SingleRegisterChange};

#[derive(Copy, Clone)]
pub(super) enum ShiftDirection {
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

#[derive(Copy, Clone)]
pub(super) enum ShiftType {
	Rotate,
	RotateWithCarry,
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
			Self::Single(register) => register,
		}
	}
}

pub(super) struct ShiftOperation {
	value: u8,
	old_carry: bool,
	destination: ShiftDestination,
	direction: ShiftDirection,
	type_: ShiftType,
}

impl ShiftOperation {
	pub(super) fn new(
		value: u8,
		old_carry: bool,
		destination: ShiftDestination,
		direction: ShiftDirection,
		type_: ShiftType,
	) -> Self {
		Self {
			value,
			old_carry,
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
			ShiftDirection::Left => (self.value << 1, self.value & 0x80 != 0),
			ShiftDirection::Right => (self.value >> 1, self.value & 0x01 != 0),
		}
	}

	fn zero_flag_for_result(&self, result: u8) -> bool {
		match self.destination {
			ShiftDestination::Acc => false,
			_ => result == 0,
		}
	}

	fn shift_in(&self, shifted_out: bool) -> Option<bool> {
		match self.type_ {
			ShiftType::Rotate => Some(shifted_out),
			ShiftType::RotateWithCarry => Some(self.old_carry),
			_ => None,
		}
	}

	fn preserve_sign_bit(&self) -> bool {
		matches!(
            (self.type_, self.direction),
            (ShiftType::ArithmeticShift, ShiftDirection::Right)
        )
	}
}

#[derive(PartialEq, Debug)]
pub(super) struct ShiftOperationResult {
	result: u8,
	destination: ShiftDestination,
	new_carry: bool,
	new_zero: bool,
}

impl Into<ChangeList> for &ShiftOperationResult {
	fn into(self) -> ChangeList {
		ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(
				self.destination.as_single_named_register(),
				self.result,
			)),
			Box::new(
				BitFlagsChangeset::zero_all()
					.with_carry_flag(self.new_carry)
					.with_zero_flag(self.new_zero),
			),
		])
	}
}

impl ShiftOperationResult {
	fn commit(&self, cpu: &mut Cpu) {
		cpu.register_bank
			.write_single_named(self.destination.as_single_named_register(), self.result);
		cpu.register_bank
			.write_bit_flag(BitFlags::Carry, self.new_carry);
		cpu.register_bank
			.write_bit_flag(BitFlags::Zero, self.new_zero);
		cpu.register_bank.write_bit_flag(BitFlags::HalfCarry, false);
		cpu.register_bank
			.write_bit_flag(BitFlags::Subtraction, false);
	}
}

pub(super) trait AsShiftOperation {
	fn as_shift_operation(&self, cpu: &mut Cpu) -> ShiftOperation;
}

struct ByteShiftOperation {
	direction: ShiftDirection,
	type_: ShiftType,
}

impl ByteShiftOperation {
	fn new(direction: ShiftDirection, type_: ShiftType) -> Self {
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
			ByteDestination::Acc => false,
			_ => result == 0,
		}
	}

	pub(super) fn compute_changes(
		&self,
		value: u8,
		old_carry: bool,
		dst: &ByteDestination,
	) -> ChangeList {
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

		let single_register_change = dst.change_destination(result);
		let bit_flags_change = BitFlagsChangeset::zero_all()
			.with_carry_flag(new_carry)
			.with_zero_flag(new_zero);

		ChangeList::new(vec![
			Box::new(single_register_change),
			Box::new(bit_flags_change),
		])
	}
}

impl ByteOperation for ByteShiftOperation {
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

#[test]
fn zero_flag() {
	assert_eq!(
		ByteShiftOperation::new(ShiftDirection::Right, ShiftType::Rotate).compute_changes(
			0,
			false,
			&ByteDestination::Acc,
		),
		ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0)),
			Box::new(
				BitFlagsChangeset::zero_all()
					.with_zero_flag(false)
					.with_carry_flag(false)
			),
		])
	);

	assert_eq!(
		ByteShiftOperation::new(ShiftDirection::Right, ShiftType::Rotate).compute_changes(
			0,
			false,
			&ByteDestination::SingleRegister {
				single_reg: SingleRegisters::B
			},
		),
		ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(SingleRegisters::B, 0)),
			Box::new(
				BitFlagsChangeset::zero_all()
					.with_zero_flag(true)
					.with_carry_flag(false)
			),
		])
	);
}

#[test]
fn rotate() {
	assert_eq!(
		ByteShiftOperation::new(ShiftDirection::Right, ShiftType::Rotate).compute_changes(
			0b1100_1010,
			false,
			&ByteDestination::Acc,
		),
		ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b0110_0101)),
			Box::new(BitFlagsChangeset::zero_all().with_zero_flag(false).with_carry_flag(false)),
		])
	);

	assert_eq!(
		ByteShiftOperation::new(ShiftDirection::Left, ShiftType::Rotate).compute_changes(
			0b1100_1010,
			false,
			&ByteDestination::Acc,
		),
		ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b1001_0101)),
			Box::new(BitFlagsChangeset::zero_all().with_zero_flag(false).with_carry_flag(true)),
		])
	);
}

#[test]
fn rotate_with_carry() {
	assert_eq!(
		ByteShiftOperation::new(ShiftDirection::Right, ShiftType::RotateWithCarry).compute_changes(
			0b0011_1010,
			true,
			&ByteDestination::Acc,
		),
		ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b1001_1101)),
			Box::new(BitFlagsChangeset::zero_all().with_zero_flag(false).with_carry_flag(false)),
		])
	);

	assert_eq!(
		ByteShiftOperation::new(ShiftDirection::Left, ShiftType::RotateWithCarry).compute_changes(
			0b1001_1101,
			false,
			&ByteDestination::Acc,
		),
		ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b0011_1010)),
			Box::new(BitFlagsChangeset::zero_all().with_zero_flag(false).with_carry_flag(true)),
		])
	);
}

#[test]
fn shift_logical() {
	assert_eq!(
		ByteShiftOperation::new(ShiftDirection::Right, ShiftType::LogicalShift).compute_changes(
			0b0011_1010,
			false,
			&ByteDestination::Acc,
		),
		ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b0001_1101)),
			Box::new(BitFlagsChangeset::zero_all().with_zero_flag(false).with_carry_flag(false)),
		])
	);

	assert_eq!(
		ByteShiftOperation::new(ShiftDirection::Left, ShiftType::LogicalShift).compute_changes(
			0b1001_1101,
			false,
			&ByteDestination::Acc,
		),
		ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b0011_1010)),
			Box::new(BitFlagsChangeset::zero_all().with_zero_flag(false).with_carry_flag(true)),
		])
	);
}

#[test]
fn shift_arithmetic() {
	assert_eq!(
		ShiftOperation {
			value: 0b1100_1010,
			old_carry: false,
			destination: ShiftDestination::Acc,
			type_: ShiftType::ArithmeticShift,
			direction: ShiftDirection::Right,
		}
			.calculate(),
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
			old_carry: false,
			destination: ShiftDestination::Acc,
			type_: ShiftType::ArithmeticShift,
			direction: ShiftDirection::Left,
		}
			.calculate(),
		ShiftOperationResult {
			result: 0b0001_1010,
			destination: ShiftDestination::Acc,
			new_carry: true,
			new_zero: false,
		}
	);
}
