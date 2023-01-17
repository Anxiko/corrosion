use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::RegisterFlags;
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::ACC_REGISTER;

struct RotateLeft {}

impl RotateLeft {
	fn new() -> Self {
		Self {}
	}
}

impl Instruction for RotateLeft {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let result = cpu.register_bank.read_single_named(ACC_REGISTER);
		let highest_bit = result & 0x80 != 0;

		let result = result.rotate_left(1);

		cpu.register_bank.write_single_named(ACC_REGISTER, result);
		cpu.register_bank.write_bit_flag(RegisterFlags::Carry, highest_bit);

		Ok(())
	}
}

#[test]
fn rotate_left() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0b1010_1010);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b0101_0101);
	expected.register_bank.write_bit_flag(RegisterFlags::Carry, true);

	assert!(RotateLeft::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);

	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0b0101_0101);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b1010_1010);
	expected.register_bank.write_bit_flag(RegisterFlags::Carry, false);

	assert!(RotateLeft::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}
