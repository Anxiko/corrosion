use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::BitFlags;
use crate::instructions::{ExecutionError, Instruction};

pub(crate) struct ToggleCarry {}

impl ToggleCarry {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl Instruction for ToggleCarry {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let carry = cpu.register_bank.read_bit_flag(BitFlags::Carry);

		cpu.register_bank.write_bit_flag(BitFlags::Carry, !carry);

		Ok(())
	}
}

pub(crate) struct ChangeCarryFlag {
	value: bool,
}

impl ChangeCarryFlag {
	pub(crate) fn new(value: bool) -> Self {
		Self { value }
	}

	pub(crate) fn set() -> Self {
		Self::new(true)
	}

	pub(crate) fn clear() -> Self {
		Self::new(false)
	}
}

impl Instruction for ChangeCarryFlag {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		cpu.register_bank.write_bit_flag(BitFlags::Carry, self.value);
		Ok(())
	}
}

#[test]
fn toggle_carry() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_bit_flag(BitFlags::Carry, true);

	let mut expected = cpu.clone();
	expected
		.register_bank
		.write_bit_flag(BitFlags::Carry, false);

	assert!(ToggleCarry::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}

#[test]
fn set_carry() {
	let mut cpu = Cpu::new();

	let mut expected = cpu.clone();
	expected.register_bank.write_bit_flag(BitFlags::Carry, true);

	assert!(ChangeCarryFlag::set().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}
