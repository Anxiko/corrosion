use std::assert_matches::assert_matches;

use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use crate::instructions::{ACC_REGISTER, ExecutionError, Instruction};
use crate::instructions::base::{BaseByteInstruction, ByteDestination, ByteOperation, ByteSource};
use crate::instructions::changeset::{Change, ChangeList, ChangesetInstruction, DoubleRegisterChange, MemoryByteWriteChange, SingleRegisterChange};

struct LoadByteOperation;

impl LoadByteOperation {
	fn new() -> Self { Self {} }
}

impl ByteOperation for LoadByteOperation {
	type C = Box<dyn Change>;

	fn execute(&self, cpu: &Cpu, src: &ByteSource, dst: &ByteDestination) -> Result<Self::C, ExecutionError> {
		let value = src.read(cpu)?;
		Ok(dst.change_destination(value))
	}
}

type LoadByteInstruction = BaseByteInstruction<LoadByteOperation>;

enum LoadAndUpdateRegister {
	Source(DoubleRegisters),
	Destination(DoubleRegisters),
}

impl LoadAndUpdateRegister {
	fn get_register(&self) -> DoubleRegisters {
		*match self {
			Self::Source(reg) => reg,
			Self::Destination(reg) => reg,
		}
	}
}

enum CounterUpdate {
	Increment,
	Decrement,
}

impl CounterUpdate {
	fn as_number(&self) -> i16 {
		match self {
			Self::Increment => 1,
			Self::Decrement => -1,
		}
	}

	fn update(&self, value: u16) -> u16 {
		let delta = self.as_number();
		value.wrapping_add_signed(delta)
	}
}

struct LoadAndUpdateInstruction {
	reg: LoadAndUpdateRegister,
	update: CounterUpdate,
}

impl LoadAndUpdateInstruction {
	fn load_from_register(reg: DoubleRegisters, update: CounterUpdate) -> Self {
		Self { reg: LoadAndUpdateRegister::Source(reg), update }
	}

	fn load_to_register(reg: DoubleRegisters, update: CounterUpdate) -> Self {
		Self { reg: LoadAndUpdateRegister::Destination(reg), update }
	}

	fn read(&self, cpu: &Cpu) -> Result<u8, ExecutionError> {
		match self.reg {
			LoadAndUpdateRegister::Source(reg) => {
				let address = cpu.register_bank.read_double_named(reg);
				let value = cpu.mapped_ram.read(address)?;
				Ok(value)
			},
			LoadAndUpdateRegister::Destination(_) => {
				Ok(cpu.register_bank.read_single_named(ACC_REGISTER))
			}
		}
	}

	fn write(&self, value: u8, cpu: &Cpu) -> Box<dyn Change> {
		match self.reg {
			LoadAndUpdateRegister::Destination(reg) => {
				Box::new(MemoryByteWriteChange::write_to_register(reg, value))
			},
			LoadAndUpdateRegister::Source(_) => {
				Box::new(SingleRegisterChange::new(ACC_REGISTER, value))
			}
		}
	}

	fn register_and_address(&self, cpu: &Cpu) -> (DoubleRegisters, u16) {
		let double_register = self.reg.get_register();
		let address = cpu.register_bank.read_double_named(double_register);

		(double_register, address)
	}
}

impl ChangesetInstruction for LoadAndUpdateInstruction {
	type C = ChangeList;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let value = self.read(cpu)?;
		let write_change = self.write(value, cpu);

		let (register, address) = self.register_and_address(cpu);
		let address = self.update.update(address);
		let update_change = DoubleRegisterChange::new(register, address);

		Ok(ChangeList::new(vec![
			write_change,
			Box::new(update_change),
		]))
	}
}


#[test]
fn load_operation() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(SingleRegisters::B, 0b1111_0000);
	let result = LoadByteOperation::new().execute(
		&cpu, &ByteSource::SingleRegister { single_reg: SingleRegisters::B }, &ByteDestination::Acc,
	).expect("Operation to execute");
	let expected: Box<dyn Change> = Box::new(SingleRegisterChange::new(ACC_REGISTER, 0b1111_0000));

	assert_eq!(
		result,
		expected
	);
}

#[test]
fn load_instruction() {
	let mut cpu = Cpu::new();
	cpu.register_bank.write_single_named(SingleRegisters::B, 0b1111_0000);

	let mut expected = cpu.clone();
	expected.register_bank.write_single_named(ACC_REGISTER, 0b1111_0000);

	let operation = LoadByteOperation::new();

	let instruction = BaseByteInstruction::new(
		ByteSource::SingleRegister { single_reg: SingleRegisters::B },
		ByteDestination::Acc,
		operation,
	);

	assert!(instruction.execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}