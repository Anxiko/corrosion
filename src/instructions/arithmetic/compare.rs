use crate::hardware::alu::sub_u8;
use crate::hardware::cpu::Cpu;
use crate::instructions::base::byte::ByteSource;
use crate::instructions::changeset::{BitFlagsChange, ChangesetExecutable};
use crate::instructions::ExecutionError;

#[derive(Debug)]
pub struct CompareInstruction {
	left: ByteSource,
	right: ByteSource,
}

impl CompareInstruction {
	pub(crate) fn new(left: ByteSource, right: ByteSource) -> Self {
		Self { left, right }
	}
}

impl ChangesetExecutable for CompareInstruction {
	type C = BitFlagsChange;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let left_value = self.left.read(cpu)?;
		let right_value = self.right.read(cpu)?;
		let result = sub_u8(left_value, right_value);

		Ok(result.change_flags())
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::register_bank::SingleRegisters;
	use crate::instructions::ACC_REGISTER;

	use super::*;

	#[test]
	fn bigger_than() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(ACC_REGISTER, 0x80);
		cpu.register_bank
			.write_single_named(SingleRegisters::B, 0x79);

		let expected = BitFlagsChange::keep_all()
			.with_carry_flag(false)
			.with_zero_flag(false)
			.with_half_carry_flag(true)
			.with_subtraction_flag(true);

		let instruction = CompareInstruction::new(
			ByteSource::read_from_acc(),
			ByteSource::SingleRegister(SingleRegisters::B),
		);

		let actual = instruction.compute_change(&cpu).expect("Compute changes");

		assert_eq!(actual, expected)
	}

	#[test]
	fn equal() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(ACC_REGISTER, 0x80);
		cpu.register_bank
			.write_single_named(SingleRegisters::B, 0x80);

		let expected = BitFlagsChange::keep_all()
			.with_carry_flag(false)
			.with_half_carry_flag(false)
			.with_subtraction_flag(true)
			.with_zero_flag(true);

		let instruction = CompareInstruction::new(
			ByteSource::read_from_acc(),
			ByteSource::SingleRegister(SingleRegisters::B),
		);

		let actual = instruction.compute_change(&cpu).expect("Compute changes");

		assert_eq!(actual, expected);
	}

	#[test]
	fn less_than() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(ACC_REGISTER, 0x80);
		cpu.register_bank
			.write_single_named(SingleRegisters::B, 0x81);

		let expected = BitFlagsChange::keep_all()
			.with_carry_flag(true)
			.with_half_carry_flag(true)
			.with_subtraction_flag(true)
			.with_zero_flag(false);

		let instruction = CompareInstruction::new(
			ByteSource::read_from_acc(),
			ByteSource::SingleRegister(SingleRegisters::B),
		);

		let actual = instruction.compute_change(&cpu).expect("Compute changes");

		assert_eq!(actual, expected);
	}
}
