pub(crate) fn byte_to_bits(byte: u8) -> [bool; 8] {
	let mut bits = [false; 8];

	for idx in 0..bits.len() {
		if byte & (1 << idx) != 0 {
			bits[idx] = true;
		}
	}

	bits
}

pub(crate) fn bits_to_byte<const N: usize>(bits: &[bool; N]) -> u8 {
	bits.iter().fold(0, |acc, &bit| (acc << 1) | bit as u8)
}