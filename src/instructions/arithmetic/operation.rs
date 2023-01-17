use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::RegisterFlags;
use crate::instructions::ACC_REGISTER;

#[derive(PartialEq, Debug)]
pub(super) struct ArithmeticOperation {
	result: u8,
	zero: bool,
	subtraction: bool,
	carry: bool,
	half_carry: bool,
}

impl ArithmeticOperation {
	pub(super) fn add(left: u8, right: u8) -> Self {
		Self::add_with_carry(left, right, false)
	}

	pub(super) fn add_with_carry(left: u8, right: u8, carry: bool) -> Self {
		let carry_val: u8 = carry.into();

		let (result, first_overflow) = left.overflowing_add(right);
		let (result, second_overflow) = result.overflowing_add(carry_val);

		Self {
			result,
			zero: result == 0,
			subtraction: false,
			carry: first_overflow || second_overflow,
			half_carry: Self::add_half_carry_flag(left, right, carry),
		}
	}

	pub(super) fn sub(left: u8, right: u8) -> Self {
		Self::sub_with_carry(left, right, false)
	}

	pub(super) fn sub_with_carry(left: u8, right: u8, carry: bool) -> Self {
		let carry_val: u8 = carry.into();

		let (result, first_underflow) = left.overflowing_sub(right);
		let (result, second_underflow) = result.overflowing_sub(carry_val);

		Self {
			result,
			zero: result == 0,
			subtraction: true,
			carry: first_underflow || second_underflow,
			half_carry: Self::sub_half_carry_flag(left, right, carry),
		}
	}

	pub(super) fn commit(&self, cpu: &mut Cpu) {
		cpu.register_bank.write_single_named(ACC_REGISTER, self.result);

		cpu.register_bank.write_bit_flag(RegisterFlags::Zero, self.zero);
		cpu.register_bank.write_bit_flag(RegisterFlags::Subtraction, self.subtraction);
		cpu.register_bank.write_bit_flag(RegisterFlags::Carry, self.carry);
		cpu.register_bank.write_bit_flag(RegisterFlags::HalfCarry, self.half_carry);
	}

	fn add_half_carry_flag(left: u8, right: u8, carry: bool) -> bool {
		let carry_val: u8 = carry.into();
		(left & LOWER_NIBBLE) + (right & LOWER_NIBBLE) + carry_val > LOWER_NIBBLE
	}

	fn sub_half_carry_flag(left: u8, right: u8, carry: bool) -> bool {
		let carry_val: u8 = carry.into();
		(left & LOWER_NIBBLE) < ((right & LOWER_NIBBLE) + carry_val)
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
fn arithmetic_add_with_carry() {
	let operation = ArithmeticOperation::add_with_carry(0x8, 0x7, true);

	assert_eq!(operation.result, 0x10);
	assert!(!operation.zero);
	assert!(!operation.subtraction);
	assert!(!operation.carry);
	assert!(operation.half_carry);

	let operation = ArithmeticOperation::add_with_carry(0x80, 0x7F, true);

	assert_eq!(operation.result, 0x00);
	assert!(operation.zero);
	assert!(!operation.subtraction);
	assert!(operation.carry);
	assert!(operation.half_carry);
}

#[test]
fn arithmetic_sub() {
	assert_eq!(
		ArithmeticOperation::sub(0x34, 0x12),
		ArithmeticOperation {
			result: 0x22,
			zero: false,
			subtraction: true,
			carry: false,
			half_carry: false,
		}
	);

	assert_eq!(
		ArithmeticOperation::sub(0x31, 0x14),
		ArithmeticOperation {
			result: 0x1D,
			zero: false,
			subtraction: true,
			carry: false,
			half_carry: true,
		}
	);

	assert_eq!(
		ArithmeticOperation::sub(0x12, 0x12),
		ArithmeticOperation {
			result: 0x00,
			zero: true,
			subtraction: true,
			carry: false,
			half_carry: false,
		}
	);

	assert_eq!(
		ArithmeticOperation::sub(0x10, 0x20),
		ArithmeticOperation {
			result: 0xF0,
			zero: false,
			subtraction: true,
			carry: true,
			half_carry: false,
		}
	);
}

#[test]
fn arithmetic_sub_with_carry() {
	assert_eq!(
		ArithmeticOperation::sub_with_carry(0x14, 0x04, true),
		ArithmeticOperation {
			result: 0x0F,
			zero: false,
			subtraction: true,
			carry: false,
			half_carry: true,
		}
	);

	assert_eq!(
		ArithmeticOperation::sub_with_carry(0x77, 0x86, true),
		ArithmeticOperation {
			result: 0xF0,
			zero: false,
			subtraction: true,
			carry: true,
			half_carry: false,
		}
	)
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

const LOWER_NIBBLE: u8 = 0xF;
