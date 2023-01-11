pub const SINGLE_REGISTER_BANK_SIZE: usize = 8;
pub const DOUBLE_REGISTER_BANK_SIZE: usize = SINGLE_REGISTER_BANK_SIZE / 2;

const FLAG_REGISTER: usize = 6;

#[derive(Default, Debug)]
pub struct RegisterBank {
	register_bank: [u8; SINGLE_REGISTER_BANK_SIZE],
}

impl RegisterBank {
	pub fn new() -> Self {
		Self {
			register_bank: [0u8; SINGLE_REGISTER_BANK_SIZE]
		}
	}

	pub fn read_single(&self, address: usize) -> Result<u8, RegisterBankError> {
		self.register_bank.get(address).copied()
			.ok_or(RegisterBankError::AddressOutOfRange { address })
	}

	pub fn write_single(&mut self, address: usize, value: u8) -> Result<(), RegisterBankError> {
		let register = self.register_bank.get_mut(address)
			.ok_or(RegisterBankError::AddressOutOfRange { address })?;
		*register = value;
		Ok(())
	}

	fn get_double_address(address: usize) -> Option<(usize, usize)> {
		match address {
			0 => Some((0, 5)),
			1 => Some((1, 2)),
			2 => Some((3, 4)),
			3 => Some((6, 7)),
			_ => None
		}
	}

	pub fn read_double(&self, address: usize) -> Result<u16, RegisterBankError> {
		let (high_address, low_address) = Self::get_double_address(address)
			.ok_or(RegisterBankError::InvalidDoubleRegister { address })?;
		let high = self.read_single(high_address)?;
		let low = self.read_single(low_address)?;
		Ok(u16::from_be_bytes([high, low]))
	}

	pub fn write_double(&mut self, address: usize, value: u16) -> Result<(), RegisterBankError> {
		let (high_address, low_address) = Self::get_double_address(address)
			.ok_or(RegisterBankError::InvalidDoubleRegister { address })?;
		let [high, low] = value.to_be_bytes();

		let high_register = self.register_bank.get_mut(high_address)
			.ok_or(RegisterBankError::AddressOutOfRange { address: high_address })?;
		*high_register = high;

		let low_register = self.register_bank.get_mut(low_address)
			.ok_or(RegisterBankError::AddressOutOfRange { address: low_address })?;
		*low_register = low;

		Ok(())
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegisterBankError {
	AddressOutOfRange { address: usize },
	InvalidDoubleRegister { address: usize },
}