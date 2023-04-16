use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::BitFlags;
use crate::instructions::base::double_byte::DoubleByteSource;

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum BranchCondition {
	Unconditional,
	TestFlag { flag: BitFlags, branch_if_equals: bool },
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum JumpInstructionDestination {
	FromSource(DoubleByteSource),
	RelativeToPc(i8),
}

impl BranchCondition {
	pub(super) fn satisfied(&self, cpu: &Cpu) -> bool {
		match self {
			Self::Unconditional => true,
			Self::TestFlag { flag, branch_if_equals } => {
				cpu.register_bank.read_bit_flag(*flag) == *branch_if_equals
			}
		}
	}
}