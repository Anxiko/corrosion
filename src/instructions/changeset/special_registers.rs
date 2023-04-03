use std::fmt::Debug;

use dyn_partial_eq::DynPartialEq;

use crate::hardware::cpu::Cpu;
use crate::instructions::ExecutionError;

use super::Change;

#[derive(PartialEq, DynPartialEq, Debug)]
pub(crate) struct SpChange {
	value: u16,
}

impl SpChange {
	pub(crate) fn new(value: u16) -> Self {
		Self { value }
	}
}

impl Change for SpChange {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		cpu.sp.write(self.value);
		Ok(())
	}
}

#[derive(PartialEq, DynPartialEq, Debug)]
pub(crate) struct PcChange {
	value: u16,
}

impl PcChange {
	pub(crate) fn new(value: u16) -> Self {
		Self { value }
	}
}

impl Change for PcChange {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		cpu.pc.write(self.value);
		Ok(())
	}
}
