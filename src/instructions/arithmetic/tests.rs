use crate::hardware::cpu::Cpu;
use crate::hardware::ram::{Ram, WORKING_RAM_START};
use crate::hardware::register_bank::{DoubleRegisters, RegisterFlags, SingleRegisters};
use crate::instructions::arithmetic::{ACC_REGISTER, Add, AddHl, AddImmediate, AddWithCarry, operation};
use crate::instructions::arithmetic::operation::ArithmeticOperation;
use crate::instructions::Instruction;

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

#[test]
fn add_hl() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0x12);
	let mem_address = 0xC123;
	cpu.mapped_ram.write(mem_address, 0x34).expect("Write to working RAM");
	cpu.register_bank.write_double_named(DoubleRegisters::HL, mem_address);

	assert!(AddHl::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu.register_bank.read_single_named(ACC_REGISTER), 0x46);
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Zero));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Subtraction));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Carry));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::HalfCarry));
}

#[test]
fn add_immediate() {
	let mut cpu = Cpu::new();

	let src_address = WORKING_RAM_START;
	cpu.pc.write(WORKING_RAM_START);
	cpu.mapped_ram.write(src_address, 0x34).expect("Write to working RAM");
	cpu.register_bank.write_single_named(ACC_REGISTER, 0x12);

	assert!(AddImmediate::new().execute(&mut cpu).is_ok());
	assert_eq!(cpu.register_bank.read_single_named(ACC_REGISTER), 0x46);
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Zero));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Subtraction));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Carry));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::HalfCarry));
}

#[test]
fn add_with_carry() {
	let mut cpu = Cpu::new();

	cpu.register_bank.write_single_named(ACC_REGISTER, 0x80);
	cpu.register_bank.write_single_named(SingleRegisters::B, 0x7F);
	cpu.register_bank.write_bit_flag(RegisterFlags::Carry, true);

	assert!(AddWithCarry::new(SingleRegisters::B).execute(&mut cpu).is_ok());
	assert_eq!(cpu.register_bank.read_single_named(ACC_REGISTER), 0x00);
	assert!(cpu.register_bank.read_bit_flag(RegisterFlags::Zero));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Subtraction));
	assert!(cpu.register_bank.read_bit_flag(RegisterFlags::Carry));
	assert!(cpu.register_bank.read_bit_flag(RegisterFlags::HalfCarry));

}