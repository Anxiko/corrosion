use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::SingleRegisters;
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