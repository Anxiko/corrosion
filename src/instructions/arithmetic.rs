use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{DoubleRegisters, RegisterFlags, SingleRegisters};
use crate::instructions::{half_carry, Instruction, InstructionError};
use crate::hardware::ram::Ram;

pub(crate) const ACC_REGISTER: SingleRegisters = SingleRegisters::A;

pub(crate) struct Add {
	src: SingleRegisters,
}

impl Add {
	pub(crate) fn new(src: SingleRegisters) -> Self {
		Self { src }
	}
}

fn base_adder(cpu: &mut Cpu, left: u8, right: u8) {
	let (result, overflow) = left.overflowing_add(right);
	cpu.register_bank.write_single_named(ACC_REGISTER, result);

	cpu.register_bank.write_bit_flag(RegisterFlags::Zero, result == 0);
	cpu.register_bank.write_bit_flag(RegisterFlags::Subtraction, false);
	cpu.register_bank.write_bit_flag(RegisterFlags::Carry, overflow);
	cpu.register_bank.write_bit_flag(RegisterFlags::HalfCarry, half_carry(left, right));
}

impl Instruction for Add {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), InstructionError> {
		let registers = &mut cpu.register_bank;
		let dst_val = registers.read_single_named(ACC_REGISTER);
		let src_val = registers.read_single_named(self.src);

		base_adder(cpu, dst_val, src_val);

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

		base_adder(cpu, dst_val, src_val);

		Ok(())
	}
}