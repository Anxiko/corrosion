use operation::{AsShiftOperation, ShiftDestination, ShiftDirection, ShiftOperation, ShiftType};

use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{RegisterFlags, SingleRegisters};
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::ACC_REGISTER;

mod operation;

enum ShiftSource {
	Acc,
	SingleRegister(SingleRegisters),
}

struct GenericShift {
	source: ShiftSource,
	destination: ShiftDestination,
	direction: ShiftDirection,
	type_: ShiftType,
}

impl GenericShift {
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
}

impl AsShiftOperation for GenericShift {
	fn as_shift_operation(&self, cpu: &mut Cpu) -> ShiftOperation {
		ShiftOperation::new(
			self.read_source(cpu),
			self.destination,
			self.direction,
			self.type_,
		)
	}
}

// TODO: Implement proper testing system for shift operations