use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::BitFlags;
use crate::instructions::{ACC_REGISTER, ExecutionError};
use crate::instructions::changeset::{BitFlagsChange, Change, ChangeList, ChangesetInstruction, SingleRegisterChange};

pub(crate) struct DecimalAdjust;

impl DecimalAdjust {
	pub(crate) fn new() -> Self {
		Self {}
	}

	// Implementation derived from here: https://forums.nesdev.org/viewtopic.php?t=15944
	fn adjust(mut value: u8, sub_flag: bool, carry_flag: bool, half_carry_flag: bool) -> (u8, bool) {
		let mut next_carry_flag = false;
		if !sub_flag {
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
				next_carry_flag = true
			}
			if half_carry_flag {
				value = value.wrapping_sub(0x06);
			}
			(value, next_carry_flag)
		}
	}
}

impl ChangesetInstruction for DecimalAdjust {
	type C = ChangeList;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let acc = cpu.register_bank.read_single_named(ACC_REGISTER);
		let sub_flag = cpu.register_bank.read_bit_flag(BitFlags::Subtraction);
		let carry_flag = cpu.register_bank.read_bit_flag(BitFlags::Carry);
		let half_carry_flag = cpu.register_bank.read_bit_flag(BitFlags::HalfCarry);

		let (next_acc, next_carry_flag) = DecimalAdjust::adjust(acc, sub_flag, carry_flag, half_carry_flag);

		Ok(ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, next_acc)),
			Box::new(BitFlagsChange::keep_all()
				.with_zero_flag(next_acc == 0)
				.with_half_carry_flag(false)
				.with_carry_flag(next_carry_flag)
			),
		]))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn adjust_add() {
		/*
			There are 3 possible scenarios when adding two nibbles that represent a decimal digit:
			- Result nibble is a valid digit (0x2 + 0x3 = 0x5)
			- Result nibble is not a valid digit, but doesn't overflow (0x7 + 0x8 = 0xF)
			- Result nibble overflows (0x8 + 0x8 = 0x10)

			When a nibble add overflows, the resulting nibble is a valid digit, though not the correct one.
			A nibble needs to be adjusted if it's not a valid digit, or if it overflows.

			Because we're adding 2 nibbles (the high and low nibbles of a byte), there are 3*3 = 9 possible combinations
			of possible results.
		*/

		// High nibble is valid

		assert_eq!(
			DecimalAdjust::adjust(0x12 + 0x34, false, false, false),
			(0x46, false)
		); // Low nibble is valid

		assert_eq!(
			DecimalAdjust::adjust(0x12 + 0x38, false, false, false),
			(0x50, false)
		); // Low nibble is not valid

		assert_eq!(
			DecimalAdjust::adjust(0x18 + 0x38, false, false, true),
			(0x56, false)
		); // Low nibble overflows

		// High nibble is not valid

		assert_eq!(
			DecimalAdjust::adjust(0x52 + 0x54, false, false, false),
			(0x06, true)
		); // Low nibble is valid

		assert_eq!(
			DecimalAdjust::adjust(0x52 + 0x58, false, false, false),
			(0x10, true)
		); // Low nibble is not valid

		assert_eq!(
			DecimalAdjust::adjust(0x58 + 0x58, false, false, true),
			(0x16, true)
		); // Low nibble overflows

		// High nibble overflows

		assert_eq!(
			DecimalAdjust::adjust(0x82u8.wrapping_add(0x84), false, true, false),
			(0x66, true)
		); // Low nibble is valid

		assert_eq!(
			DecimalAdjust::adjust(0x82u8.wrapping_add(0x88), false, true, false),
			(0x70, true)
		); // Low nibble is not valid

		assert_eq!(
			DecimalAdjust::adjust(0x88u8.wrapping_add(0x88), false, true, true),
			(0x76, true)
		); // Low nibble overflows
	}

	#[test]
	fn adjust_sub() {
		/*
			When subbing nibbles, an underflow (borrow from the next highest bit) can occur, if the minuend is less than
			the subtrahend. THe resulting nibble may or may not represent a valid digit, but the result will always be
			invalid.

			Half carry and carry flag indicate if a borrow happens on the subtractions from the lower and higher nibbles.
			Because a nibble subtraction may or not borrow, and there are two nibbles in a byte, there are 2 * 2 = 4
			possible scenarios
		*/

		// High nibble no borrow

		assert_eq!(
			DecimalAdjust::adjust(0x34 - 0x12, true, false, false),
			(0x22, false)
		); // Low nibble no borrow

		assert_eq!(
			DecimalAdjust::adjust(0x34 - 0x15, true, false, true),
			(0x19, false)
		); // Low nibble borrow

		// High nibble borrow

		assert_eq!(
			DecimalAdjust::adjust(0x34u8.wrapping_sub(0x42), true, true, false),
			(0x92, true)
		); // Low nibble no borrow

		assert_eq!(
			DecimalAdjust::adjust(0x34u8.wrapping_sub(0x45), true, true, true),
			(0x89, true)
		); // Low nibble borrow
	}

	#[test]
	fn decimal_adjust() {
		let mut cpu = Cpu::new();
		cpu.register_bank.write_single_named(ACC_REGISTER, 0x32 + 0x18);
		cpu.register_bank.write_bit_flag(BitFlags::HalfCarry, true);

		let instruction = DecimalAdjust::new();
		let actual = instruction.compute_change(&cpu).expect("Compute changes");
		let expected = ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, 0x50)),
			Box::new(
				BitFlagsChange::keep_all()
					.with_zero_flag(false)
					.with_carry_flag(false)
					.with_half_carry_flag(false)
			),
		]);

		assert_eq!(
			actual,
			expected
		);
	}
}