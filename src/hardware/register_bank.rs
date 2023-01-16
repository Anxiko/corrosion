use num_enum::IntoPrimitive;

#[cfg(test)]
mod tests;

pub const SINGLE_REGISTER_BANK_SIZE: usize = 8;
pub const DOUBLE_REGISTER_BANK_SIZE: usize = SINGLE_REGISTER_BANK_SIZE / 2;

#[derive(IntoPrimitive, Copy, Clone)]
#[repr(u8)]
pub enum SingleRegisters {
	A = 0,
	B,
	C,
	D,
	E,
	F,
	H,
	L,
}

#[derive(IntoPrimitive, Copy, Clone)]
#[repr(u8)]
pub enum DoubleRegisters {
	AF = 0,
	BC,
	DE,
	HL,
}

impl DoubleRegisters {
	fn get_high_and_low(&self) -> (SingleRegisters, SingleRegisters) {
		match self {
			Self::AF => (SingleRegisters::A, SingleRegisters::F),
			Self::BC => (SingleRegisters::B, SingleRegisters::C),
			Self::DE => (SingleRegisters::D, SingleRegisters::E),
			Self::HL => (SingleRegisters::H, SingleRegisters::L)
		}
	}
}

const FLAG_REGISTER: usize = 6;

#[derive(IntoPrimitive, Copy, Clone)]
#[repr(u8)]
pub enum RegisterFlags {
	Zero = 7,
	Subtraction = 6,
	HalfCarry = 5,
	Carry = 4,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
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

	pub fn read_bit_flag(&self, flag: RegisterFlags) -> bool {
		let flag: u8 = flag.into();
		let bitmask: u8 = 1u8 << flag;

		let flag_register = self.read_single(FLAG_REGISTER).unwrap();

		flag_register & bitmask != 0
	}

	pub fn write_bit_flag(&mut self, flag: RegisterFlags, bit: bool) {
		let flag: u8 = flag.into();
		let bitmask: u8 = 1u8 << flag;
		let shifted_bit: u8 =
			if bit {
				bitmask
			} else {
				0
			};

		let flag_register: u8 = self.read_single(FLAG_REGISTER).unwrap();
		let new_flag_register = (flag_register & (!bitmask)) | shifted_bit;

		self.write_single(FLAG_REGISTER, new_flag_register).unwrap();
	}

	pub fn read_single_named(&self, single_register: SingleRegisters) -> u8 {
		let address: u8 = single_register.into();
		self.read_single(address as usize).unwrap()
	}

	pub fn write_single_named(&mut self, single_register: SingleRegisters, value: u8) {
		let address: u8 = single_register.into();
		self.write_single(address as usize, value).unwrap();
	}


	pub fn read_double_named(&self, double_register: DoubleRegisters) -> u16 {
		let address: u8 = double_register.into();
		self.read_double(address as usize).unwrap()
	}

	pub fn write_double_named(&mut self, double_register: DoubleRegisters, value: u16) {
		let address: u8 = double_register.into();
		self.write_double(address as usize, value).unwrap();
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegisterBankError {
	AddressOutOfRange { address: usize },
	InvalidDoubleRegister { address: usize },
}

const PC_START: u16 = 0x100;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ProgramCounter {
	pc: u16,
}

impl ProgramCounter {
	pub(crate) fn new() -> Self {
		Self { pc: PC_START }
	}

	pub(crate) fn read(&self) -> u16 {
		self.pc
	}

	pub(crate) fn write(&mut self, value: u16) {
		self.pc = value;
	}

	pub(crate) fn increment(&mut self) {
		let (result, _overflow) = self.pc.overflowing_add(1);
		self.pc = result;
	}
}