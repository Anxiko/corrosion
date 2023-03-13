use crate::instructions::changeset::BitFlagsChange;

pub(crate) struct AluU8Result {
	pub(crate) sub: bool,
	pub(crate) result: u8,
	pub(crate) half_carry: bool,
	pub(crate) carry: bool,
}

impl From<AluU8Result> for BitFlagsChange {
	fn from(value: AluU8Result) -> Self {
		Self::new(
			Some(value.result == 0),
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