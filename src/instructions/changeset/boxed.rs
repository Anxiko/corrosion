use std::any::Any;

use dyn_partial_eq::DynPartialEq;

use crate::hardware::cpu::Cpu;
use crate::instructions::ExecutionError;

use super::Change;

impl DynPartialEq for Box<dyn Change> {
	fn box_eq(&self, other: &dyn Any) -> bool {
		match other.downcast_ref::<Self>() {
			None => false,
			Some(other) => {
				let boxed_self = &(**self);
				let boxed_other = &(**other);

				boxed_self.box_eq(boxed_other.as_any())
			}
		}
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

#[cfg(test)]
mod tests {
	use crate::hardware::register_bank::DoubleRegisters;
	use crate::instructions::changeset::{DoubleRegisterChange, SingleRegisterChange};
	use crate::instructions::ACC_REGISTER;

	use super::*;

	#[test]
	fn equality() {
		let left: Box<dyn Change> = Box::new(SingleRegisterChange::new(ACC_REGISTER, 0x12));
		let right: Box<dyn Change> = Box::new(SingleRegisterChange::new(ACC_REGISTER, 0x12));

		assert_eq!(left, right)
	}

	#[test]
	fn inequality() {
		let left: Box<dyn Change> = Box::new(SingleRegisterChange::new(ACC_REGISTER, 0x12));
		let right: Box<dyn Change> = Box::new(DoubleRegisterChange::new(DoubleRegisters::HL, 0x12));

		assert_ne!(left, right)
	}

	#[test]
	fn change() {
		let mut actual = Cpu::new();
		let mut expected = actual.clone();
		expected.register_bank.write_single_named(ACC_REGISTER, 0x12);

		let change: Box<dyn Change> = Box::new(SingleRegisterChange::new(ACC_REGISTER, 0x12));
		change.commit_change(&mut actual).unwrap();

		assert_eq!(actual, expected);
	}
}
