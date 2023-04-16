use crate::hardware::cpu::Cpu;
use crate::instructions::base::double_byte::DoubleByteSource;
use crate::instructions::changeset::{Change, ChangesetInstruction, NoChange, PcChange};
use crate::instructions::ExecutionError;
use crate::instructions::flow::BranchCondition;

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum JumpInstructionDestination {
	FromSource(DoubleByteSource),
	RelativeToPc(i8),
}

impl JumpInstructionDestination {
	fn resolve(&self, cpu: &Cpu) -> Result<u16, ExecutionError> {
		match self {
			Self::FromSource(source) => {
				source.read(cpu)
			}
			Self::RelativeToPc(delta) => Ok(cpu.pc.read().wrapping_add_signed((*delta).into()))
		}
	}
}


pub(crate) struct JumpInstruction {
	dst: JumpInstructionDestination,
	condition: BranchCondition,
}

impl JumpInstruction {
	pub(crate) fn new(dst: JumpInstructionDestination, condition: BranchCondition) -> Self {
		Self { dst, condition }
	}
}

impl ChangesetInstruction for JumpInstruction {
	type C = Box<dyn Change>;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		if self.condition.satisfied(cpu) {
			let destination = self.dst.resolve(cpu)?;
			Ok(Box::new(PcChange::new(destination)))
		} else {
			Ok(Box::new(NoChange::new()))
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::ram::{Ram, WORKING_RAM_START};
	use crate::hardware::register_bank::{BitFlags, DoubleRegisters};

	use super::*;

	fn get_cpu() -> Cpu {
		let mut cpu = Cpu::new();
		cpu.pc.write(0x1234);
		cpu.register_bank.write_double_named(DoubleRegisters::HL, WORKING_RAM_START);
		cpu.mapped_ram.write_double_byte(WORKING_RAM_START, 0x5678).expect("Write to RAM");
		cpu.register_bank.write_bit_flag(BitFlags::Zero, true);
		cpu
	}

	#[test]
	fn unconditional_jump_to_immediate() {
		let cpu = get_cpu();

		let instruction = JumpInstruction::new(
			JumpInstructionDestination::FromSource(DoubleByteSource::Immediate(0xABCD)),
			BranchCondition::Unconditional,
		);

		let expected: Box<dyn Change> = Box::new(PcChange::new(0xABCD));
		let actual = instruction.compute_change(&cpu).expect("Compute change");

		assert_eq!(actual, expected);
	}

	#[test]
	fn flag_test_relative_jump() {
		let cpu = get_cpu();

		let instruction = JumpInstruction::new(
			JumpInstructionDestination::RelativeToPc(-0x7F),
			BranchCondition::TestFlag { flag: BitFlags::Carry, branch_if_equals: false },
		);

		let expected: Box<dyn Change> = Box::new(PcChange::new(0x11B5));
		let actual = instruction.compute_change(&cpu).expect("Compute change");

		assert_eq!(actual, expected);
	}

	#[test]
	fn flag_test_relative_jump_failed() {
		let cpu = get_cpu();

		let instruction = JumpInstruction::new(
			JumpInstructionDestination::RelativeToPc(-0x7F),
			BranchCondition::TestFlag { flag: BitFlags::Carry, branch_if_equals: true },
		);

		let expected: Box<dyn Change> = Box::new(NoChange::new());
		let actual = instruction.compute_change(&cpu).expect("Compute change");

		assert_eq!(actual, expected);
	}
}