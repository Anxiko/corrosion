use super::{Ram, RamError, Rom};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RomChip<'a, const S: usize> {
	ref_memory: &'a [u8; S],
}

impl<'a, const S: usize> RomChip<'a, S> {
	pub(super) fn new(ref_memory: &'a [u8; S]) -> Self {
		Self { ref_memory }
	}
}

impl<'a, const S: usize> Rom for RomChip<'a, S> {
	fn read_byte(&self, address: u16) -> Result<u8, RamError> {
		self.ref_memory
			.get(usize::from(address))
			.copied()
			.ok_or(RamError::InvalidAddress(address))
	}
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct RamChip<const S: usize> {
	memory: Box<[u8; S]>,
}

impl<const S: usize> RamChip<S> {
	fn new(memory: Box<[u8; S]>) -> Self {
		Self { memory }
	}
}

impl<const S: usize> Default for RamChip<S> {
	fn default() -> Self {
		RamChip::new(Box::new([0; S]))
	}
}

impl<const S: usize> Rom for RamChip<S> {
	fn read_byte(&self, address: u16) -> Result<u8, RamError> {
		self.memory
			.get(usize::from(address))
			.copied()
			.ok_or(RamError::InvalidAddress(address))
	}
}

impl<const S: usize> Ram for RamChip<S> {
	fn write_byte(&mut self, address: u16, value: u8) -> Result<(), RamError> {
		let ptr = self
			.memory
			.get_mut(usize::from(address))
			.ok_or(RamError::InvalidAddress(address))?;

		*ptr = value;
		Ok(())
	}
}
