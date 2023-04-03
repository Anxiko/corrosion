use std::fmt::Debug;

use dyn_partial_eq::{dyn_partial_eq, DynPartialEq};

use crate::hardware::cpu::Cpu;
use crate::instructions::{ExecutionError, Instruction};

pub(crate) use self::flags::{BitFlagsChange, ChangeIme};
pub(crate) use self::memory::{MemoryByteWriteChange, MemoryDoubleByteWriteChange};
pub(crate) use self::registers::{DoubleRegisterChange, SingleRegisterChange};
pub(crate) use self::special_registers::{PcChange, SpChange};

#[dyn_partial_eq]
pub(crate) trait Change: Debug {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError>;
}

mod registers;
mod special_registers;
mod flags;
mod boxed;
mod memory;


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

pub(super) trait ChangesetInstruction {
	type C: Change;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError>;
}

impl<T> Instruction for T
	where
		T: ChangesetInstruction,
{
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let change = self.compute_change(cpu)?;
		change.commit_change(cpu)?;
		Ok(())
	}
}
