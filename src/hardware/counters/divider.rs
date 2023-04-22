use super::Tick;
use crate::hardware::ram::{Ram, RamError, Rom};

#[derive(Debug)]
pub(crate) struct DividerRegister {
	value: u16,
}

impl DividerRegister {
	fn new(value: u16) -> Self {
		Self { value }
	}
}

impl Default for DividerRegister {
	fn default() -> Self {
		DividerRegister::new(0)
	}
}

impl Tick for DividerRegister {
	fn tick(&mut self) {
		let (new_value, overflow) = self.value.overflowing_add(1);
		if overflow {
			// TODO: trigger interrupt
		}
		self.value = new_value;
	}
}

impl Rom for DividerRegister {
	fn read_byte(&self, address: u16) -> Result<u8, RamError> {
		if address == 0 {
			// Only the highest byte is mapped to memory
			Ok(self.value.to_be_bytes()[0])
		} else {
			Err(RamError::InvalidAddress(address))
		}
	}

	fn read_double_byte(&self, address: u16) -> Result<u16, RamError> {
		Err(RamError::InvalidAddress(address))
	}
}

impl Ram for DividerRegister {
	fn write_byte(&mut self, address: u16, _value: u8) -> Result<(), RamError> {
		if address == 0 {
			self.value = 0; // Writes reset the register, no matter the value
			Ok(())
		} else {
			Err(RamError::InvalidAddress(address))
		}
	}

	fn write_double_byte(&mut self, address: u16, _value: u16) -> Result<(), RamError> {
		Err(RamError::InvalidAddress(address))
	}
}
