use dyn_partial_eq::DynPartialEq;

use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use crate::instructions::ExecutionError;

use super::Change;

#[derive(PartialEq, DynPartialEq, Debug)]
pub(in crate::instructions) struct SingleRegisterChange {
	reg: SingleRegisters,
	value: u8,
}

impl SingleRegisterChange {
	pub(in crate::instructions) fn new(reg: SingleRegisters, value: u8) -> Self {
		Self { reg, value }
	}
}

impl Change for SingleRegisterChange {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		cpu.register_bank.write_single_named(self.reg, self.value);
		Ok(())
	}
}

#[derive(PartialEq, DynPartialEq, Debug)]
pub(in crate::instructions) struct DoubleRegisterChange {
	reg: DoubleRegisters,
	value: u16,
}

impl DoubleRegisterChange {
	pub(in crate::instructions) fn new(reg: DoubleRegisters, value: u16) -> Self {
		Self { reg, value }
	}
}

impl Change for DoubleRegisterChange {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		cpu.register_bank.write_double_named(self.reg, self.value);
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::instructions::ACC_REGISTER;

	use super::*;

	#[test]
	fn single_register() {
		let mut actual = Cpu::new();

		let change = SingleRegisterChange::new(ACC_REGISTER, 0x34);

		let mut expected = actual.clone();
		expected.register_bank.write_single_named(ACC_REGISTER, 0x34);

		change.commit_change(&mut actual).unwrap();
		assert_eq!(actual, expected);
	}

	#[test]
	fn double_register() {
		let mut actual = Cpu::new();

		let change = DoubleRegisterChange::new(DoubleRegisters::BC, 0x1234);

		let mut expected = actual.clone();
		expected.register_bank.write_double_named(DoubleRegisters::BC, 0x1234);

		change.commit_change(&mut actual).unwrap();
		assert_eq!(actual, expected);
	}
}