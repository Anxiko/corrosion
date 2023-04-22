use crate::hardware::ram::{Ram, RamError, Rom};

// TODO: implement audio
pub(crate) struct Audio;

impl Rom for Audio {
	fn read_byte(&self, _address: u16) -> Result<u8, RamError> {
		Ok(0)
	}
}

impl Ram for Audio {
	fn write_byte(&mut self, _address: u16, _value: u8) -> Result<(), RamError> {
		Ok(())
	}
}
