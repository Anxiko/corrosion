use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{RegisterFlags, SingleRegisters};
use crate::instructions::arithmetic::{ACC_REGISTER, Add, ArithmeticOperation};
use crate::instructions::Instruction;

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

#[test]
fn add() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0x12);
	cpu.register_bank.write_single_named(SingleRegisters::B, 0x34);

	assert!(Add::new(SingleRegisters::B).execute(&mut cpu).is_ok());

	assert_eq!(cpu.register_bank.read_single_named(ACC_REGISTER), 0x46);
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Zero));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Subtraction));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Carry));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::HalfCarry));
}