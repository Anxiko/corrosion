use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::SingleRegisters;
use crate::instructions::arithmetic::Add;
use crate::instructions::Instruction;

#[test]
fn add() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(SingleRegisters::A, 0x12);
	cpu.register_bank.write_single_named(SingleRegisters::B, 0x34);

	Add::new(SingleRegisters::B).execute(&mut cpu);

	assert_eq!(cpu.register_bank.read_single_named(SingleRegisters::A), 0x12 + 0x34);
}