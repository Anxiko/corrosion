use crate::bits::byte_to_bits;

const DISPLAY_ENABLED_BIT: usize = 7;
const TILE_SOURCE_BIT: usize = 4;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(super) enum TileDataAddressMode {
	/* The addressing mode used by BG and Window for tile data*/
	SignedAddressing,   // 0x8800 to 0x97FF
	UnsignedAddressing, // 0x8000 to 0x8FFF
}

impl From<bool> for TileDataAddressMode {
	fn from(value: bool) -> Self {
		match value {
			false => Self::SignedAddressing,
			true => Self::UnsignedAddressing,
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(super) enum TileMapAddressMode {
	/* The addressing mode used by BG and Window for the tile map */
	First,  // 0x9800 to 0x9BFF
	Second, // 0x9C00 to 0x9FFF
}

impl From<bool> for TileMapAddressMode {
	fn from(value: bool) -> Self {
		match value {
			false => Self::First,
			true => Self::Second,
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(super) enum ObjSize {
	SingleTile,
	DoubleTile,
}

impl From<bool> for ObjSize {
	fn from(value: bool) -> Self {
		match value {
			false => Self::SingleTile,
			true => Self::DoubleTile,
		}
	}
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct DecodedLcdControl {
	pub display_enabled: bool,
	pub window_tile_map: TileMapAddressMode,
	pub window_enable: bool,
	pub tile_data: TileDataAddressMode,
	pub bg_tile_map: TileMapAddressMode,
	pub obj_size: ObjSize,
	pub obj_enable: bool,
	pub bg_and_window_priority: bool,
}

impl From<u8> for DecodedLcdControl {
	fn from(value: u8) -> Self {
		let [bg_and_window_priority, obj_enable, obj_size, bg_tile_map, tile_data, window_enable, window_tile_map, display_enabled] =
			byte_to_bits(value);

		Self {
			display_enabled,
			window_tile_map: window_tile_map.into(),
			window_enable,
			tile_data: tile_data.into(),
			bg_tile_map: bg_tile_map.into(),
			obj_size: obj_size.into(),
			obj_enable,
			bg_and_window_priority,
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
			window_tile_map: TileMapAddressMode::First,
			window_enable: true,
			tile_data: TileDataAddressMode::UnsignedAddressing,
			bg_tile_map: TileMapAddressMode::First,
			obj_size: ObjSize::DoubleTile,
			obj_enable: false,
			bg_and_window_priority: true,
		};

		assert_eq!(actual, expected);
	}

	#[test]
	fn decode_inverse_control() {
		let actual = DecodedLcdControl::from(!LCD_CONTROL);
		let expected = DecodedLcdControl {
			display_enabled: true,
			window_tile_map: TileMapAddressMode::Second,
			window_enable: false,
			tile_data: TileDataAddressMode::SignedAddressing,
			bg_tile_map: TileMapAddressMode::Second,
			obj_size: ObjSize::SingleTile,
			obj_enable: true,
			bg_and_window_priority: false,
		};

		assert_eq!(actual, expected);
	}
}
