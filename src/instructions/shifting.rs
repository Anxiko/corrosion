use std::assert_matches::assert_matches;

use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{BitFlags, SingleRegisters};
use crate::instructions::{ACC_REGISTER, ExecutionError, Instruction};
use crate::instructions::base::{BaseByteInstruction, ByteDestination, ByteOperation, ByteSource};
use crate::instructions::changeset::{Change, ChangeList, SingleRegisterChange};
use crate::instructions::shifting::operation::{ByteShiftOperation, ShiftDirection, ShiftType};

mod operation;

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

type ByteShiftInstruction = BaseByteInstruction<ByteShiftOperation>;

pub(crate) struct ByteSwapOperation {}

impl ByteSwapOperation {
	pub(crate) fn new() -> Self { Self {} }
}

impl ByteOperation for ByteSwapOperation {
	type C = Box<dyn Change>;

	fn execute(&self, cpu: &Cpu, src: &ByteSource, dst: &ByteDestination) -> Result<Self::C, ExecutionError> {
		let byte = src.read(cpu)?;
		let high = byte & 0xF0;
		let low = byte & 0x0F;

		let result = (high >> 4) | (low << 4);
		Ok(dst.change_destination(result))
	}
}

pub(crate) type ByteSwapInstruction = BaseByteInstruction<ByteSwapOperation>;

#[test]
fn shift_instruction() {
	let mut cpu = Cpu::new();
	let old_carry = true;
	cpu.register_bank
		.write_single_named(ACC_REGISTER, 0b0011_0101);
	cpu.register_bank.write_bit_flag(BitFlags::Carry, old_carry);

	let mut expected = cpu.clone();
	expected
		.register_bank
		.write_single_named(ACC_REGISTER, 0b0110_1011);
	expected
		.register_bank
		.write_bit_flag(BitFlags::Carry, false);

	let instruction = ByteShiftInstruction::new(
		ByteSource::Acc,
		ByteDestination::Acc,
		ByteShiftOperation::new(ShiftDirection::Left, ShiftType::RotateWithCarry),
	);


	assert_matches!(instruction.execute(&mut cpu), Ok(()));
	assert_eq!(cpu, expected);
}

#[test]
fn swap_operation() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(SingleRegisters::B, 0b0011_0101);

	let operation = ByteSwapOperation::new();

	let actual = operation.execute(
		&cpu,
		&ByteSource::read_from_single(SingleRegisters::B),
		&ByteDestination::Acc,
	).expect("Operation to execute");

	let expected: Box<dyn Change> = Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b0101_0011));

	assert_eq!(actual, expected);
}
