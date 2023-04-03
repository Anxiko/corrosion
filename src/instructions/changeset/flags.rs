use dyn_partial_eq::DynPartialEq;

use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::BitFlags;
use crate::instructions::ExecutionError;

use super::Change;

#[derive(PartialEq, DynPartialEq, Debug)]
pub(crate) struct BitFlagsChange {
	zero: Option<bool>,
	subtraction: Option<bool>,
	half_carry: Option<bool>,
	carry: Option<bool>,
}

impl BitFlagsChange {
	pub(crate) fn new(
		zero: Option<bool>,
		subtraction: Option<bool>,
		half_carry: Option<bool>,
		carry: Option<bool>,
	) -> Self {
		Self {
			zero,
			subtraction,
			half_carry,
			carry,
		}
	}

	pub(crate) fn keep_all() -> Self {
		Self {
			zero: None,
			subtraction: None,
			half_carry: None,
			carry: None,
		}
	}

	pub(crate) fn zero_all() -> Self {
		Self {
			zero: Some(false),
			subtraction: Some(false),
			half_carry: Some(false),
			carry: Some(false),
		}
	}

	pub(crate) fn with_zero_flag(mut self, value: bool) -> Self {
		self.zero = Some(value);
		self
	}

	pub(crate) fn with_subtraction_flag(mut self, value: bool) -> Self {
		self.subtraction = Some(value);
		self
	}

	pub(crate) fn with_half_carry_flag(mut self, value: bool) -> Self {
		self.half_carry = Some(value);
		self
	}

	pub(crate) fn with_carry_flag(mut self, value: bool) -> Self {
		self.carry = Some(value);
		self
	}

	pub(crate) fn keep_zero_flag(mut self) -> Self {
		self.zero = None;
		self
	}

	pub(crate) fn keep_subtraction_flag(mut self) -> Self {
		self.subtraction = None;
		self
	}

	pub(crate) fn keep_half_carry(mut self) -> Self {
		self.half_carry = None;
		self
	}

	pub(crate) fn keep_carry_flag(mut self) -> Self {
		self.carry = None;
		self
	}

	fn write_to(cpu: &mut Cpu, flag: BitFlags, maybe_value: Option<bool>) {
		if let Some(value) = maybe_value {
			cpu.register_bank.write_bit_flag(flag, value)
		}
	}
}

impl Change for BitFlagsChange {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		BitFlagsChange::write_to(cpu, BitFlags::Zero, self.zero);
		BitFlagsChange::write_to(cpu, BitFlags::Subtraction, self.subtraction);
		BitFlagsChange::write_to(cpu, BitFlags::HalfCarry, self.half_carry);
		BitFlagsChange::write_to(cpu, BitFlags::Carry, self.carry);

		Ok(())
	}
}

#[derive(PartialEq, DynPartialEq, Debug)]
pub(crate) struct ChangeIme {
	value: bool,
}

impl ChangeIme {
	pub(crate) fn new(value: bool) -> Self {
		Self { value }
	}
}

impl Change for ChangeIme {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		cpu.ime.write(self.value);
		Ok(())
	}
}