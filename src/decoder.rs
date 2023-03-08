use crate::bytes::byte_to_bits;
use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::base::{ByteDestination, ByteSource};
use crate::instructions::shifting::{ByteShiftInstruction, ByteShiftOperation, ShiftDirection, ShiftType};

enum DecoderError {
	ExecutionError(ExecutionError)
}

enum DecoderState {
	Empty,
	WithPrefix { prefix: DecodedInstructionPrefix },
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum DecodedInstructionPrefix {
	CB
}

impl DecodedInstructionPrefix {
	fn try_decode_prefix(maybe_prefix: u8) -> Option<DecodedInstructionPrefix> {
		match maybe_prefix {
			0xCB => Some(Self::CB),
			_ => None
		}
	}
}

fn decoder(cpu: &mut Cpu) -> Result<Box<dyn Instruction>, DecoderError> {
	let first_byte = cpu.next_byte()?;

	let prefix = DecodedInstructionPrefix::try_decode_prefix(first_byte);
	let opcode: u8;

	if prefix.is_some() {
		opcode = cpu.next_byte()?;
	} else {
		opcode = first_byte;
	}

	decode_opcode(prefix, opcode, cpu)
}

fn decode_opcode(
	prefix: Option<DecodedInstructionPrefix>, opcode: u8, cpu: &mut Cpu,
) -> Result<Box<dyn Instruction>, DecoderError> {
	let (x, y, z) = decode_xyz(opcode);

	match prefix {
		Some(DecodedInstructionPrefix::CB) => {
			match x {
				[false, false] /* x = 0 */ => {
					match y {
						[y0, y1, false] => /* 0 <= y < 4 */{
							let shift_direction = match y0 {
								false => ShiftDirection::Left,
								true => ShiftDirection::Right
							};
							let shift_type = match y1 {
								false => ShiftType::Rotate,
								true => ShiftType::RotateWithCarry
							};

							let source = decode_byte_source(z);

							Ok(Box::new(ByteShiftInstruction::new(
								source,
								ByteDestination::Acc,
								ByteShiftOperation::new(ShiftDirection::Left, ShiftType::Rotate),
							)))
						},
						[false, false, true] => /* y = 4 */ todo!(),
						[true, false, true] => /* y = 5 */ todo!(),
						[false, true, true] => /* y = 6 */ todo!(),
						[true, true, true] => /* y = 7 */ todo!(),
					}
				},
				[true, false] /* x = 1 */ => todo!(),
				[false, true] /* x = 2 */ => todo!(),
				[true, true] /* x = 3 */ => todo!(),
			}
		},
		None => todo!()
	}
}

fn decode_byte_source(opcode_part: [bool; 3]) -> ByteSource {
	match opcode_part {
		[false, false, false] => ByteSource::read_from_single(SingleRegisters::B), // 0 => B
		[true, false, false] => ByteSource::read_from_single(SingleRegisters::C), // 1 => C
		[false, true, false] => ByteSource::read_from_single(SingleRegisters::D), // 2 => D
		[true, true, false] => ByteSource::read_from_single(SingleRegisters::E), // 3 => E
		[false, false, true] => ByteSource::read_from_single(SingleRegisters::H), // 4 => H
		[true, false, true] => ByteSource::read_from_single(SingleRegisters::L), // 5 => L
		[false, true, true] => ByteSource::read_from_register_address(DoubleRegisters::HL), // 6 => (HL)
		[true, true, true] => ByteSource::read_from_single(SingleRegisters::A),
	}
}

fn decode_xyz(opcode: u8) -> ([bool; 2], [bool; 3], [bool; 3]) {
	let bits = byte_to_bits(opcode);

	let x = bits[6..8].try_into().unwrap();
	let y = bits[3..6].try_into().unwrap();
	let z = bits[0..3].try_into().unwrap();

	(x, y, z)
}

impl From<ExecutionError> for DecoderError {
	fn from(value: ExecutionError) -> Self {
		DecoderError::ExecutionError(value)
	}
}