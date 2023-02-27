use crate::hardware::cpu::Cpu;
use crate::hardware::ram::{Ram, WORKING_RAM_START};
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use crate::instructions::changeset::{BitFlagsChange, Change, ChangeList, ChangesetInstruction, MemoryByteWriteChange, SingleRegisterChange};
use crate::instructions::ExecutionError;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum SingleBitOperand {
	SingleRegister(SingleRegisters),
	MemoryAddress,
}

impl SingleBitOperand {
	fn operate_on_register(reg: SingleRegisters) -> Self {
		Self::SingleRegister(reg)
	}

	fn read_byte(&self, cpu: &Cpu) -> Result<u8, ExecutionError> {
		match self {
			Self::SingleRegister(reg) => {
				Ok(cpu.register_bank.read_single_named(*reg))
			}
			Self::MemoryAddress => {
				let address = cpu.register_bank.read_double_named(DoubleRegisters::HL);
				let byte = cpu.mapped_ram.read_byte(address)?;
				Ok(byte)
			}
		}
	}

	fn write_change(&self, byte: u8) -> Box<dyn Change> {
		match self {
			Self::SingleRegister(reg) => {
				Box::new(SingleRegisterChange::new(*reg, byte))
			}
			Self::MemoryAddress => {
				Box::new(MemoryByteWriteChange::write_to_register(DoubleRegisters::HL, byte))
			}
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum SingleBitOperation {
	Test,
	Write(bool),
}

impl SingleBitOperation {
	fn compute_change(&self, byte: u8, bitmask: u8, operand: &SingleBitOperand) -> Box<dyn Change> {
		match self {
			Self::Test => {
				let test = byte & bitmask != 0;

				let flags_change = BitFlagsChange::keep_all()
					.with_zero_flag(test)
					.with_subtraction_flag(false)
					.with_half_carry_flag(true);


				Box::new(flags_change)
			}
			Self::Write(bit) => {
				let result = if *bit {
					byte | bitmask
				} else {
					byte & (!bitmask)
				};
				operand.write_change(byte)
			}
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct SingleBitInstruction {
	operand: SingleBitOperand,
	operation: SingleBitOperation,
	bit_shift: [bool; 3],
}

impl SingleBitInstruction {
	fn new(operand: SingleBitOperand, operation: SingleBitOperation, bit_shift: [bool; 3]) -> Self {
		Self { operand, operation, bit_shift }
	}

	fn get_bit(&self) -> u8 {
		self.bit_shift.iter().rev().fold(0, |acc, &bit| (acc << 1) | (bit as u8))
	}
}

impl ChangesetInstruction for SingleBitInstruction {
	type C = Box<dyn Change>;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let byte = self.operand.read_byte(cpu)?;
		let bitmask = self.get_bit();

		Ok(self.operation.compute_change(byte, bitmask, &self.operand))
	}
}

#[test]
fn test_bit() {
	let mut cpu = Cpu::new();

	cpu.register_bank.write_single_named(SingleRegisters::B, 0b11001010);
	cpu.register_bank.write_double_named(DoubleRegisters::HL, WORKING_RAM_START);
	cpu.mapped_ram.write_byte(WORKING_RAM_START, 0b11001010).expect("Write to RAM");

	let cpu =  cpu;

	let instruction = SingleBitInstruction::new(
		SingleBitOperand::SingleRegister(SingleRegisters::B),
		SingleBitOperation::Test,
		[true, true, false],
	);

	let actual = instruction.compute_change(&cpu).expect("Compute changes");
	let expected: Box<dyn Change> = Box::new(
		BitFlagsChange::keep_all()
			.with_zero_flag(true)
			.with_subtraction_flag(false)
			.with_half_carry_flag(true)
	);

	assert_eq!(actual, expected);

	let instruction = SingleBitInstruction::new(
		SingleBitOperand::MemoryAddress,
		SingleBitOperation::Test,
		[false, false, true],
	);

	let actual = instruction.compute_change(&cpu).expect("Compute changes");
	let expected: Box<dyn Change> = Box::new(
		BitFlagsChange::keep_all()
			.with_zero_flag(false)
			.with_subtraction_flag(false)
			.with_half_carry_flag(true)
	);

	assert_eq!(actual, expected);
}