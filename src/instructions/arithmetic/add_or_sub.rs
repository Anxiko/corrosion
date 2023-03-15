use crate::hardware::alu::{add_with_carry_u8, AluU8Result, sub_u8_with_carry};
use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::BitFlags;

use crate::instructions::base::{BinaryInstruction, BinaryOperation, ByteDestination, ByteSource};
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
			BinaryArithmeticOperationType::Add => {
				add_with_carry_u8(left, right, carry)
			}
			BinaryArithmeticOperationType::Sub => {
				sub_u8_with_carry(left, right, carry)
			}
		}
	}
}

impl BinaryOperation for BinaryArithmeticOperation {
	type C = ChangeList;

	fn compute_changes(&self, cpu: &Cpu, left: &ByteSource, right: &ByteSource, dst: &ByteDestination) -> Result<Self::C, ExecutionError> {
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

pub(crate) type BinaryArithmeticInstruction = BinaryInstruction<BinaryArithmeticOperation>;
