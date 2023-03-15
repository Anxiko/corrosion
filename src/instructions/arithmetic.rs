use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::BitFlags;
use crate::instructions::{ACC_REGISTER, ExecutionError};
use crate::instructions::changeset::{BitFlagsChange, Change, ChangeList, ChangesetInstruction, SingleRegisterChange};

pub(crate) mod add_or_sub;
pub(crate) mod inc_or_dec;
pub(crate) mod compare;

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