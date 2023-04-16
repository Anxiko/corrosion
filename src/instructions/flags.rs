use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::BitFlags;
use crate::instructions::ExecutionError;
use crate::instructions::changeset::{BitFlagsChange, ChangesetInstruction};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum BitFlagChangeType {
	Write(bool),
	Toggle,
}

impl BitFlagChangeType {
	fn new_value(&self, current_value: bool) -> bool {
		match self {
			Self::Toggle => !current_value,
			Self::Write(new_value) => *new_value
		}
	}
}

pub(crate) struct ChangeCarryFlagInstruction {
	change_type: BitFlagChangeType,
}

impl ChangeCarryFlagInstruction {
	pub(crate) fn new(change_type: BitFlagChangeType) -> Self {
		Self { change_type }
	}
}

impl ChangesetInstruction for ChangeCarryFlagInstruction {
	type C = BitFlagsChange;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let current_value = cpu.register_bank.read_bit_flag(BitFlags::Carry);
		let new_value = self.change_type.new_value(current_value);

		Ok(
			BitFlagsChange::keep_all()
				.with_carry_flag(new_value)
				.with_half_carry_flag(false)
				.with_subtraction_flag(false)
		)
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn toggle_carry() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_bit_flag(BitFlags::Carry, true);

		let instruction = ChangeCarryFlagInstruction::new(BitFlagChangeType::Toggle);

		let actual = instruction.compute_change(&cpu).unwrap();
		let expected = BitFlagsChange::keep_all()
			.with_carry_flag(false)
			.with_half_carry_flag(false)
			.with_subtraction_flag(false);

		assert_eq!(actual, expected);
	}

	#[test]
	fn set_carry() {
		let cpu = Cpu::new();

		let instruction = ChangeCarryFlagInstruction::new(
			BitFlagChangeType::Write(true)
		);

		let actual = instruction.compute_change(&cpu).unwrap();
		let expected = BitFlagsChange::keep_all()
			.with_carry_flag(true)
			.with_half_carry_flag(false)
			.with_subtraction_flag(false);

		assert_eq!(actual, expected);
	}
}