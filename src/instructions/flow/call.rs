use std::fmt::{Display, Formatter};

use crate::bits::bits_to_byte;
use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::BitFlags;
use crate::instructions::base::double_byte::DoubleByteSource;
use crate::instructions::changeset::{
	Change, ChangeList, ChangesetExecutable, MemoryDoubleByteWriteChange, PcChange, SpChange,
};
use crate::instructions::flow::BranchCondition;
use crate::instructions::ExecutionError;

#[derive(Debug)]
pub(crate) struct CallInstruction {
	condition: BranchCondition,
	address: u16,
}

impl CallInstruction {
	pub(crate) fn new(condition: BranchCondition, address: u16) -> Self {
		Self { condition, address }
	}

	pub(crate) fn call(address: u16) -> Self {
		Self::new(BranchCondition::Unconditional, address)
	}

	pub(crate) fn call_conditional(flag: BitFlags, branch_if_equals: bool, address: u16) -> Self {
		Self::new(BranchCondition::TestFlag { flag, branch_if_equals }, address)
	}

	pub(crate) fn restart(bits: [bool; 3]) -> Self {
		let bits_as_byte: u16 = bits_to_byte(&bits).into();
		let address = 8u16 * bits_as_byte;
		Self::call(address)
	}
}

impl ChangesetExecutable for CallInstruction {
	type C = ChangeList;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let mut changes: Vec<Box<dyn Change>> = Vec::new();

		if self.condition.satisfied(cpu) {
			let mut sp = cpu.sp.read();
			sp = sp.wrapping_add_signed(-2);
			changes.push(Box::new(SpChange::new(sp)));

			let old_pc = cpu.pc.read();
			changes.push(Box::new(MemoryDoubleByteWriteChange::write_to_source(
				DoubleByteSource::StackPointer,
				old_pc,
			)));

			changes.push(Box::new(PcChange::new(self.address)))
		}

		Ok(ChangeList::new(changes))
	}
}

impl Display for CallInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "call")?;
		if let Some(condition) = self.condition.as_maybe_string() {
			write!(f, "{condition}, ")?;
		}
		write!(f, "{:#06X}", self.address)?;

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::cpu::Cpu;
	use crate::hardware::ram::WORKING_RAM_START;

	use super::*;

	fn get_cpu() -> Cpu {
		let mut cpu = Cpu::new();
		cpu.pc.write(0x1234);
		cpu.sp.write(WORKING_RAM_START + 10);
		cpu
	}

	#[test]
	fn call() {
		let cpu = get_cpu();

		let instruction = CallInstruction::call(0x4321);

		let actual = instruction.compute_change(&cpu).unwrap();
		let expected = ChangeList::new(vec![
			Box::new(SpChange::new(WORKING_RAM_START + 8)),
			Box::new(MemoryDoubleByteWriteChange::write_to_source(
				DoubleByteSource::StackPointer,
				0x1234,
			)),
			Box::new(PcChange::new(0x4321)),
		]);

		assert_eq!(actual, expected);
	}

	#[test]
	fn failed_call() {
		let cpu = get_cpu();

		let instruction = CallInstruction::call_conditional(BitFlags::Carry, true, 0x4321);

		let actual = instruction.compute_change(&cpu).unwrap();
		let expected = ChangeList::new(vec![]);

		assert_eq!(actual, expected);
	}
}
