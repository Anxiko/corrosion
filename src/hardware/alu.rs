pub(crate) struct AluU8Result {
	pub(crate) result: u8,
	pub(crate) half_carry: bool,
	pub(crate) carry: bool,
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
		result,
		half_carry,
		carry: first_overflow || second_overflow,
	}
}

fn half_carry_for_add_u8(left: u8, right: u8, carry: bool) -> bool {
	let carry: u8 = carry.into();
	((left & 0x0F) + (right & 0x0F) + carry) > 0x0F
}