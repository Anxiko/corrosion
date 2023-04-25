use crate::hardware::ram::{Ram, RamError, Rom};

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub(crate) struct ScreenCord {
	pub y: u8, // Y appears before X in memory mappings
	pub x: u8,
}

impl Rom for ScreenCord {
	fn read_byte(&self, address: u16) -> Result<u8, RamError> {
		match address {
			0 => Ok(self.y),
			1 => Ok(self.y),
			_ => Err(RamError::InvalidAddress(address)),
		}
	}
}

impl Ram for ScreenCord {
	fn write_byte(&mut self, address: u16, value: u8) -> Result<(), RamError> {
		let maybe_ptr = match address {
			0 => Some(&mut self.y),
			1 => Some(&mut self.x),
			_ => None,
		};

		maybe_ptr
			.map(|ptr| {
				*ptr = value;
			})
			.ok_or(RamError::InvalidAddress(address))
	}
}
