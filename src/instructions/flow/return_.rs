use std::fmt::{Display, Formatter};

use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Rom;
use crate::hardware::register_bank::BitFlags;
use crate::instructions::changeset::{Change, ChangeIme, ChangeList, ChangesetExecutable, PcChange, SpChange};
use crate::instructions::flow::BranchCondition;
use crate::instructions::ExecutionError;

#[derive(Debug)]
pub(crate) struct ReturnInstruction {
	condition: BranchCondition,
	enable_interrupts: bool,
}

impl ReturnInstruction {
	pub(crate) fn new(condition: BranchCondition, enable_interrupts: bool) -> Self {
		Self {
			condition,
			enable_interrupts,
		}
	}

	pub(crate) fn ret_conditional(flag: BitFlags, value: bool) -> Self {
		Self::new(
			BranchCondition::TestFlag {
				flag,
				branch_if_equals: value,
			},
			false,
		)
	}
}

impl Display for ReturnInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "ret")?;
		if self.enable_interrupts {
			write!(f, "i")?;
		}
		if let Some(condition) = self.condition.as_maybe_string() {
			write!(f, " {condition}")?;
		}

		Ok(())
	}
}

impl ChangesetExecutable for ReturnInstruction {
	type C = ChangeList;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let mut changes: Vec<Box<dyn Change>> = Vec::new();
		if self.condition.satisfied(cpu) {
			let sp_value = cpu.sp.read();
			let address = cpu.mapped_ram.read_double_byte(sp_value)?;

			changes.push(Box::new(PcChange::new(address)));
			changes.push(Box::new(SpChange::new(sp_value + 2)));

			if self.enable_interrupts {
				changes.push(Box::new(ChangeIme::new(true)));
			}
		}

		Ok(ChangeList::new(changes))
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::cpu::Cpu;
	use crate::hardware::ram::{Ram, WORKING_RAM_START};
	use crate::hardware::register_bank::BitFlags;
	use crate::instructions::changeset::{ChangeIme, ChangeList, ChangesetExecutable, PcChange, SpChange};
	use crate::instructions::flow::{BranchCondition, ReturnInstruction};

	fn get_cpu() -> Cpu {
		let mut cpu = Cpu::new();
		cpu.pc.write(0x1234);
		cpu.sp.write(WORKING_RAM_START + 10);
		cpu.mapped_ram
			.write_double_byte(WORKING_RAM_START + 10, 0x4321)
			.unwrap();
		cpu
	}

	#[test]
	fn unconditional_return() {
		let cpu = get_cpu();

		let instruction = ReturnInstruction::new(BranchCondition::Unconditional, false);

		let actual = instruction.compute_change(&cpu).unwrap();
		let expected = ChangeList::new(vec![
			Box::new(PcChange::new(0x4321)),
			Box::new(SpChange::new(WORKING_RAM_START + 12)),
		]);

		assert_eq!(actual, expected);
	}

	#[test]
	fn conditional_return_enable_interrupts() {
		let cpu = get_cpu();

		let instruction = ReturnInstruction::new(
			BranchCondition::TestFlag {
				flag: BitFlags::Carry,
				branch_if_equals: false,
			},
			true,
		);

		let actual = instruction.compute_change(&cpu).unwrap();
		let expected = ChangeList::new(vec![
			Box::new(PcChange::new(0x4321)),
			Box::new(SpChange::new(WORKING_RAM_START + 12)),
			Box::new(ChangeIme::new(true)),
		]);

		assert_eq!(actual, expected);
	}

	#[test]
	fn failed_conditional_return() {
		let cpu = get_cpu();

		let instruction = ReturnInstruction::ret_conditional(BitFlags::Carry, true);

		let actual = instruction.compute_change(&cpu).unwrap();
		let expected = ChangeList::new(vec![]);

		assert_eq!(actual, expected);
	}
}
