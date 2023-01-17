use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::SingleRegisters;
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::ACC_REGISTER;

pub(crate) struct And {
	src: SingleRegisters,
}

impl And {
	pub(crate) fn new(src: SingleRegisters) -> Self {
		Self { src }
	}
}

impl Instruction for And {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let dst_val = cpu.register_bank.read_single_named(ACC_REGISTER);
		let src_val = cpu.register_bank.read_single_named(self.src);

		let result = src_val & dst_val;
		cpu.register_bank.write_single_named(ACC_REGISTER, result);

		Ok(())
	}
}

pub(crate) struct Or {
	src: SingleRegisters,
}

impl Or {
	pub(crate) fn new(src: SingleRegisters) -> Self {
		Self { src }
	}
}

impl Instruction for Or {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let dst_val = cpu.register_bank.read_single_named(ACC_REGISTER);
		let src_val = cpu.register_bank.read_single_named(self.src);

		let result = src_val | dst_val;
		cpu.register_bank.write_single_named(ACC_REGISTER, result);

		Ok(())
	}
}

pub(crate) struct Xor {
	src: SingleRegisters,
}

impl Xor {
	pub(crate) fn new(src: SingleRegisters) -> Self {
		Self { src }
	}
}

impl Instruction for Xor {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let dst_val = cpu.register_bank.read_single_named(ACC_REGISTER);
		let src_val = cpu.register_bank.read_single_named(self.src);

		let result = src_val ^ dst_val;
		cpu.register_bank.write_single_named(ACC_REGISTER, result);

		Ok(())
	}
}

pub(crate) struct Negate {}

impl Negate {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl Instruction for Negate {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let dst_val = cpu.register_bank.read_single_named(ACC_REGISTER);
		let resutl = !dst_val;

		cpu.register_bank.write_single_named(ACC_REGISTER, resutl);

		Ok(())
	}
}

#[test]
fn and_instruction() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0b0011_1100);
	cpu.register_bank.write_single_named(SingleRegisters::B, 0b0101_1010);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b0001_1000);

	assert!(And::new(SingleRegisters::B).execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}

#[test]
fn or_instruction() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0b0011_1100);
	cpu.register_bank.write_single_named(SingleRegisters::B, 0b0101_1010);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b0111_1110);

	assert!(Or::new(SingleRegisters::B).execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}

#[test]
fn xor_instruction() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0b0011_1100);
	cpu.register_bank.write_single_named(SingleRegisters::B, 0b0101_1010);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b0110_0110);

	assert!(Xor::new(SingleRegisters::B).execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}

#[test]
fn neg_instruction() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0b1100_1010);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b0011_0101);

	assert!(Negate::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}