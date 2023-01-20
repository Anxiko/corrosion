use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{BitFlags, SingleRegisters};
use crate::instructions::{ExecutionError, Instruction};

pub(super) trait Change {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError>;
}

pub(super) struct SingleRegisterChange {
	reg: SingleRegisters,
	value: u8,
}

impl SingleRegisterChange {
	pub(super) fn new(reg: SingleRegisters, value: u8) -> Self {
		Self { reg, value }
	}
}

impl Change for SingleRegisterChange {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		cpu.register_bank.write_single_named(self.reg, self.value);
		Ok(())
	}
}

pub(super) struct BitFlagsChangeset {
	zero: Option<bool>,
	subtraction: Option<bool>,
	half_carry: Option<bool>,
	carry: Option<bool>,
}

impl BitFlagsChangeset {
	pub(super) fn new(
		zero: Option<bool>, subtraction: Option<bool>, half_carry: Option<bool>, carry: Option<bool>,
	) -> Self {
		Self {
			zero,
			subtraction,
			half_carry,
			carry,
		}
	}

	pub(super) fn keep_all() -> Self {
		Self {
			zero: None,
			subtraction: None,
			half_carry: None,
			carry: None,
		}
	}

	pub(super) fn zero_all() -> Self {
		Self {
			zero: None,
			subtraction: None,
			half_carry: None,
			carry: None,
		}
	}

	pub(super) fn with_zero_flag(mut self, value: bool) -> Self {
		self.zero = Some(value);
		self
	}

	pub(super) fn with_subtraction_flag(mut self, value: bool) -> Self {
		self.subtraction = Some(value);
		self
	}

	pub(super) fn with_half_carry_flag(mut self, value: bool) -> Self {
		self.half_carry = Some(value);
		self
	}

	pub(super) fn with_carry_flag(mut self, value: bool) -> Self {
		self.carry = Some(value);
		self
	}

	fn write_to(cpu: &mut Cpu, flag: BitFlags, maybe_value: Option<bool>) {
		if let Some(value) = maybe_value {
			cpu.register_bank.write_bit_flag(flag, value)
		}
	}
}

impl Change for BitFlagsChangeset {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		BitFlagsChangeset::write_to(cpu, BitFlags::Zero, self.zero);
		BitFlagsChangeset::write_to(cpu, BitFlags::Subtraction, self.subtraction);
		BitFlagsChangeset::write_to(cpu, BitFlags::HalfCarry, self.half_carry);
		BitFlagsChangeset::write_to(cpu, BitFlags::Carry, self.carry);

		Ok(())
	}
}

pub(super) struct ChangeList {
	changes: Vec<Box<dyn Change>>,
}

impl ChangeList {
	pub(super) fn new(changes: Vec<Box<dyn Change>>) -> Self {
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

pub(super) trait ChangesetInstruction<T: Change> {
	fn compute_change(&self, cpu: &mut Cpu) -> T;
}

impl<T> Instruction for T where
	T: ChangesetInstruction<ChangeList>,
{
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		self.compute_change(cpu).commit_change(cpu)
	}
}