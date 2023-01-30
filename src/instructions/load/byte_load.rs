use std::assert_matches::assert_matches;

use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::SingleRegisters;
use crate::instructions::{ACC_REGISTER, ExecutionError, Instruction};
use crate::instructions::base::{BaseByteInstruction, ByteDestination, ByteOperation, ByteSource};
use crate::instructions::changeset::{Change, SingleRegisterChange};

struct LoadByteOperation;

impl LoadByteOperation {
	fn new() -> Self { Self {} }
}

impl ByteOperation for LoadByteOperation {
	type C = Box<dyn Change>;

	fn execute(&self, cpu: &Cpu, src: &ByteSource, dst: &ByteDestination) -> Result<Self::C, ExecutionError> {
		let value = src.read(cpu)?;
		Ok(dst.change_destination(value))
	}
}

type LoadByteInstruction = BaseByteInstruction<LoadByteOperation>;

#[test]
fn load_operation() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(SingleRegisters::B, 0b1111_0000);
	let result = LoadByteOperation::new().execute(
		&cpu, &ByteSource::SingleRegister { single_reg: SingleRegisters::B }, &ByteDestination::Acc,
	).expect("Operation to execute");

	assert_eq!(
		result,
		SingleRegisterChange::new(ACC_REGISTER, 0b1111_0000)
	);
}

#[test]
fn load_instruction() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(SingleRegisters::B, 0b1111_0000);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b1111_0000);

	let operation = LoadByteOperation::new();

	let instruction = BaseByteInstruction::new(
		ByteSource::SingleRegister { single_reg: SingleRegisters::B },
		ByteDestination::Acc,
		operation,
	);

	assert!(instruction.execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}