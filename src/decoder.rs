use crate::bits::byte_to_bits;
use crate::decoder::prefixed::{decode_prefixed_shifting, decode_prefixed_single_bit};
use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::base::ByteSource;
use crate::instructions::single_bit::SingleBitOperation;

mod prefixed;

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
				[false, false] /* x = 0 */ => Ok(decode_prefixed_shifting(y, z)),
				[true, false] /* x = 1 */ => Ok(decode_prefixed_single_bit(
					SingleBitOperation::Test, y, z,
				)),
				[false, true] /* x = 2 */ => Ok(decode_prefixed_single_bit(
					SingleBitOperation::Write(false), y, z,
				)),
				[true, true] /* x = 3 */ => Ok(decode_prefixed_single_bit(
					SingleBitOperation::Write(true), y, z,
				)),
			}
		},
		None => todo!()
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum DecodedInstructionOperand {
	SingleRegister(SingleRegisters),
	HlMemoryAddress,
}

impl DecodedInstructionOperand {
	fn from_opcode_part(opcode_part: [bool; 3]) -> Self {
		match opcode_part {
			[false, false, false] => Self::SingleRegister(SingleRegisters::B), // 0 => B
			[true, false, false] => Self::SingleRegister(SingleRegisters::C), // 1 => C
			[false, true, false] => Self::SingleRegister(SingleRegisters::D), // 2 => D
			[true, true, false] => Self::SingleRegister(SingleRegisters::E), // 3 => E
			[false, false, true] => Self::SingleRegister(SingleRegisters::H), // 4 => H
			[true, false, true] => Self::SingleRegister(SingleRegisters::L), // 5 => L
			[false, true, true] => Self::HlMemoryAddress, // 6 => (HL)
			[true, true, true] => Self::SingleRegister(SingleRegisters::A), // 7 => A
		}
	}
}


impl From<DecodedInstructionOperand> for ByteSource {
	fn from(value: DecodedInstructionOperand) -> Self {
		match value {
			DecodedInstructionOperand::SingleRegister(single_reg) => Self::read_from_single(single_reg),
			DecodedInstructionOperand::HlMemoryAddress => Self::read_from_register_address(DoubleRegisters::HL)
		}
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