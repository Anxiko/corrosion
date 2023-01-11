use crate::core::registers::RegisterBankError::AddressOutOfRange;

const REGISTER_BANK_SIZE: usize = 8;
const FLAG_REGISTER: usize = 6;

struct RegisterBank {
	register_bank: [u8; REGISTER_BANK_SIZE],
}

impl RegisterBank {
	fn new() -> Self {
		Self {
			register_bank: [0u8; REGISTER_BANK_SIZE]
		}
	}

	fn read_single(&self, address: usize) -> Result<u8, RegisterBankError> {
		self.register_bank.get(address).copied()
			.ok_or(RegisterBankError::AddressOutOfRange { address })
	}

	fn write_single(&mut self, address: usize, value: u8) -> Result<(), RegisterBankError> {
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

	fn read_double(&self, address: usize) -> Result<u16, RegisterBankError> {
		let (high_address, low_address) = Self::get_double_address(address)
			.ok_or(RegisterBankError::AddressOutOfRange { address })?;
		let high = self.read_single(high_address)?;
		let low = self.read_single(low_address)?;
		Ok(u16::from_be_bytes([high, low]))
	}

	fn write_double(&mut self, address: usize, value: u16) -> Result<(), RegisterBankError> {
		let (high_address, low_address) = Self::get_double_address(address)
			.ok_or(RegisterBankError::InvalidDoubleRegister { address })?;
		let [high, low] = value.to_be_bytes();

		let high_register = self.register_bank.get_mut(high_address)
			.ok_or(AddressOutOfRange { address: high_address })?;
		*high_register = high;

		let low_register = self.register_bank.get_mut(low_address)
			.ok_or(RegisterBankError::InvalidDoubleRegister { address: low_address })?;
		*low_register = low;

		Ok(())
	}
}

enum RegisterBankError {
	AddressOutOfRange { address: usize },
	InvalidDoubleRegister { address: usize },
}