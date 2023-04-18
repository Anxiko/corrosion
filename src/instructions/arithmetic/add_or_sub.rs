use std::fmt::{Display, Formatter};

use crate::hardware::alu::{add_with_carry_u8, AluU8Result, sub_u8_with_carry};
use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::BitFlags;
use crate::instructions::base::byte::{BinaryByteOperation, ByteDestination, ByteSource};
use crate::instructions::base::byte::BinaryByteInstruction;
use crate::instructions::changeset::ChangeList;
use crate::instructions::ExecutionError;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum BinaryArithmeticOperationType {
	Add,
	Sub,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct BinaryArithmeticOperation {
	type_: BinaryArithmeticOperationType,
	with_carry: bool,
}

impl BinaryArithmeticOperation {
	pub(crate) fn new(type_: BinaryArithmeticOperationType, with_carry: bool) -> Self {
		Self { type_, with_carry }
	}

	fn alu_result(&self, left: u8, right: u8, carry: bool) -> AluU8Result {
		match self.type_ {
			BinaryArithmeticOperationType::Add => add_with_carry_u8(left, right, carry),
			BinaryArithmeticOperationType::Sub => sub_u8_with_carry(left, right, carry),
		}
	}

	fn as_str(&self) -> &str {
		match self {
			Self { type_: BinaryArithmeticOperationType::Add, with_carry: false } => "add",
			Self { type_: BinaryArithmeticOperationType::Add, with_carry: true } => "adc",
			Self { type_: BinaryArithmeticOperationType::Sub, with_carry: false } => "sub",
			Self { type_: BinaryArithmeticOperationType::Sub, with_carry: true } => "sbc"
		}
	}
}

impl BinaryByteOperation for BinaryArithmeticOperation {
	type C = ChangeList;

	fn compute_changes(
		&self,
		cpu: &Cpu,
		left: &ByteSource,
		right: &ByteSource,
		dst: &ByteDestination,
	) -> Result<Self::C, ExecutionError> {
		let carry = self.with_carry && cpu.register_bank.read_bit_flag(BitFlags::Carry);
		let left = left.read(cpu)?;
		let right = right.read(cpu)?;

		let result = self.alu_result(left, right, carry);

		Ok(ChangeList::new(vec![
			result.change_dst(dst),
			Box::new(result.change_flags()),
		]))
	}
}

impl Display for BinaryArithmeticOperation {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

pub(crate) type BinaryArithmeticInstruction = BinaryByteInstruction<BinaryArithmeticOperation>;

#[cfg(test)]
mod tests {
	use crate::hardware::cpu::Cpu;
	use crate::hardware::register_bank::{BitFlags, SingleRegisters};
	use crate::instructions::arithmetic::add_or_sub::{
		BinaryArithmeticInstruction, BinaryArithmeticOperation, BinaryArithmeticOperationType,
	};
	use crate::instructions::base::byte::{ByteDestination, ByteSource};
	use crate::instructions::changeset::{
		BitFlagsChange, ChangeList, ChangesetExecutable, SingleRegisterChange,
	};

	#[test]
	fn add() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_bit_flag(BitFlags::Carry, true);
		cpu.register_bank
			.write_single_named(SingleRegisters::A, 0x12);
		cpu.register_bank
			.write_single_named(SingleRegisters::B, 0x34);
		let cpu = cpu;

		let instruction = BinaryArithmeticInstruction::new(
			ByteSource::SingleRegister(SingleRegisters::A),
			ByteSource::SingleRegister(SingleRegisters::B),
			ByteDestination::SingleRegister(SingleRegisters::A),
			BinaryArithmeticOperation::new(BinaryArithmeticOperationType::Add, false),
		);

		let expected = ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(SingleRegisters::A, 0x46)),
			Box::new(
				BitFlagsChange::keep_all()
					.with_zero_flag(false)
					.with_half_carry_flag(false)
					.with_carry_flag(false)
					.with_subtraction_flag(false),
			),
		]);

		let actual = instruction.compute_change(&cpu).expect("Compute changes");

		assert_eq!(actual, expected);
	}

	#[test]
	fn add_with_carry() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_bit_flag(BitFlags::Carry, true);
		cpu.register_bank
			.write_single_named(SingleRegisters::A, 0x18);
		cpu.register_bank
			.write_single_named(SingleRegisters::B, 0x37);
		let cpu = cpu;

		let instruction = BinaryArithmeticInstruction::new(
			ByteSource::SingleRegister(SingleRegisters::A),
			ByteSource::SingleRegister(SingleRegisters::B),
			ByteDestination::SingleRegister(SingleRegisters::A),
			BinaryArithmeticOperation::new(BinaryArithmeticOperationType::Add, true),
		);

		let expected = ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(SingleRegisters::A, 0x50)),
			Box::new(
				BitFlagsChange::keep_all()
					.with_zero_flag(false)
					.with_half_carry_flag(true)
					.with_carry_flag(false)
					.with_subtraction_flag(false),
			),
		]);

		let actual = instruction.compute_change(&cpu).expect("Compute changes");

		assert_eq!(actual, expected);
	}

	#[test]
	fn sub() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_bit_flag(BitFlags::Carry, false);
		cpu.register_bank
			.write_single_named(SingleRegisters::A, 0x18);
		cpu.register_bank
			.write_single_named(SingleRegisters::B, 0x37);
		let cpu = cpu;

		let instruction = BinaryArithmeticInstruction::new(
			ByteSource::SingleRegister(SingleRegisters::A),
			ByteSource::SingleRegister(SingleRegisters::B),
			ByteDestination::SingleRegister(SingleRegisters::A),
			BinaryArithmeticOperation::new(BinaryArithmeticOperationType::Sub, true),
		);

		let expected = ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(SingleRegisters::A, 0xE1)),
			Box::new(
				BitFlagsChange::keep_all()
					.with_zero_flag(false)
					.with_half_carry_flag(false)
					.with_carry_flag(true)
					.with_subtraction_flag(true),
			),
		]);

		let actual = instruction.compute_change(&cpu).expect("Compute changes");

		assert_eq!(actual, expected);
	}
}
