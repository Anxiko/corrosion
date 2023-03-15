use crate::hardware::alu::delta_u8;
use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::BitFlags;
use crate::instructions::{ACC_REGISTER, ExecutionError};
use crate::instructions::base::{BaseByteInstruction, ByteDestination, ByteOperation, ByteSource};
use crate::instructions::changeset::{BitFlagsChange, Change, ChangeList, ChangesetInstruction, SingleRegisterChange};

#[cfg(test)]
mod tests;

pub(crate) mod add;
mod operation;
pub(crate) mod sub;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum IncOrDecOperationType {
	Increment,
	Decrement,
}

impl IncOrDecOperationType {
	fn delta(&self) -> i8 {
		match self {
			Self::Increment => 1,
			Self::Decrement => -1
		}
	}

	fn is_sub(&self) -> bool {
		match self {
			Self::Increment => false,
			Self::Decrement => true
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct IncOrDecOperation {
	type_: IncOrDecOperationType,
}

impl IncOrDecOperation {
	pub(crate) fn new(type_: IncOrDecOperationType) -> Self {
		Self { type_ }
	}
}

impl ByteOperation for IncOrDecOperation {
	type C = Box<dyn Change>;

	fn execute(&self, cpu: &Cpu, src: &ByteSource, dst: &ByteDestination) -> Result<Self::C, ExecutionError> {
		let value = src.read(cpu)?;
		let delta = self.type_.delta();

		let alu_result = delta_u8(value, delta);
		let result = alu_result.result;
		let bitflags_change = BitFlagsChange::from(alu_result).keep_carry_flag();

		Ok(Box::new(ChangeList::new(vec![
			dst.change_destination(result),
			Box::new(bitflags_change),
		])))
	}
}

pub(crate) type IncOrDecInstruction = BaseByteInstruction<IncOrDecOperation>;

pub(crate) struct DecimalAdjust;

impl DecimalAdjust {
	pub(crate) fn new() -> Self {
		Self {}
	}

	// Implementation derived from here: https://forums.nesdev.org/viewtopic.php?t=15944
	fn adjust(mut value: u8, sub_flag: bool, carry_flag: bool, half_carry_flag: bool) -> (u8, bool) {
		if !sub_flag {
			let mut next_carry_flag = false;
			if carry_flag || value > 0x99 {
				value = value.wrapping_add(0x60);
				next_carry_flag = true;
			}
			if half_carry_flag || (value & 0x0F) > 0x09 {
				value = value.wrapping_add(0x06)
			}
			(value, next_carry_flag)
		} else {
			if carry_flag {
				value = value.wrapping_sub(0x60);
			}
			if half_carry_flag {
				value = value.wrapping_sub(0x06);
			}
			(value, false)
		}
	}
}

impl ChangesetInstruction for DecimalAdjust {
	type C = Box<dyn Change>;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let acc = cpu.register_bank.read_single_named(ACC_REGISTER);
		let sub_flag = cpu.register_bank.read_bit_flag(BitFlags::Subtraction);
		let carry_flag = cpu.register_bank.read_bit_flag(BitFlags::Carry);
		let half_carry_flag = cpu.register_bank.read_bit_flag(BitFlags::HalfCarry);

		let (next_acc, next_carry_flag) = DecimalAdjust::adjust(acc, sub_flag, carry_flag, half_carry_flag);

		Ok(Box::new(ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, next_acc)),
			Box::new(BitFlagsChange::keep_all()
				.with_zero_flag(next_acc == 0)
				.with_half_carry_flag(false)
				.with_carry_flag(next_carry_flag)
			),
		])))
	}
}