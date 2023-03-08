pub(crate) fn byte_to_bits(byte: u8) -> [bool; 8] {
	let mut bits = [false; 8];

	for idx in 0..bits.len() {
		if byte & ( 1 << idx) != 0 {
			bits[idx] = true;
		}
	}

	bits
}