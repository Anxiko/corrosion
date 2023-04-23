use super::RamError;

pub(crate) trait Rom {
	fn read_byte(&self, address: u16) -> Result<u8, RamError>;

	fn read_double_byte(&self, address: u16) -> Result<u16, RamError> {
		let low = self.read_byte(address)?;
		let high = self.read_byte(address.wrapping_add(1))?;

		Ok(u16::from_be_bytes([high, low]))
	}
}

pub(crate) trait Ram: Rom {
	fn write_byte(&mut self, address: u16, value: u8) -> Result<(), RamError>;
	fn write_double_byte(&mut self, address: u16, value: u16) -> Result<(), RamError> {
		let [high, low] = value.to_be_bytes();
		self.write_byte(address, low)?;
		self.write_byte(address.wrapping_add(1), high)?;

		Ok(())
	}
}

impl Rom for u8 {
	fn read_byte(&self, address: u16) -> Result<u8, RamError> {
		if address == 0 {
			Ok(*self)
		} else {
			Err(RamError::InvalidAddress(address))
		}
	}

	fn read_double_byte(&self, address: u16) -> Result<u16, RamError> {
		Err(RamError::InvalidAddress(address))
	}
}

impl Ram for u8 {
	fn write_byte(&mut self, address: u16, value: u8) -> Result<(), RamError> {
		if address == 0 {
			*self = value;
			Ok(())
		} else {
			Err(RamError::InvalidAddress(address))
		}
	}

	fn write_double_byte(&mut self, address: u16, _value: u16) -> Result<(), RamError> {
		Err(RamError::InvalidAddress(address))
	}
}
