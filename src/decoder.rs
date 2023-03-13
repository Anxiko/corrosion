use crate::bits::byte_to_bits;
use crate::decoder::prefixed::{decode_prefixed_shifting, decode_prefixed_single_bit};
use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::{BitFlags, DoubleRegisters, SingleRegisters};
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::base::{ByteDestination, ByteSource, DoubleByteDestination, DoubleByteSource};
use crate::instructions::control::{NopInstruction, StopInstruction};
use crate::instructions::double_arithmetic::{BinaryDoubleAddInstruction, BinaryDoubleAddOperation, IncOrDecDoubleInstruction, IncOrDecDoubleOperation, IncOrDecDoubleType};
use crate::instructions::jump::{JumpInstruction, JumpInstructionCondition, JumpInstructionDestination};
use crate::instructions::load::byte_load::{ByteLoadIndex, ByteLoadInstruction, ByteLoadOperation, ByteLoadUpdate, ByteLoadUpdateType};
use crate::instructions::load::double_byte_load::{DoubleByteLoadInstruction, DoubleByteLoadOperation};
use crate::instructions::single_bit::SingleBitOperation;

mod prefixed;

enum DecoderError {
	ExecutionError(ExecutionError)
}

enum DecoderState {
	Empty,
	WithPrefix { prefix: DecodedInstructionPrefix },
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum DecodedInstructionPrefix {
	CB
}

impl DecodedInstructionPrefix {
	fn try_decode_prefix(maybe_prefix: u8) -> Option<DecodedInstructionPrefix> {
		match maybe_prefix {
			0xCB => Some(Self::CB),
			_ => None
		}
	}
}

fn decoder(cpu: &mut Cpu) -> Result<Box<dyn Instruction>, DecoderError> {
	let first_byte = cpu.next_byte()?;

	let prefix = DecodedInstructionPrefix::try_decode_prefix(first_byte);
	let opcode: u8;

	if prefix.is_some() {
		opcode = cpu.next_byte()?;
	} else {
		opcode = first_byte;
	}

	decode_opcode(prefix, opcode, cpu)
}

fn decode_opcode(
	prefix: Option<DecodedInstructionPrefix>, opcode: u8, cpu: &mut Cpu,
) -> Result<Box<dyn Instruction>, DecoderError> {
	let (x, y, z) = decode_xyz(opcode);

	match prefix {
		Some(DecodedInstructionPrefix::CB) => {
			match x {
				[false, false] /* x = 0 */ => Ok(decode_prefixed_shifting(y, z)),
				[true, false] /* x = 1 */ => Ok(decode_prefixed_single_bit(
					SingleBitOperation::Test, y, z,
				)),
				[false, true] /* x = 2 */ => Ok(decode_prefixed_single_bit(
					SingleBitOperation::Write(false), y, z,
				)),
				[true, true] /* x = 3 */ => Ok(decode_prefixed_single_bit(
					SingleBitOperation::Write(true), y, z,
				)),
			}
		},
		None => {
			match x {
				[false, false] /* x = 0 */ => {
					match z {
						[false, false, false] /* z = 0 */ => {
							match y {
								[false, false, false] /* y = 0 */ => {
									Ok(Box::new(NopInstruction::new()))
								}
								[true, false, false] /* y = 1 */ => {
									let address = load_next_u16(cpu)?;

									Ok(Box::new(DoubleByteLoadInstruction::new(
										DoubleByteSource::Immediate(address),
										DoubleByteDestination::StackPointer,
										DoubleByteLoadOperation,
									)))
								},
								[false, true, false] /* y = 2 */ => {
									Ok(Box::new(StopInstruction::new()))
								},
								[true, true, false] /* y = 3 */ => {
									let delta = load_i8(cpu)?;

									Ok(Box::new(JumpInstruction::new(
										JumpInstructionDestination::Relative(delta),
										JumpInstructionCondition::Unconditional,
									)))
								}
								[y0, y1, true]/* 4 <= y < 8 */ => {
									let flag = match y1 {
										false => BitFlags::Zero,
										true => BitFlags::Carry
									};
									let branch_if_equals = y0;
									let delta = load_i8(cpu)?;

									Ok(Box::new(JumpInstruction::new(
										JumpInstructionDestination::Relative(delta),
										JumpInstructionCondition::TestFlag { flag, branch_if_equals },
									)))
								}
							}
						}
						[true, false, false] /* z = 1 */ => {
							let [y0, y1, y2] = y;
							let q = y0;
							let p = [y1, y2];

							let double_register_operand =
								DecodedInstructionDoubleRegister::from_opcode_part_double_or_sp(p);

							match q {
								false => {
									let immediate = load_next_u16(cpu)?;

									Ok(Box::new(DoubleByteLoadInstruction::new(
										DoubleByteSource::Immediate(immediate),
										double_register_operand.into(),
										DoubleByteLoadOperation::new(),
									)))
								},
								true => {
									Ok(Box::new(BinaryDoubleAddInstruction::new(
										DoubleByteSource::DoubleRegister(DoubleRegisters::HL),
										double_register_operand.into(),
										DoubleByteDestination::DoubleRegister(DoubleRegisters::HL),
										BinaryDoubleAddOperation,
									)))
								}
							}
						},
						[false, true, false] /* z = 2 */ => {
							let [y0, y1, y2] = y;
							let q = y0;
							let p = [y1, y2];

							match p[1] {
								false /* 0 <= p < 2*/ => {
									let register_address = match p[0] {
										false /* p = 0 */ => {
											DoubleRegisters::BC
										},
										true /* p = 1 */ => {
											DoubleRegisters::DE
										}
									};

									let (destination, source) = match q {
										false /* q = 0 */ => {
											(
												ByteDestination::AddressInRegister(register_address),
												ByteSource::Acc
											)
										},
										true /* q = 1 */ => {
											(
												ByteDestination::Acc,
												ByteSource::AddressInRegister(register_address)
											)
										}
									};

									Ok(Box::new(ByteLoadInstruction::new(
										source, destination, ByteLoadOperation::no_update(),
									)))
								}
								true /* 2 <= p < 4 */ => {
									let update_type = match p[0] {
										false /* p = 3 */ => {
											ByteLoadUpdateType::Increment
										}
										true /* p = 4 */ => {
											ByteLoadUpdateType::Decrement
										}
									};

									let update = ByteLoadUpdate::new(
										ByteLoadIndex::DoubleRegister(DoubleRegisters::HL),
										update_type,
									);

									let operation = ByteLoadOperation::with_update(update);

									let (destination, source) = match q {
										false /* q = 0 */ => {
											(
												ByteDestination::AddressInRegister(DoubleRegisters::HL),
												ByteSource::Acc
											)
										}
										true /* q = 1 */ => {
											(
												ByteDestination::Acc,
												ByteSource::AddressInRegister(DoubleRegisters::HL)
											)
										}
									};

									Ok(Box::new(ByteLoadInstruction::new(
										source, destination, operation,
									)))
								}
							}
						},
						[true, true, false] /* z = 3 */ => {
							let (p, q) = decode_pq(y);

							let decoded_double_operator = DecodedInstructionDoubleRegister::from_opcode_part_double_or_sp(p);


							let inc_or_dec_type = match q {
								false => IncOrDecDoubleType::Increment,
								true => IncOrDecDoubleType::Decrement
							};

							Ok(Box::new(IncOrDecDoubleInstruction::new(
								decoded_double_operator.into(),
								decoded_double_operator.into(),
								IncOrDecDoubleOperation::new(inc_or_dec_type),
							)))
						},
						_ => todo!()
					}
				}
				_ => todo!()
			}
		}
	}
}

fn decode_pq(y: [bool; 3]) -> ([bool; 2], bool) {
	let [y0, y1, y2] = y;
	let q = y0;
	let p = [y1, y2];
	(p, q)
}

fn load_next_u16(cpu: &mut Cpu) -> Result<u16, ExecutionError> {
	let low = cpu.next_byte()?;
	let high = cpu.next_byte()?;

	Ok(u16::from_be_bytes([low, high]))
}

fn load_i8(cpu: &mut Cpu) -> Result<i8, ExecutionError> {
	let delta = cpu.next_byte()?;
	let delta = delta as i8;

	Ok(delta)
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum DecodedInstructionOperand {
	SingleRegister(SingleRegisters),
	HlMemoryAddress,
}

impl DecodedInstructionOperand {
	fn from_opcode_part(opcode_part: [bool; 3]) -> Self {
		match opcode_part {
			[false, false, false] => Self::SingleRegister(SingleRegisters::B), // 0 => B
			[true, false, false] => Self::SingleRegister(SingleRegisters::C), // 1 => C
			[false, true, false] => Self::SingleRegister(SingleRegisters::D), // 2 => D
			[true, true, false] => Self::SingleRegister(SingleRegisters::E), // 3 => E
			[false, false, true] => Self::SingleRegister(SingleRegisters::H), // 4 => H
			[true, false, true] => Self::SingleRegister(SingleRegisters::L), // 5 => L
			[false, true, true] => Self::HlMemoryAddress, // 6 => (HL)
			[true, true, true] => Self::SingleRegister(SingleRegisters::A), // 7 => A
		}
	}
}


impl From<DecodedInstructionOperand> for ByteSource {
	fn from(value: DecodedInstructionOperand) -> Self {
		match value {
			DecodedInstructionOperand::SingleRegister(single_reg) => Self::read_from_single(single_reg),
			DecodedInstructionOperand::HlMemoryAddress => Self::read_from_register_address(DoubleRegisters::HL)
		}
	}
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum DecodedInstructionDoubleRegister {
	DoubleRegister(DoubleRegisters),
	Sp,
	Af,
}

impl DecodedInstructionDoubleRegister {
	fn from_opcode_maybe_double(opcode_part: [bool; 2]) -> Option<DoubleRegisters> {
		match opcode_part {
			[false, false] => Some(DoubleRegisters::BC),
			[true, false] => Some(DoubleRegisters::DE),
			[false, true] => Some(DoubleRegisters::HL),
			[true, true] => None
		}
	}

	fn from_opcode_part_double_or_sp(opcode_part: [bool; 2]) -> Self {
		Self::from_opcode_maybe_double(opcode_part)
			.map_or(Self::Sp, |double_register| Self::DoubleRegister(double_register))
	}

	fn from_opcode_part_double_or_af(opcode_part: [bool; 2]) -> Self {
		Self::from_opcode_maybe_double(opcode_part)
			.map_or(Self::Af, |double_register| Self::DoubleRegister(double_register))
	}
}

impl From<DecodedInstructionDoubleRegister> for DoubleByteSource {
	fn from(value: DecodedInstructionDoubleRegister) -> Self {
		match value {
			DecodedInstructionDoubleRegister::DoubleRegister(double_reg) => Self::DoubleRegister(double_reg),
			DecodedInstructionDoubleRegister::Af => Self::DoubleRegister(DoubleRegisters::AF),
			DecodedInstructionDoubleRegister::Sp => Self::StackPointer
		}
	}
}

impl From<DecodedInstructionDoubleRegister> for DoubleByteDestination {
	fn from(value: DecodedInstructionDoubleRegister) -> Self {
		match value {
			DecodedInstructionDoubleRegister::DoubleRegister(double_reg) => Self::DoubleRegister(double_reg),
			DecodedInstructionDoubleRegister::Af => Self::DoubleRegister(DoubleRegisters::AF),
			DecodedInstructionDoubleRegister::Sp => Self::StackPointer
		}
	}
}

fn decode_xyz(opcode: u8) -> ([bool; 2], [bool; 3], [bool; 3]) {
	let bits = byte_to_bits(opcode);

	let x = bits[6..8].try_into().unwrap();
	let y = bits[3..6].try_into().unwrap();
	let z = bits[0..3].try_into().unwrap();

	(x, y, z)
}

impl From<ExecutionError> for DecoderError {
	fn from(value: ExecutionError) -> Self {
		DecoderError::ExecutionError(value)
	}
}