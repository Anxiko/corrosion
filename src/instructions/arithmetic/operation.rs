use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::RegisterFlags;
use crate::instructions::arithmetic::{ACC_REGISTER, LOWER_NIBBLE};

pub(super) struct ArithmeticOperation {
	result: u8,
	zero: bool,
	subtraction: bool,
	carry: bool,
	half_carry: bool,
}

impl ArithmeticOperation {
	pub(super) fn add(left: u8, right: u8) -> Self {
		let (result, overflow) = left.overflowing_add(right);

		Self {
			result,
			zero: result == 0,
			subtraction: false,
			carry: overflow,
			half_carry: Self::half_carry(left, right),
		}
	}

	pub(super) fn commit(&self, cpu: &mut Cpu) {
		cpu.register_bank.write_single_named(ACC_REGISTER, self.result);

		cpu.register_bank.write_bit_flag(RegisterFlags::Zero, self.zero);
		cpu.register_bank.write_bit_flag(RegisterFlags::Subtraction, self.subtraction);
		cpu.register_bank.write_bit_flag(RegisterFlags::Carry, self.carry);
		cpu.register_bank.write_bit_flag(RegisterFlags::HalfCarry, self.half_carry);
	}

	fn half_carry(left: u8, right: u8) -> bool {
		(left & LOWER_NIBBLE) + (right & LOWER_NIBBLE) > LOWER_NIBBLE
	}
}

#[test]
fn arithmetic_add() {
	let operation = ArithmeticOperation::add(0x12, 0x34);

	assert_eq!(operation.result, 0x46);
	assert!(!operation.zero);
	assert!(!operation.subtraction);
	assert!(!operation.carry);
	assert!(!operation.half_carry);

	let operation = ArithmeticOperation::add(0x46, 0x0A);

	assert_eq!(operation.result, 0x50);
	assert!(!operation.zero);
	assert!(!operation.subtraction);
	assert!(!operation.carry);
	assert!(operation.half_carry);

	let operation = ArithmeticOperation::add(0x50, 0xB0);

	assert_eq!(operation.result, 0x00);
	assert!(operation.zero);
	assert!(!operation.subtraction);
	assert!(operation.carry);
	assert!(!operation.half_carry);
}

#[test]
fn arithmetic_commit() {
	test_commit_config(0x10, false, false, true, false);
	test_commit_config(0x00, true, false, false, true);
	test_commit_config(0x00, true, true, false, false);
}

fn test_commit_config(result: u8, zero: bool, subtraction: bool, half_carry: bool, carry: bool) {
	let mut cpu = Cpu::new();
	let operation = ArithmeticOperation {
		result,
		zero,
		subtraction,
		half_carry,
		carry,
	};

	operation.commit(&mut cpu);

	assert_eq!(cpu.register_bank.read_single_named(ACC_REGISTER), result);
	assert_eq!(cpu.register_bank.read_bit_flag(RegisterFlags::Zero), zero);
	assert_eq!(cpu.register_bank.read_bit_flag(RegisterFlags::Subtraction), subtraction);
	assert_eq!(cpu.register_bank.read_bit_flag(RegisterFlags::Carry), carry);
	assert_eq!(cpu.register_bank.read_bit_flag(RegisterFlags::HalfCarry), half_carry);
}