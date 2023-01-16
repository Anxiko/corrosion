use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{RegisterFlags, SingleRegisters};
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::arithmetic::ACC_REGISTER;
use crate::instructions::arithmetic::operation::ArithmeticOperation;

pub(super) struct Sub {
	src: SingleRegisters,
}

impl Sub {
	pub(super) fn new(src: SingleRegisters) -> Self {
		Self { src }
	}
}

impl Instruction for Sub {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let src_val = cpu.register_bank.read_single_named(self.src);
		let dst_val = cpu.register_bank.read_single_named(ACC_REGISTER);

		ArithmeticOperation::sub(dst_val, src_val).commit(cpu);

		Ok(())
	}
}

pub(super) struct SubWithCarry {
	src: SingleRegisters,
}

impl SubWithCarry {
	pub(super) fn new(src: SingleRegisters) -> Self {
		Self { src }
	}
}

impl Instruction for SubWithCarry {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let src_val = cpu.register_bank.read_single_named(self.src);
		let dst_val = cpu.register_bank.read_single_named(ACC_REGISTER);
		let carry_flag = cpu.register_bank.read_bit_flag(RegisterFlags::Carry);

		ArithmeticOperation::sub_with_carry(dst_val, src_val, carry_flag).commit(cpu);

		Ok(())
	}
}

pub(super) struct Compare {
	src: SingleRegisters,
}

impl Compare {
	pub(super) fn new(src: SingleRegisters) -> Self {
		Self { src }
	}
}

impl Instruction for Compare {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let src_val = cpu.register_bank.read_single_named(self.src);
		let dst_val = cpu.register_bank.read_single_named(ACC_REGISTER);

		ArithmeticOperation::sub(dst_val, src_val).commit(cpu);
		// Restore ACC, since Compare only updates bit flags
		cpu.register_bank.write_single_named(ACC_REGISTER, dst_val);

		Ok(())
	}
}

pub(super) struct Decrement {}

impl Decrement {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl Instruction for Decrement {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let dst_val = cpu.register_bank.read_single_named(ACC_REGISTER);

		ArithmeticOperation::sub(dst_val, 1).commit(cpu);

		Ok(())
	}
}