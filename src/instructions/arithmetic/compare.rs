use crate::hardware::alu::sub_u8;
use crate::hardware::cpu::Cpu;
use crate::instructions::base::ByteSource;
use crate::instructions::changeset::{BitFlagsChange, ChangesetInstruction};
use crate::instructions::ExecutionError;

pub struct CompareInstruction {
	left: ByteSource,
	right: ByteSource,
}

impl CompareInstruction {
	pub(crate) fn new(left: ByteSource, right: ByteSource) -> Self {
		Self { left, right }
	}
}

impl ChangesetInstruction for CompareInstruction {
	type C = BitFlagsChange;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let left_value = self.left.read(cpu)?;
		let right_value = self.right.read(cpu)?;
		let result = sub_u8(left_value, right_value);

		Ok(result.change_flags())
	}
}
