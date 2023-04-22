use std::fmt::Debug;

use dyn_partial_eq::{dyn_partial_eq, DynPartialEq};

use crate::hardware::cpu::Cpu;
use crate::instructions::{Executable, ExecutionError};

pub(crate) use self::flags::{BitFlagsChange, ChangeIme};
pub(crate) use self::list::ChangeList;
pub(crate) use self::memory::{MemoryByteWriteChange, MemoryDoubleByteWriteChange};
pub(crate) use self::noop::NoChange;
pub(crate) use self::registers::{DoubleRegisterChange, SingleRegisterChange};
pub(crate) use self::special_registers::{PcChange, SpChange};

#[dyn_partial_eq]
pub(crate) trait Change: Debug {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError>;
}

mod boxed;
mod flags;
mod list;
mod memory;
mod noop;
mod registers;
mod special_registers;

pub(super) trait ChangesetExecutable {
	type C: Change;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError>;
}

impl<T> Executable for T
	where
		T: ChangesetExecutable,
{
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let change = self.compute_change(cpu)?;
		change.commit_change(cpu)?;
		Ok(())
	}
}
