use operation::ArithmeticOperation;

use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use crate::instructions::{ExecutionError, Instruction};

#[cfg(test)]
mod tests;

mod operation;

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

impl Instruction for Add {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
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
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let src_address = cpu.register_bank.read_double_named(DoubleRegisters::HL);
		let src_val = cpu.mapped_ram.read(src_address)?;
		let dst_val = cpu.register_bank.read_single_named(ACC_REGISTER);

		ArithmeticOperation::add(dst_val, src_val).commit(cpu);

		Ok(())
	}
}

pub(crate) struct AddImmediate {}

impl AddImmediate {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl Instruction for AddImmediate {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let src_address = cpu.next_pc();
		let src_val = cpu.mapped_ram.read(src_address)?;
		let dst_val = cpu.register_bank.read_single_named(ACC_REGISTER);

		ArithmeticOperation::add(dst_val, src_val).commit(cpu);

		Ok(())
	}
}
