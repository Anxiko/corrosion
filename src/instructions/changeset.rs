use std::any::Any;
use std::fmt::{Debug, Formatter};

use dyn_partial_eq::{dyn_partial_eq, DynPartialEq};

use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::{BitFlags, DoubleRegisters, SingleRegisters};
use crate::instructions::{ExecutionError, Instruction};

#[dyn_partial_eq]
pub(super) trait Change: Debug {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError>;
}

#[derive(PartialEq, DynPartialEq, Debug)]
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

#[derive(PartialEq, DynPartialEq, Debug)]
pub(super) struct BitFlagsChange {
	zero: Option<bool>,
	subtraction: Option<bool>,
	half_carry: Option<bool>,
	carry: Option<bool>,
}

impl BitFlagsChange {
	pub(super) fn new(
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

impl Change for BitFlagsChange {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		BitFlagsChange::write_to(cpu, BitFlags::Zero, self.zero);
		BitFlagsChange::write_to(cpu, BitFlags::Subtraction, self.subtraction);
		BitFlagsChange::write_to(cpu, BitFlags::HalfCarry, self.half_carry);
		BitFlagsChange::write_to(cpu, BitFlags::Carry, self.carry);

		Ok(())
	}
}

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

#[derive(Debug, PartialEq)]
enum MemoryByteWriteAddress {
	Immediate(u16),
	Register(DoubleRegisters),
}

impl MemoryByteWriteAddress {
	fn resolve(&self, cpu: &Cpu) -> u16 {
		match self {
			Self::Immediate(address) => *address,
			Self::Register(double_register) => {
				cpu.register_bank.read_double_named(*double_register)
			}
		}
	}
}

#[derive(Debug, PartialEq, DynPartialEq)]
pub(super) struct MemoryByteWrite {
	address: MemoryByteWriteAddress,
	value: u8,
}

impl MemoryByteWrite {
	pub(super) fn write_to_immediate(address: u16, value: u8) -> Self {
		Self { address: MemoryByteWriteAddress::Immediate(address), value }
	}

	pub(super) fn write_to_register(double_register: DoubleRegisters, value: u8) -> Self {
		Self { address: MemoryByteWriteAddress::Register(double_register), value }
	}
}

impl Change for MemoryByteWrite {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		cpu.mapped_ram.write(self.address.resolve(cpu), self.value)?;
		Ok(())
	}
}

#[derive(PartialEq, DynPartialEq, Debug)]
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
