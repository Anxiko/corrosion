use crate::instructions::base::ByteDestination;
use crate::instructions::changeset::{BitFlagsChange, Change};

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct AluU8Result {
	pub(crate) result: u8,
	pub(crate) sub: bool,
	pub(crate) half_carry: bool,
	pub(crate) carry: bool,
}

impl AluU8Result {
	pub(crate) fn zero(&self) -> bool {
		self.result == 0
	}

	pub(crate) fn change_flags(&self) -> BitFlagsChange {
		BitFlagsChange::new(
			Some(self.zero()),
			Some(self.sub),
			Some(self.half_carry),
			Some(self.carry),
		)
	}

	pub(crate) fn change_dst(&self, dst: &ByteDestination) -> Box<dyn Change> {
		dst.change_destination(self.result)
	}
}

impl From<AluU8Result> for BitFlagsChange {
	fn from(value: AluU8Result) -> Self {
		Self::new(
			Some(value.zero()),
			Some(value.sub),
			Some(value.half_carry),
			Some(value.carry),
		)
	}
}

pub(crate) fn add_u8(left: u8, right: u8) -> AluU8Result {
	add_with_carry_u8(left, right, false)
}

pub(crate) fn add_with_carry_u8(left: u8, right: u8, carry: bool) -> AluU8Result {
	let carry_val: u8 = carry.into();

	let (result, first_overflow) = left.overflowing_add(right);
	let (result, second_overflow) = result.overflowing_add(carry_val);

	let half_carry = half_carry_for_add_u8(left, right, carry);

	AluU8Result {
		sub: false,
		result,
		half_carry,
		carry: first_overflow || second_overflow,
	}
}

pub(crate) fn sub_u8(left: u8, right: u8) -> AluU8Result {
	sub_u8_with_carry(left, right, false)
}

pub(crate) fn sub_u8_with_carry(left: u8, right: u8, carry: bool) -> AluU8Result {
	let carry_val: u8 = carry.into();

	let (result, first_underflow) = left.overflowing_sub(right);
	let (result, second_underflow) = result.overflowing_sub(carry_val);

	AluU8Result {
		sub: true,
		result,
		carry: first_underflow || second_underflow,
		half_carry: half_carry_for_sub_u8(left, right, carry),
	}
}

pub(crate) fn delta_u8(left: u8, right: i8) -> AluU8Result {
	if right < 0 {
		sub_u8(left, right.unsigned_abs())
	} else {
		add_u8(left, right.unsigned_abs())
	}
}

fn half_carry_for_add_u8(left: u8, right: u8, carry: bool) -> bool {
	let carry: u8 = carry.into();
	((left & 0x0F) + (right & 0x0F) + carry) > 0x0F
}

fn half_carry_for_sub_u8(left: u8, right: u8, carry: bool) -> bool {
	let carry_val: u8 = carry.into();
	(left & 0x0F) < ((right & 0x0F) + carry_val)
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn arithmetic_add() {
		assert_eq!(
			add_u8(0x12, 0x34),
			AluU8Result {
				result: 0x46,
				sub: false,
				half_carry: false,
				carry: false,
			}
		);

		assert_eq!(
			add_u8(0x46, 0x0A),
			AluU8Result {
				result: 0x50,
				sub: false,
				half_carry: true,
				carry: false,
			}
		);

		assert_eq!(
			add_u8(0x50, 0xB0),
			AluU8Result {
				result: 0x00,
				sub: false,
				half_carry: false,
				carry: true,
			}
		)
	}

	#[test]
	fn arithmetic_add_with_carry() {
		assert_eq!(
			add_with_carry_u8(0x08, 0x07, true),
			AluU8Result {
				result: 0x10,
				sub: false,
				half_carry: true,
				carry: false,
			}
		);

		assert_eq!(
			add_with_carry_u8(0x80, 0x7F, true),
			AluU8Result {
				result: 0x00,
				sub: false,
				half_carry: true,
				carry: true,
			}
		);
	}

	#[test]
	fn arithmetic_sub() {
		assert_eq!(
			sub_u8(0x34, 0x12),
			AluU8Result {
				result: 0x22,
				sub: true,
				half_carry: false,
				carry: false,
			}
		);

		assert_eq!(
			sub_u8(0x31, 0x14),
			AluU8Result {
				result: 0x1D,
				sub: true,
				half_carry: true,
				carry: false,
			}
		);

		assert_eq!(
			sub_u8(0x12, 0x12),
			AluU8Result {
				result: 0x00,
				sub: true,
				half_carry: false,
				carry: false,
			}
		);

		assert_eq!(
			sub_u8(0x10, 0x20),
			AluU8Result {
				result: 0xF0,
				sub: true,
				half_carry: false,
				carry: true,
			}
		)
	}

	#[test]
	fn arithmetic_sub_with_carry() {
		assert_eq!(
			sub_u8_with_carry(0x14, 0x04, true),
			AluU8Result {
				result: 0x0F,
				sub: true,
				half_carry: true,
				carry: false,
			}
		);

		assert_eq!(
			sub_u8_with_carry(0x77, 0x86, true),
			AluU8Result {
				result: 0xF0,
				sub: true,
				half_carry: false,
				carry: true,
			}
		)
	}
}