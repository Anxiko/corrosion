use dyn_partial_eq::DynPartialEq;

use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use crate::instructions::base::double_byte::DoubleByteSource;
use crate::instructions::ExecutionError;

use super::Change;

#[derive(Debug, PartialEq)]
pub(crate) enum MemoryWriteAddress {
	Immediate(u16),
	DoubleByteSource(DoubleByteSource),
	OffsetRegister { base: u16, offset: SingleRegisters },
}

impl MemoryWriteAddress {
	fn resolve(&self, cpu: &Cpu) -> Result<u16, ExecutionError> {
		match self {
			Self::Immediate(address) => Ok(*address),
			Self::DoubleByteSource(src) => src.read(cpu),
			Self::OffsetRegister { base, offset } => {
				let offset_value = cpu.register_bank.read_single_named(*offset);
				Ok(base.wrapping_add(offset_value.into()))
			}
		}
	}
}

#[derive(Debug, PartialEq, DynPartialEq)]
pub(crate) struct MemoryByteWriteChange {
	address: MemoryWriteAddress,
	value: u8,
}

impl MemoryByteWriteChange {
	pub(crate) fn write_to_immediate(address: u16, value: u8) -> Self {
		Self {
			address: MemoryWriteAddress::Immediate(address),
			value,
		}
	}

	pub(crate) fn write_to_register(double_register: DoubleRegisters, value: u8) -> Self {
		Self {
			address: MemoryWriteAddress::DoubleByteSource(DoubleByteSource::DoubleRegister(double_register)),
			value,
		}
	}

	pub(crate) fn write_to_offset(base: u16, offset: SingleRegisters, value: u8) -> Self {
		Self {
			address: MemoryWriteAddress::OffsetRegister { base, offset },
			value,
		}
	}
}

impl Change for MemoryByteWriteChange {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let address = self.address.resolve(cpu)?;
		cpu.mapped_ram.write_byte(address, self.value)?;
		Ok(())
	}
}

#[derive(Debug, PartialEq, DynPartialEq)]
pub(crate) struct MemoryDoubleByteWriteChange {
	address: MemoryWriteAddress,
	value: u16,
}

impl MemoryDoubleByteWriteChange {
	pub(crate) fn new(address: MemoryWriteAddress, value: u16) -> Self {
		Self { address, value }
	}

	pub(crate) fn write_to_immediate(address: u16, value: u16) -> Self {
		Self::new(MemoryWriteAddress::Immediate(address), value)
	}

	pub(crate) fn write_to_source(src: DoubleByteSource, value: u16) -> Self {
		Self::new(MemoryWriteAddress::DoubleByteSource(src), value)
	}
}

impl Change for MemoryDoubleByteWriteChange {
	fn commit_change(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let address = self.address.resolve(cpu)?;

		cpu.mapped_ram.write_double_byte(address, self.value)?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::ram::WORKING_RAM_START;
	use crate::instructions::ACC_REGISTER;

	use super::*;

	#[test]
	fn resolve_immediate() {
		let cpu = Cpu::new();

		assert_eq!(
			MemoryWriteAddress::Immediate(WORKING_RAM_START).resolve(&cpu).unwrap(),
			WORKING_RAM_START
		)
	}

	#[test]
	fn resolve_register() {
		let mut cpu = Cpu::new();
		cpu.register_bank
			.write_double_named(DoubleRegisters::HL, WORKING_RAM_START);

		assert_eq!(
			MemoryWriteAddress::DoubleByteSource(DoubleByteSource::DoubleRegister(DoubleRegisters::HL))
				.resolve(&cpu)
				.unwrap(),
			WORKING_RAM_START
		);
	}

	#[test]
	fn resolve_sp() {
		let mut cpu = Cpu::new();
		cpu.sp.write(WORKING_RAM_START);

		assert_eq!(
			MemoryWriteAddress::DoubleByteSource(DoubleByteSource::StackPointer)
				.resolve(&cpu)
				.unwrap(),
			WORKING_RAM_START
		);
	}

	#[test]
	fn resolve_offset() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(ACC_REGISTER, 0x01);

		assert_eq!(
			MemoryWriteAddress::OffsetRegister {
				offset: ACC_REGISTER,
				base: WORKING_RAM_START,
			}
			.resolve(&cpu)
			.unwrap(),
			WORKING_RAM_START + 1
		);
	}

	#[test]
	fn write_byte() {
		let mut actual = Cpu::new();
		let mut expected = actual.clone();
		expected.mapped_ram.write_byte(WORKING_RAM_START, 0x12).unwrap();

		let change = MemoryByteWriteChange::write_to_immediate(WORKING_RAM_START, 0x12);
		change.commit_change(&mut actual).unwrap();

		assert_eq!(actual, expected);
	}

	#[test]
	fn double_write_byte() {
		let mut actual = Cpu::new();
		actual.sp.write(WORKING_RAM_START);
		let mut expected = actual.clone();
		expected
			.mapped_ram
			.write_double_byte(WORKING_RAM_START, 0x1234)
			.unwrap();

		let change = MemoryDoubleByteWriteChange::write_to_source(DoubleByteSource::StackPointer, 0x1234);
		change.commit_change(&mut actual).unwrap();

		assert_eq!(actual, expected);
	}
}
