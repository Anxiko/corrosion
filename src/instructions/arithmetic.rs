use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{RegisterFlags, SingleRegisters};
use crate::instructions::{half_carry, Instruction};

pub(crate) struct Add {
	src: SingleRegisters,
}

impl Add {
	pub(crate) fn new(src: SingleRegisters) -> Self {
		Self { src }
	}
}

impl Instruction for Add {
	fn execute(&self, cpu: &mut Cpu) {
		let registers = &mut cpu.register_bank;
		let dst_val = registers.read_single_named(SingleRegisters::A);
		let src_val = registers.read_single_named(self.src);

		let (result, overflow) = dst_val.overflowing_add(src_val);

		registers.write_bit_flag(RegisterFlags::Zero, result == 0);
		registers.write_bit_flag(RegisterFlags::Subtraction, false);
		registers.write_bit_flag(RegisterFlags::Carry, overflow);
		registers.write_bit_flag(RegisterFlags::HalfCarry, half_carry(src_val, dst_val));
	}
}