use std::any::Any;
use std::fmt::Debug;

use dyn_partial_eq::{dyn_partial_eq, DynPartialEq};

use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::{BitFlags, DoubleRegisters, SingleRegisters};
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::base::DoubleByteSource;

#[dyn_partial_eq]
pub(crate) trait Change: Debug {
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
pub(super) struct DoubleRegisterChange {
	reg: DoubleRegisters,
	value: u16,
}

impl DoubleRegisterChange {
	pub(super) fn new(reg: DoubleRegisters, value: u16) -> Self {
		Self { reg, value }
	}
}

impl Change for DoubleRegisterChange {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		cpu.register_bank.write_double_named(self.reg, self.value);
		Ok(())
	}
}

#[derive(PartialEq, DynPartialEq, Debug)]
pub(super) struct SpChange {
	value: u16,
}

impl SpChange {
	pub(super) fn new(value: u16) -> Self {
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
pub(super) struct PcChange {
	value: u16,
}

impl PcChange {
	pub(super) fn new(value: u16) -> Self {
		Self { value }
	}
}

impl Change for PcChange {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		cpu.pc.write(self.value);
		Ok(())
	}
}

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
			},
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
