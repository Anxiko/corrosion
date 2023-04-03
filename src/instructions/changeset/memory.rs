use dyn_partial_eq::DynPartialEq;

use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use crate::instructions::base::double_byte::DoubleByteSource;
use crate::instructions::ExecutionError;

use super::Change;

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
pub(crate) struct MemoryByteWriteChange {
	address: MemoryByteWriteAddress,
	value: u8,
}

impl MemoryByteWriteChange {
	pub(crate) fn write_to_immediate(address: u16, value: u8) -> Self {
		Self { address: MemoryByteWriteAddress::Immediate(address), value }
	}

	pub(crate) fn write_to_register(double_register: DoubleRegisters, value: u8) -> Self {
		Self { address: MemoryByteWriteAddress::Register(double_register), value }
	}

	pub(crate) fn write_to_offset(base: u16, offset: SingleRegisters, value: u8) -> Self {
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
pub(crate) struct MemoryDoubleByteWriteChange {
	address_source: DoubleByteSource,
	value: u16,
}

impl MemoryDoubleByteWriteChange {
	pub(crate) fn new(address_source: DoubleByteSource, value: u16) -> Self {
		Self { address_source, value }
	}

	pub(crate) fn write_to_immediate_address(address: u16, value: u16) -> Self {
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