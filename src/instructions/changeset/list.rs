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
