use crate::hardware::cpu::Cpu;

pub(crate) mod arithmetic;
#[cfg(test)]
mod tests;

const LOWER_NIBBLE: u8 = 0xF;

trait Instruction {
	fn execute(&self, cpu: &mut Cpu);
}

fn half_carry(left: u8, right: u8) -> bool {
	(left & LOWER_NIBBLE) + (right & LOWER_NIBBLE) > LOWER_NIBBLE
}