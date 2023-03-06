use crate::hardware::cpu::Cpu;
use crate::instructions::{ExecutionError, Instruction};

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
	let opbyte: u8;

	if prefix.is_some() {
		opbyte = cpu.next_byte()?;
	} else {
		opbyte = first_byte;
	}

	todo!()
}

impl From<ExecutionError> for DecoderError {
	fn from(value: ExecutionError) -> Self {
		DecoderError::ExecutionError(value)
	}
}