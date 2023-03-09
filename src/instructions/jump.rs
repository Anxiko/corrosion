use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{BitFlags, DoubleRegisters};
use crate::instructions::changeset::{Change, ChangesetInstruction, NoChange, PcChange};
use crate::instructions::ExecutionError;

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum JumpInstructionDestination {
	Immediate(u16),
	AddressOnHl,
	Relative(i8),
}

impl JumpInstructionDestination {
	fn resolve(&self, cpu: &Cpu) -> u16 {
		match self {
			Self::Immediate(address) => *address,
			Self::AddressOnHl => cpu.register_bank.read_double_named(DoubleRegisters::HL),
			Self::Relative(delta) => cpu.pc.read().wrapping_add_signed((*delta).into())
		}
	}
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum JumpInstructionCondition {
	Unconditional,
	TestFlag { flag: BitFlags, branch_if_equals: bool },
}

impl JumpInstructionCondition {
	fn satisfied(&self, cpu: &Cpu) -> bool {
		match self {
			Self::Unconditional => true,
			Self::TestFlag { flag, branch_if_equals } => {
				cpu.register_bank.read_bit_flag(*flag) == *branch_if_equals
			}
		}
	}
}

pub(crate) struct JumpInstruction {
	dst: JumpInstructionDestination,
	condition: JumpInstructionCondition,
}

impl JumpInstruction {
	pub(crate) fn new(dst: JumpInstructionDestination, condition: JumpInstructionCondition) -> Self {
		Self { dst, condition }
	}
}

impl ChangesetInstruction for JumpInstruction {
	type C = Box<dyn Change>;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		if self.condition.satisfied(cpu) {
			let destination = self.dst.resolve(cpu);
			Ok(Box::new(PcChange::new(destination)))
		} else {
			Ok(Box::new(NoChange::new()))
		}
	}
}