use dyn_partial_eq::DynPartialEq;

use crate::hardware::cpu::Cpu;
use crate::instructions::ExecutionError;

use super::Change;

#[derive(PartialEq, DynPartialEq, Debug)]
pub(crate) struct ChangeList {
	changes: Vec<Box<dyn Change>>,
}

impl ChangeList {
	pub(crate) fn new(changes: Vec<Box<dyn Change>>) -> Self {
		Self { changes }
	}
}

impl Change for ChangeList {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		for change in self.changes.iter() {
			change.commit_change(cpu)?;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::ram::{Ram, WORKING_RAM_START};
	use crate::instructions::changeset::{MemoryByteWriteChange, SingleRegisterChange};
	use crate::instructions::ACC_REGISTER;

	use super::*;

	#[test]
	fn empty_list() {
		let mut actual = Cpu::new();
		let expected = actual.clone();

		let change = ChangeList::new(Vec::new());
		change.commit_change(&mut actual).unwrap();

		assert_eq!(actual, expected);
	}

	#[test]
	fn multiple_changes() {
		let mut actual = Cpu::new();
		let mut expected = actual.clone();
		expected
			.register_bank
			.write_single_named(ACC_REGISTER, 0xFF);
		expected
			.mapped_ram
			.write_byte(WORKING_RAM_START, 0x12)
			.unwrap();

		let change = ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0xFF)),
			Box::new(MemoryByteWriteChange::write_to_immediate(
				WORKING_RAM_START,
				0x12,
			)),
		]);
		change.commit_change(&mut actual).unwrap();

		assert_eq!(actual, expected);
	}
}
