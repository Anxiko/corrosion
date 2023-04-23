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
