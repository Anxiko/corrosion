use std::fmt::{Display, Formatter};

use crate::hardware::alu::delta_u8;
use crate::hardware::cpu::Cpu;
use crate::instructions::base::byte::{ByteDestination, ByteSource, UnaryByteInstruction, UnaryByteOperation};
use crate::instructions::changeset::{BitFlagsChange, ChangeList};
use crate::instructions::shared::IndexUpdateType;
use crate::instructions::ExecutionError;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct IncOrDecByteOperation {
	type_: IndexUpdateType,
}

impl IncOrDecByteOperation {
	pub fn new(type_: IndexUpdateType) -> Self {
		Self { type_ }
	}

	fn as_str(&self) -> &str {
		match self.type_ {
			IndexUpdateType::Increment => "inc",
			IndexUpdateType::Decrement => "dec",
		}
	}
}

impl UnaryByteOperation for IncOrDecByteOperation {
	type C = ChangeList;

	fn execute(&self, cpu: &Cpu, src: &ByteSource, dst: &ByteDestination) -> Result<Self::C, ExecutionError> {
		let value = src.read(cpu)?;
		let delta = self.type_.to_delta();

		let alu_result = delta_u8(value, delta);
		let result = alu_result.result;
		let bitflags_change = BitFlagsChange::from(alu_result).keep_carry_flag();

		Ok(ChangeList::new(vec![
			dst.change_destination(result),
			Box::new(bitflags_change),
		]))
	}
}

impl Display for IncOrDecByteOperation {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

pub(crate) type IncOrDecByteInstruction = UnaryByteInstruction<IncOrDecByteOperation>;

#[cfg(test)]
mod tests {
	use crate::instructions::changeset::{ChangesetExecutable, SingleRegisterChange};
	use crate::instructions::ACC_REGISTER;

	use super::*;

	#[test]
	fn increase() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(ACC_REGISTER, 0x80);

		let instruction = IncOrDecByteInstruction::new(
			ByteSource::read_from_acc(),
			ByteDestination::write_to_acc(),
			IncOrDecByteOperation::new(IndexUpdateType::Increment),
		);

		let expected = ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0x81)),
			Box::new(
				BitFlagsChange::keep_all()
					.with_subtraction_flag(false)
					.with_zero_flag(false)
					.with_half_carry_flag(false),
			),
		]);

		let actual = instruction.compute_change(&cpu).expect("Compute changes");
		assert_eq!(actual, expected);
	}

	#[test]
	fn decrease() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(ACC_REGISTER, 0x80);

		let instruction = IncOrDecByteInstruction::new(
			ByteSource::read_from_acc(),
			ByteDestination::write_to_acc(),
			IncOrDecByteOperation::new(IndexUpdateType::Decrement),
		);

		let expected = ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0x7F)),
			Box::new(
				BitFlagsChange::keep_all()
					.with_subtraction_flag(true)
					.with_zero_flag(false)
					.with_half_carry_flag(true),
			),
		]);

		let actual = instruction.compute_change(&cpu).expect("Compute changes");
		assert_eq!(actual, expected);
	}
}
