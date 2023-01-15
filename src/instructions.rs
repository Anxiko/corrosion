use crate::hardware::cpu::Cpu;
use crate::hardware::ram::RamError;

pub(crate) mod arithmetic;
#[cfg(test)]
mod tests;

const LOWER_NIBBLE: u8 = 0xF;

trait Instruction {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), InstructionError>;
}

pub(crate) enum InstructionError {
	RamError(RamError)
}

impl From<RamError> for InstructionError {
	fn from(ram_error: RamError) -> Self {
		Self::RamError(ram_error)
	}
}

fn half_carry(left: u8, right: u8) -> bool {
	(left & LOWER_NIBBLE) + (right & LOWER_NIBBLE) > LOWER_NIBBLE
}