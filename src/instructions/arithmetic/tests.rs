use crate::hardware::cpu::Cpu;
use crate::hardware::ram::{Ram, WORKING_RAM_START};
use crate::hardware::register_bank::{BitFlags, DoubleRegisters, SingleRegisters};
use crate::instructions::ACC_REGISTER;
use crate::instructions::arithmetic::add::{Add, AddHl, AddImmediate, AddWithCarry, Increment};
use crate::instructions::arithmetic::sub::{Compare, Decrement, Sub, SubWithCarry};
use crate::instructions::Instruction;

#[test]
fn add() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0x12);
	cpu.register_bank
		.write_single_named(SingleRegisters::B, 0x34);

	assert!(Add::new(SingleRegisters::B).execute(&mut cpu).is_ok());

	assert_eq!(cpu.register_bank.read_single_named(ACC_REGISTER), 0x46);
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::Zero));
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::Subtraction));
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::Carry));
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::HalfCarry));
}

#[test]
fn add_hl() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0x12);
	let mem_address = 0xC123;
	cpu.mapped_ram
		.write_byte(mem_address, 0x34)
		.expect("Write to working RAM");
	cpu.register_bank
		.write_double_named(DoubleRegisters::HL, mem_address);

	assert!(AddHl::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu.register_bank.read_single_named(ACC_REGISTER), 0x46);
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::Zero));
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::Subtraction));
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::Carry));
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::HalfCarry));
}

#[test]
fn add_immediate() {
	let mut cpu = Cpu::new();

	let src_address = WORKING_RAM_START;
	cpu.pc.write(WORKING_RAM_START);
	cpu.mapped_ram
		.write_byte(src_address, 0x34)
		.expect("Write to working RAM");
	cpu.register_bank.write_single_named(ACC_REGISTER, 0x12);

	assert!(AddImmediate::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu.register_bank.read_single_named(ACC_REGISTER), 0x46);
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::Zero));
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::Subtraction));
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::Carry));
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::HalfCarry));
}

#[test]
fn add_with_carry() {
	let mut cpu = Cpu::new();

	cpu.register_bank.write_single_named(ACC_REGISTER, 0x80);
	cpu.register_bank
		.write_single_named(SingleRegisters::B, 0x7F);
	cpu.register_bank.write_bit_flag(BitFlags::Carry, true);

	assert!(AddWithCarry::new(SingleRegisters::B)
		.execute(&mut cpu)
		.is_ok());
	assert_eq!(cpu.register_bank.read_single_named(ACC_REGISTER), 0x00);
	assert!(cpu.register_bank.read_bit_flag(BitFlags::Zero));
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::Subtraction));
	assert!(cpu.register_bank.read_bit_flag(BitFlags::Carry));
	assert!(cpu.register_bank.read_bit_flag(BitFlags::HalfCarry));
}

#[test]
fn sub() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0x34);
	cpu.register_bank
		.write_single_named(SingleRegisters::B, 0x12);

	assert!(Sub::new(SingleRegisters::B).execute(&mut cpu).is_ok());

	assert_eq!(cpu.register_bank.read_single_named(ACC_REGISTER), 0x22);
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::Zero));
	assert!(cpu.register_bank.read_bit_flag(BitFlags::Subtraction));
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::Carry));
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::HalfCarry));
}

#[test]
fn sub_with_carry() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0x34);
	cpu.register_bank
		.write_single_named(SingleRegisters::B, 0x24);
	cpu.register_bank.write_bit_flag(BitFlags::Carry, true);

	assert!(SubWithCarry::new(SingleRegisters::B)
		.execute(&mut cpu)
		.is_ok());

	assert_eq!(cpu.register_bank.read_single_named(ACC_REGISTER), 0x0F);
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::Zero));
	assert!(cpu.register_bank.read_bit_flag(BitFlags::Subtraction));
	assert!(cpu.register_bank.read_bit_flag(BitFlags::HalfCarry));
	assert!(!cpu.register_bank.read_bit_flag(BitFlags::Carry));
}

#[test]
fn compare() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0x12);
	cpu.register_bank
		.write_single_named(SingleRegisters::B, 0x34);

	let mut expected = cpu.clone();
	expected.register_bank.write_bit_flag(BitFlags::Zero, false);
	expected.register_bank.write_bit_flag(BitFlags::Carry, true);
	expected
		.register_bank
		.write_bit_flag(BitFlags::HalfCarry, true);
	expected
		.register_bank
		.write_bit_flag(BitFlags::Subtraction, true);

	assert!(Compare::new(SingleRegisters::B).execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}

#[test]
fn increment() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0x7F);

	let mut expected = cpu.clone();
	expected
		.register_bank
		.write_single_named(ACC_REGISTER, 0x80);
	expected.register_bank.write_bit_flag(BitFlags::Zero, false);
	expected
		.register_bank
		.write_bit_flag(BitFlags::Carry, false);
	expected
		.register_bank
		.write_bit_flag(BitFlags::HalfCarry, true);
	expected
		.register_bank
		.write_bit_flag(BitFlags::Subtraction, false);

	assert!(Increment::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}

#[test]
fn decrement() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0x80);

	let mut expected = cpu.clone();
	expected
		.register_bank
		.write_single_named(ACC_REGISTER, 0x7F);
	expected.register_bank.write_bit_flag(BitFlags::Zero, false);
	expected
		.register_bank
		.write_bit_flag(BitFlags::Carry, false);
	expected
		.register_bank
		.write_bit_flag(BitFlags::HalfCarry, true);
	expected
		.register_bank
		.write_bit_flag(BitFlags::Subtraction, true);

	assert!(Decrement::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}
