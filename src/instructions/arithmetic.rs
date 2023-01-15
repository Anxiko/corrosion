use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{DoubleRegisters, RegisterFlags, SingleRegisters};
use crate::instructions::{Instruction, InstructionError};
use crate::hardware::ram::Ram;

#[cfg(test)]
mod tests;

pub(crate) const ACC_REGISTER: SingleRegisters = SingleRegisters::A;

const LOWER_NIBBLE: u8 = 0xF;

pub(crate) struct Add {
	src: SingleRegisters,
}

impl Add {
	pub(crate) fn new(src: SingleRegisters) -> Self {
		Self { src }
	}
}

pub(crate) struct ArithmeticOperation {
	result: u8,
	zero: bool,
	subtraction: bool,
	carry: bool,
	half_carry: bool,
}

impl ArithmeticOperation {
	pub(crate) fn add(left: u8, right: u8) -> Self {
		let (result, overflow) = left.overflowing_add(right);

		Self {
			result,
			zero: result == 0,
			subtraction: false,
			carry: overflow,
			half_carry: Self::half_carry(left, right),
		}
	}

	fn commit(&self, cpu: &mut Cpu) {
		cpu.register_bank.write_single_named(ACC_REGISTER, self.result);

		cpu.register_bank.write_bit_flag(RegisterFlags::Zero, self.zero);
		cpu.register_bank.write_bit_flag(RegisterFlags::Subtraction, self.subtraction);
		cpu.register_bank.write_bit_flag(RegisterFlags::Carry, self.carry);
		cpu.register_bank.write_bit_flag(RegisterFlags::HalfCarry, self.half_carry);
	}

	fn half_carry(left: u8, right: u8) -> bool {
		(left & LOWER_NIBBLE) + (right & LOWER_NIBBLE) > LOWER_NIBBLE
	}
}

impl Instruction for Add {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), InstructionError> {
		let registers = &mut cpu.register_bank;
		let dst_val = registers.read_single_named(ACC_REGISTER);
		let src_val = registers.read_single_named(self.src);

		ArithmeticOperation::add(dst_val, src_val).commit(cpu);

		Ok(())
	}
}

struct AddHl;

impl AddHl {
	fn new() -> Self {
		Self {}
	}
}

impl Instruction for AddHl {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), InstructionError> {
		let src_address = cpu.register_bank.read_double_named(DoubleRegisters::HL);
		let src_val = cpu.mapped_ram.read(src_address)?;
		let dst_val = cpu.register_bank.read_single_named(ACC_REGISTER);

		ArithmeticOperation::add(dst_val, src_val).commit(cpu);

		Ok(())
	}
}
