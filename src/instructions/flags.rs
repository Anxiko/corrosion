use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::RegisterFlags;
use crate::instructions::{ExecutionError, Instruction};

pub(crate) struct ToggleCarry {}

impl ToggleCarry {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl Instruction for ToggleCarry {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let carry = cpu.register_bank.read_bit_flag(RegisterFlags::Carry);

		cpu.register_bank.write_bit_flag(RegisterFlags::Carry, !carry);

		Ok(())
	}
}


pub(crate) struct SetCarry {}

impl SetCarry {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl Instruction for SetCarry {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		cpu.register_bank.write_bit_flag(RegisterFlags::Carry, true);
		Ok(())
	}
}

#[test]
fn toggle_carry() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_bit_flag(RegisterFlags::Carry, true);

	let mut expected = cpu.clone();
	expected.register_bank.write_bit_flag(RegisterFlags::Carry, false);

	assert!(ToggleCarry::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}

#[test]
fn set_carry() {
	let mut cpu = Cpu::new();

	let mut expected = cpu.clone();
	expected.register_bank.write_bit_flag(RegisterFlags::Carry, true);

	assert!(SetCarry::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}