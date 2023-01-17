use operation::{AsShiftOperation, ShiftDestination, ShiftDirection, ShiftOperation, ShiftType};

use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::RegisterFlags;
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::ACC_REGISTER;

mod operation;

struct RotateLeft {}

impl RotateLeft {
	fn new() -> Self {
		Self {}
	}
}

impl AsShiftOperation for RotateLeft {
	fn as_shift_operation(&self, cpu: &mut Cpu) -> ShiftOperation {
		ShiftOperation::new(
			cpu.register_bank.read_single_named(ACC_REGISTER),
			ShiftDestination::Acc,
			ShiftDirection::Left,
			ShiftType::Rotate,
		)
	}
}

struct RotateLeftWithCarry {}

impl RotateLeftWithCarry {
	fn new() -> Self {
		Self {}
	}
}

impl AsShiftOperation for RotateLeftWithCarry {
	fn as_shift_operation(&self, cpu: &mut Cpu) -> ShiftOperation {
		ShiftOperation::new(
			cpu.register_bank.read_single_named(ACC_REGISTER),
			ShiftDestination::Acc,
			ShiftDirection::Left,
			ShiftType::RotateWithCarry { old_carry: cpu.register_bank.read_bit_flag(RegisterFlags::Carry) },
		)
	}
}

struct RotateRight {}

impl RotateRight {
	fn new() -> Self {
		Self {}
	}
}

impl Instruction for RotateRight {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let result = cpu.register_bank.read_single_named(ACC_REGISTER);
		let lowest_bit = result & 0x01 != 0;

		let result = result.rotate_right(1);

		cpu.register_bank.write_single_named(ACC_REGISTER, result);
		cpu.register_bank.write_bit_flag(RegisterFlags::Carry, lowest_bit);

		Ok(())
	}
}

struct RotateRightWithCarry {}

impl RotateRightWithCarry {
	fn new() -> Self {
		Self {}
	}
}

impl Instruction for RotateRightWithCarry {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let old_carry = cpu.register_bank.read_bit_flag(RegisterFlags::Carry);
		let result = cpu.register_bank.read_single_named(ACC_REGISTER);
		let lowest_bit = result & 0x01 != 0;

		let result = (result >> 1) | (u8::from(old_carry) * 0x80);

		cpu.register_bank.write_single_named(ACC_REGISTER, result);
		cpu.register_bank.write_bit_flag(RegisterFlags::Carry, lowest_bit);

		Ok(())
	}
}

#[test]
fn rotate_left() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0b1010_1010);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b0101_0101);
	expected.register_bank.write_bit_flag(RegisterFlags::Carry, true);

	assert!(RotateLeft::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);

	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0b0101_0101);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b1010_1010);
	expected.register_bank.write_bit_flag(RegisterFlags::Carry, false);

	assert!(RotateLeft::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}

#[test]
fn rotate_left_with_carry() {
	let mut cpu = Cpu::new();

	cpu.register_bank.write_single_named(ACC_REGISTER, 0b1010_1010);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b0101_0100);
	expected.register_bank.write_bit_flag(RegisterFlags::Carry, true);

	assert!(RotateLeftWithCarry::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);

	let mut cpu = Cpu::new();

	cpu.register_bank.write_single_named(ACC_REGISTER, 0b0101_0101);
	cpu.register_bank.write_bit_flag(RegisterFlags::Carry, true);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b1010_1011);
	expected.register_bank.write_bit_flag(RegisterFlags::Carry, false);

	assert!(RotateLeftWithCarry::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}


#[test]
fn rotate_right() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0b1010_1010);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b0101_0101);
	expected.register_bank.write_bit_flag(RegisterFlags::Carry, false);

	assert!(RotateRight::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);

	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0b0101_0101);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b1010_1010);
	expected.register_bank.write_bit_flag(RegisterFlags::Carry, true);

	assert!(RotateRight::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}

#[test]
fn rotate_right_with_carry() {
	let mut cpu = Cpu::new();

	cpu.register_bank.write_single_named(ACC_REGISTER, 0b1010_1010);
	cpu.register_bank.write_bit_flag(RegisterFlags::Carry, true);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b1101_0101);
	expected.register_bank.write_bit_flag(RegisterFlags::Carry, false);

	assert!(RotateRightWithCarry::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);

	let mut cpu = Cpu::new();

	cpu.register_bank.write_single_named(ACC_REGISTER, 0b0101_0101);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b0010_1010);
	expected.register_bank.write_bit_flag(RegisterFlags::Carry, true);

	assert!(RotateRightWithCarry::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}