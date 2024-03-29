use std::fmt::{Display, Formatter};

use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::DoubleRegisters;
use crate::instructions::base::byte::{ByteDestination, ByteSource, UnaryByteInstruction, UnaryByteOperation};
use crate::instructions::changeset::{ChangeList, DoubleRegisterChange};
use crate::instructions::shared::IndexUpdateType;
use crate::instructions::ExecutionError;

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) struct ByteLoadUpdate {
	index: DoubleRegisters,
	type_: IndexUpdateType,
}

impl ByteLoadUpdate {
	pub(crate) fn new(index: DoubleRegisters, type_: IndexUpdateType) -> Self {
		Self { index, type_ }
	}

	fn compute_change(&self, cpu: &Cpu) -> DoubleRegisterChange {
		let index_value = cpu.register_bank.read_double_named(self.index);
		let delta = self.type_.to_delta();
		let index_value = index_value.wrapping_add_signed(delta.into());
		DoubleRegisterChange::new(self.index, index_value)
	}

	fn as_str(&self) -> &str {
		match self.type_ {
			IndexUpdateType::Increment => "i",
			IndexUpdateType::Decrement => "d",
		}
	}
}

#[derive(Debug)]
pub(crate) struct ByteLoadOperation {
	update: Option<ByteLoadUpdate>,
}

impl ByteLoadOperation {
	pub(crate) fn new(update: Option<ByteLoadUpdate>) -> Self {
		Self { update }
	}

	pub(crate) fn no_update() -> Self {
		Self::new(None)
	}

	pub(crate) fn with_update(update: ByteLoadUpdate) -> Self {
		Self { update: Some(update) }
	}
}

impl UnaryByteOperation for ByteLoadOperation {
	type C = ChangeList;

	fn execute(&self, cpu: &Cpu, src: &ByteSource, dst: &ByteDestination) -> Result<Self::C, ExecutionError> {
		let value = src.read(cpu)?;
		let mut changes = vec![dst.change_destination(value)];

		if let Some(update) = self.update {
			changes.push(Box::new(update.compute_change(cpu)));
		}

		Ok(ChangeList::new(changes))
	}
}

impl Display for ByteLoadOperation {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "ld")?;
		if let Some(update) = self.update {
			write!(f, "{}", update.as_str())?;
		}

		Ok(())
	}
}

pub(crate) type ByteLoadInstruction = UnaryByteInstruction<ByteLoadOperation>;

impl ByteLoadInstruction {
	pub(crate) fn just_load(src: ByteSource, dst: ByteDestination) -> Self {
		Self::new(src, dst, ByteLoadOperation::no_update())
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::ram::{Ram, WORKING_RAM_START};
	use crate::hardware::register_bank::SingleRegisters;
	use crate::instructions::changeset::{ChangesetExecutable, MemoryByteWriteChange, SingleRegisterChange};
	use crate::instructions::ACC_REGISTER;

	use super::*;

	#[test]
	fn just_load() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(SingleRegisters::B, 0x80);

		let instruction = ByteLoadInstruction::just_load(
			ByteSource::SingleRegister(SingleRegisters::B),
			ByteDestination::write_to_acc(),
		);

		let expected = ChangeList::new(vec![Box::new(SingleRegisterChange::new(ACC_REGISTER, 0x80))]);
		let actual = instruction.compute_change(&cpu).expect("Compute changes");

		assert_eq!(actual, expected);
	}

	#[test]
	fn load_from_index_and_update() {
		let mut cpu = Cpu::new();
		cpu.register_bank
			.write_double_named(DoubleRegisters::HL, WORKING_RAM_START);
		cpu.mapped_ram
			.write_byte(WORKING_RAM_START, 0x80)
			.expect("Write to RAM");

		let instruction = ByteLoadInstruction::new(
			ByteSource::AddressInRegister(DoubleRegisters::HL),
			ByteDestination::write_to_acc(),
			ByteLoadOperation::with_update(ByteLoadUpdate::new(DoubleRegisters::HL, IndexUpdateType::Increment)),
		);

		let expected = ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0x80)),
			Box::new(DoubleRegisterChange::new(DoubleRegisters::HL, WORKING_RAM_START + 1)),
		]);
		let actual = instruction.compute_change(&cpu).expect("Compute changes");

		assert_eq!(actual, expected);
	}

	#[test]
	fn load_to_index_and_update() {
		let mut cpu = Cpu::new();
		cpu.register_bank
			.write_double_named(DoubleRegisters::HL, WORKING_RAM_START + 1);
		cpu.register_bank.write_single_named(SingleRegisters::B, 0x80);

		let instruction = ByteLoadInstruction::new(
			ByteSource::SingleRegister(SingleRegisters::B),
			ByteDestination::AddressInRegister(DoubleRegisters::HL),
			ByteLoadOperation::with_update(ByteLoadUpdate::new(DoubleRegisters::HL, IndexUpdateType::Decrement)),
		);

		let expected = ChangeList::new(vec![
			Box::new(MemoryByteWriteChange::write_to_register(DoubleRegisters::HL, 0x80)),
			Box::new(DoubleRegisterChange::new(DoubleRegisters::HL, WORKING_RAM_START)),
		]);
		let actual = instruction.compute_change(&cpu).expect("Compute changes");

		assert_eq!(actual, expected);
	}
}
