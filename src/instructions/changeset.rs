use std::any::Any;
use std::fmt::Debug;

use dyn_partial_eq::{dyn_partial_eq, DynPartialEq};

use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::base::double_byte::DoubleByteSource;

pub(crate) use self::flags::{BitFlagsChange, ChangeIme};
pub(crate) use self::registers::{DoubleRegisterChange, SingleRegisterChange};
pub(crate) use self::special_registers::{PcChange, SpChange};

#[dyn_partial_eq]
pub(crate) trait Change: Debug {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError>;
}

mod registers;
mod special_registers;
mod flags;

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
	OffsetRegister { base: u16, offset: SingleRegisters },
}

impl MemoryByteWriteAddress {
	fn resolve(&self, cpu: &Cpu) -> u16 {
		match self {
			Self::Immediate(address) => *address,
			Self::Register(double_register) => {
				cpu.register_bank.read_double_named(*double_register)
			}
			Self::OffsetRegister { base, offset } => {
				let offset_value = cpu.register_bank.read_single_named(*offset);
				base.wrapping_add(offset_value.into())
			}
		}
	}
}

#[derive(Debug, PartialEq, DynPartialEq)]
pub(super) struct MemoryByteWriteChange {
	address: MemoryByteWriteAddress,
	value: u8,
}

impl MemoryByteWriteChange {
	pub(super) fn write_to_immediate(address: u16, value: u8) -> Self {
		Self { address: MemoryByteWriteAddress::Immediate(address), value }
	}

	pub(super) fn write_to_register(double_register: DoubleRegisters, value: u8) -> Self {
		Self { address: MemoryByteWriteAddress::Register(double_register), value }
	}

	pub(super) fn write_to_offset(base: u16, offset: SingleRegisters, value: u8) -> Self {
		Self { address: MemoryByteWriteAddress::OffsetRegister { base, offset }, value }
	}
}

impl Change for MemoryByteWriteChange {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		cpu.mapped_ram.write_byte(self.address.resolve(cpu), self.value)?;
		Ok(())
	}
}

#[derive(Debug, PartialEq, DynPartialEq)]
pub(super) struct MemoryDoubleByteWriteChange {
	address_source: DoubleByteSource,
	value: u16,
}

impl MemoryDoubleByteWriteChange {
	pub(super) fn new(address_source: DoubleByteSource, value: u16) -> Self {
		Self { address_source, value }
	}

	pub(super) fn write_to_immediate_address(address: u16, value: u16) -> Self {
		Self::new(DoubleByteSource::Immediate(address), value)
	}
}

impl Change for MemoryDoubleByteWriteChange {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let address = self.address_source.read(cpu)?;

		cpu.mapped_ram.write_double_byte(address, self.value)?;
		Ok(())
	}
}

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
