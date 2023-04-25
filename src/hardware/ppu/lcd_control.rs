use crate::bits::byte_to_bits;

const DISPLAY_ENABLED_BIT: usize = 7;
const TILE_SOURCE_BIT: usize = 4;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(super) enum TileSource {
	/* The addressing mode used for BG and Window */
	SignedAddressing,   // 0x8800 to 0x97FF
	UnsignedAddressing, // 0x8000 to 0x8FFF
}

impl From<bool> for TileSource {
	fn from(value: bool) -> Self {
		match value {
			false => TileSource::SignedAddressing,
			true => TileSource::UnsignedAddressing,
		}
	}
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct DecodedLcdControl {
	pub display_enabled: bool,
	pub tile_source: TileSource,
}

impl From<u8> for DecodedLcdControl {
	fn from(value: u8) -> Self {
		let bits = byte_to_bits(value);
		Self {
			display_enabled: bits[DISPLAY_ENABLED_BIT],
			tile_source: TileSource::from(bits[TILE_SOURCE_BIT]),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const LCD_CONTROL: u8 = 0b0011_0101;

	#[test]
	fn decode_control() {
		let actual = DecodedLcdControl::from(LCD_CONTROL);
		let expected = DecodedLcdControl {
			display_enabled: false,
			tile_source: TileSource::UnsignedAddressing,
		};

		assert_eq!(actual, expected);
	}

	#[test]
	fn decode_inverse_control() {
		let actual = DecodedLcdControl::from(!LCD_CONTROL);
		let expected = DecodedLcdControl {
			display_enabled: true,
			tile_source: TileSource::SignedAddressing,
		};

		assert_eq!(actual, expected);
	}
}
