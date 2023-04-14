use crate::hardware::alu::delta_u8;
use crate::hardware::cpu::Cpu;
use crate::instructions::base::byte::{ByteDestination, ByteSource, UnaryByteInstruction, UnaryByteOperation};
use crate::instructions::changeset::{BitFlagsChange, ChangeList};
use crate::instructions::ExecutionError;
use crate::instructions::shared::IndexUpdateType;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct IncOrDecOperation {
	type_: IndexUpdateType,
}

impl IncOrDecOperation {
	pub fn new(type_: IndexUpdateType) -> Self {
		Self { type_ }
	}
}

impl UnaryByteOperation for IncOrDecOperation {
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

pub(crate) type IncOrDecInstruction = UnaryByteInstruction<IncOrDecOperation>;

#[cfg(test)]
mod tests {
	use crate::instructions::ACC_REGISTER;
	use crate::instructions::changeset::{ChangesetInstruction, SingleRegisterChange};

	use super::*;

	#[test]
	fn increase() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(ACC_REGISTER, 0x80);

		let instruction = IncOrDecInstruction::new(
			ByteSource::read_from_acc(),
			ByteDestination::write_to_acc(),
			IncOrDecOperation::new(IndexUpdateType::Increment),
		);

		let expected = ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0x81)),
			Box::new(
				BitFlagsChange::keep_all()
					.with_subtraction_flag(false)
					.with_zero_flag(false)
					.with_half_carry_flag(false)
			),
		]);

		let actual = instruction.compute_change(&cpu).expect("Compute changes");
		assert_eq!(actual, expected);
	}

	#[test]
	fn decrease() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(ACC_REGISTER, 0x80);

		let instruction = IncOrDecInstruction::new(
			ByteSource::read_from_acc(),
			ByteDestination::write_to_acc(),
			IncOrDecOperation::new(IndexUpdateType::Decrement),
		);

		let expected = ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0x7F)),
			Box::new(
				BitFlagsChange::keep_all()
					.with_subtraction_flag(true)
					.with_zero_flag(false)
					.with_half_carry_flag(true)
			),
		]);

		let actual = instruction.compute_change(&cpu).expect("Compute changes");
		assert_eq!(actual, expected);
	}
}