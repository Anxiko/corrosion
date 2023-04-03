use std::any::Any;

use dyn_partial_eq::DynPartialEq;

use crate::hardware::cpu::Cpu;
use crate::instructions::ExecutionError;

use super::Change;

impl DynPartialEq for Box<dyn Change> {
	fn box_eq(&self, other: &dyn Any) -> bool {
		let other: Option<&Self> = other.downcast_ref();
		other.is_some_and(|other| {
			let boxed_self = &(**self);
			let boxed_other = &(**other);

			boxed_self.box_eq(boxed_other.as_any())
		})
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
}

impl Change for Box<dyn Change> {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let boxed_change = &(**self);
		boxed_change.commit_change(cpu)
	}
}