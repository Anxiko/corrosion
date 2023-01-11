use corrosion::hardware::register_bank::{DOUBLE_REGISTER_BANK_SIZE, RegisterBank, RegisterBankError, SINGLE_REGISTER_BANK_SIZE};

#[test]
fn single_register() {
	let mut register_bank = RegisterBank::new();

	for register in 0..SINGLE_REGISTER_BANK_SIZE {
		let register_value = 0x12u8 + register as u8;
		assert!(register_bank.write_single(register, register_value).is_ok());
	}

	for register in 0..SINGLE_REGISTER_BANK_SIZE {
		let expected_register_value = 0x12u8 + register as u8;
		assert_eq!(register_bank.read_single(register), Ok(expected_register_value));
	}

	assert_eq!(
		register_bank.read_single(SINGLE_REGISTER_BANK_SIZE),
		Err(RegisterBankError::AddressOutOfRange { address: SINGLE_REGISTER_BANK_SIZE })
	);

	assert_eq!(
		register_bank.write_single(SINGLE_REGISTER_BANK_SIZE, 0x12),
		Err(RegisterBankError::AddressOutOfRange { address: SINGLE_REGISTER_BANK_SIZE })
	);
}

#[test]
fn double_register() {
	let mut register_bank = RegisterBank::new();

	for register in 0..DOUBLE_REGISTER_BANK_SIZE {
		let register_value = 0xab12u16 + register as u16;
		assert!(register_bank.write_double(register, register_value).is_ok());
	}

	for register in 0..DOUBLE_REGISTER_BANK_SIZE {
		let expected_register_value = 0xab12u16 + register as u16;
		assert_eq!(register_bank.read_double(register), Ok(expected_register_value));
	}

	assert_eq!(
		register_bank.read_double(DOUBLE_REGISTER_BANK_SIZE),
		Err(RegisterBankError::InvalidDoubleRegister { address: DOUBLE_REGISTER_BANK_SIZE })
	);

	assert_eq!(
		register_bank.write_double(DOUBLE_REGISTER_BANK_SIZE, 0xab12),
		Err(RegisterBankError::InvalidDoubleRegister { address: DOUBLE_REGISTER_BANK_SIZE })
	);
}