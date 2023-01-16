use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::SingleRegisters;
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::arithmetic::ACC_REGISTER;

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