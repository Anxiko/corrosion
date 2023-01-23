use std::assert_matches::assert_matches;
use operation::{AsShiftOperation, ShiftDestination, ShiftDirection, ShiftOperation, ShiftType};

use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{BitFlags, SingleRegisters};
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::ACC_REGISTER;
use crate::instructions::changeset::{Change, ChangeList, ChangesetInstruction};

mod operation;

enum ShiftSource {
	Acc,
	SingleRegister(SingleRegisters),
}

struct ShiftInstruction {
	source: ShiftSource,
	destination: ShiftDestination,
	direction: ShiftDirection,
	type_: ShiftType,
}

impl ShiftInstruction {
	fn new(source: ShiftSource, destination: ShiftDestination, direction: ShiftDirection, type_: ShiftType) -> Self {
		Self { source, destination, direction, type_ }
	}

	fn read_source(&self, cpu: &mut Cpu) -> u8 {
		let reg = match self.source {
			ShiftSource::Acc => ACC_REGISTER,
			ShiftSource::SingleRegister(reg) => reg
		};

		cpu.register_bank.read_single_named(reg)
	}

	fn as_shift_operation(&self, cpu: &mut Cpu) -> ShiftOperation {
		ShiftOperation::new(
			self.read_source(cpu),
			self.destination,
			self.direction,
			self.type_,
		)
	}
}

impl ChangesetInstruction for ShiftInstruction {
	type C = ChangeList;

	fn compute_change(&self, cpu: &mut Cpu) -> Result<Self::C, ExecutionError> {
		let operation = self.as_shift_operation(cpu);
		let operation_result = &operation.calculate();
		Ok(operation_result.into())
	}
}

#[test]
fn shift_instruction() {
	let mut cpu = Cpu::new();
	let old_carry = true;
	cpu.register_bank.write_single_named(ACC_REGISTER, 0b0011_0101);
	cpu.register_bank.write_bit_flag(BitFlags::Carry, old_carry);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b0110_1011);
	expected.register_bank.write_bit_flag(BitFlags::Carry, false);

	let instruction = ShiftInstruction::new(
		ShiftSource::Acc,
		ShiftDestination::Acc,
		ShiftDirection::Left,
		ShiftType::RotateWithCarry { old_carry },
	);

	assert_matches!(instruction.execute(&mut cpu), Ok(()));
	assert_eq!(cpu, expected);
}