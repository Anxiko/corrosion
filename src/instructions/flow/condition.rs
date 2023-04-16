use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::BitFlags;

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum BranchCondition {
	Unconditional,
	TestFlag {
		flag: BitFlags,
		branch_if_equals: bool,
	},
}

impl BranchCondition {
	pub(super) fn satisfied(&self, cpu: &Cpu) -> bool {
		match self {
			Self::Unconditional => true,
			Self::TestFlag {
				flag,
				branch_if_equals,
			} => cpu.register_bank.read_bit_flag(*flag) == *branch_if_equals,
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::cpu::Cpu;
	use crate::hardware::register_bank::BitFlags;
	use crate::instructions::flow::BranchCondition;

	#[test]
	fn unconditional() {
		let cpu = Cpu::new();

		assert!(BranchCondition::Unconditional.satisfied(&cpu))
	}

	#[test]
	fn satisfied_flag_test() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_bit_flag(BitFlags::Carry, true);

		assert!(BranchCondition::TestFlag {
			flag: BitFlags::Carry,
			branch_if_equals: true
		}
		.satisfied(&cpu));
		assert!(BranchCondition::TestFlag {
			flag: BitFlags::Zero,
			branch_if_equals: false
		}
		.satisfied(&cpu));
	}

	#[test]
	fn unsatisfied_flag_test() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_bit_flag(BitFlags::Carry, true);

		assert!(!BranchCondition::TestFlag {
			flag: BitFlags::Carry,
			branch_if_equals: false
		}
		.satisfied(&cpu));
		assert!(!BranchCondition::TestFlag {
			flag: BitFlags::Zero,
			branch_if_equals: true
		}
		.satisfied(&cpu));
	}
}
