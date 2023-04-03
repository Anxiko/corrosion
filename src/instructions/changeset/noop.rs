use dyn_partial_eq::DynPartialEq;
use crate::hardware::cpu::Cpu;
use crate::instructions::ExecutionError;
use super::Change;

#[derive(PartialEq, DynPartialEq, Debug)]
pub(crate) struct NoChange {}

impl NoChange {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl Change for NoChange {
	fn commit_change(&self, _cpu: &mut Cpu) -> Result<(), ExecutionError> {
		Ok(())
	}
}
