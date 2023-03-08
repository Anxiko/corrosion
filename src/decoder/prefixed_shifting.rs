use crate::decoder::decode_byte_source;
use crate::instructions::base::ByteDestination;
use crate::instructions::Instruction;
use crate::instructions::shifting::{ByteShiftInstruction, ByteShiftOperation, ByteSwapInstruction, ByteSwapOperation, ShiftDirection, ShiftType};

pub(super) fn decode_prefixed_shifting(y: [bool; 3], z: [bool; 3]) -> Box<dyn Instruction> {
	let source = decode_byte_source(z);

	let shift_direction = match y[0] {
		false => ShiftDirection::Left,
		true => ShiftDirection::Right
	};

	let shift_type = match y {
		[_, false, false] /* 0 <= y < 2 */ => ShiftType::Rotate,
		[_, true, false] /* 2 <= y < 4 */ => ShiftType::RotateWithCarry,
		[_, false, true]  /* 4 <= y < 6 */ => ShiftType::ArithmeticShift,
		[_, true, true] /* 6 <= y < 8 */ => ShiftType::LogicalShift
	};

	match (shift_type, shift_direction) {
		(ShiftType::LogicalShift, ShiftDirection::Left) => Box::new(ByteSwapInstruction::new(
			source, ByteDestination::Acc, ByteSwapOperation::new(),
		)), // Logical left shift does not exist, instead this encodes a swap instruction
		(_, _) => Box::new(ByteShiftInstruction::new(
			source, ByteDestination::Acc, ByteShiftOperation::new(shift_direction, shift_type),
		))
	}
}