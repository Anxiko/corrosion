use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{RegisterFlags, SingleRegisters};
use crate::instructions::arithmetic::{ACC_REGISTER, Add};
use crate::instructions::Instruction;

#[test]
fn add() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(ACC_REGISTER, 0x12);
	cpu.register_bank.write_single_named(SingleRegisters::B, 0x34);

	Add::new(SingleRegisters::B).execute(&mut cpu);

	assert_eq!(cpu.register_bank.read_single_named(ACC_REGISTER), 0x46);
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Zero));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Subtraction));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Carry));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::HalfCarry));

	cpu.register_bank.write_single_named(SingleRegisters::C, 0x0A);

	Add::new(SingleRegisters::C).execute(&mut cpu);

	assert_eq!(cpu.register_bank.read_single_named(ACC_REGISTER), 0x50);
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Zero));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Subtraction));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Carry));
	assert!(cpu.register_bank.read_bit_flag(RegisterFlags::HalfCarry));

	cpu.register_bank.write_single_named(SingleRegisters::D, 0xB0);

	Add::new(SingleRegisters::D).execute(&mut cpu);

	assert_eq!(cpu.register_bank.read_single_named(ACC_REGISTER), 0x00);
	assert!(cpu.register_bank.read_bit_flag(RegisterFlags::Zero));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::Subtraction));
	assert!(cpu.register_bank.read_bit_flag(RegisterFlags::Carry));
	assert!(!cpu.register_bank.read_bit_flag(RegisterFlags::HalfCarry));
}