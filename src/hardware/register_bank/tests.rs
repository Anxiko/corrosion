use crate::hardware::register_bank::{
	DOUBLE_REGISTER_BANK_SIZE, RegisterBank, RegisterBankError, BitFlags, SINGLE_REGISTER_BANK_SIZE,
	SingleRegisters, DoubleRegisters,
};

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

#[test]
fn bit_flags() {
	let mut register_bank = RegisterBank::new();
	for bit_flag in [
		BitFlags::Carry, BitFlags::HalfCarry, BitFlags::Subtraction, BitFlags::Zero
	] {
		assert!(!register_bank.read_bit_flag(bit_flag));
		register_bank.write_bit_flag(bit_flag, true);
		assert!(register_bank.read_bit_flag(bit_flag));
		register_bank.write_bit_flag(bit_flag, false);
		assert!(!register_bank.read_bit_flag(bit_flag));
	}
}

#[test]
fn named_register() {
	let mut register_bank = RegisterBank::new();

	register_bank.write_single_named(SingleRegisters::A, 0x12);
	assert_eq!(register_bank.read_single_named(SingleRegisters::A), 0x12);

	register_bank.write_single_named(SingleRegisters::F, 0x34);
	assert_eq!(register_bank.read_single_named(SingleRegisters::F), 0x34);

	assert_eq!(register_bank.read_double_named(DoubleRegisters::AF), 0x1234);

	register_bank.write_double_named(DoubleRegisters::BC, 0x5678);
	assert_eq!(register_bank.read_double_named(DoubleRegisters::BC), 0x5678);
}