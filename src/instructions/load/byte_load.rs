use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::DoubleRegisters;
use crate::instructions::{ACC_REGISTER, ExecutionError};
use crate::instructions::base::{BaseByteInstruction, ByteDestination, ByteOperation, ByteSource};
use crate::instructions::changeset::{Change, ChangeList, ChangesetInstruction, DoubleRegisterChange, MemoryByteWriteChange, SingleRegisterChange};

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum ByteLoadIndex {
	Sp,
	DoubleRegister(DoubleRegisters),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum ByteLoadUpdateType {
	Increment,
	Decrement,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) struct ByteLoadUpdate {
	index: ByteLoadIndex,
	type_: ByteLoadUpdateType,
}

impl ByteLoadUpdate {
	pub(crate) fn new(index: ByteLoadIndex, type_: ByteLoadUpdateType) -> Self {
		Self {
			index,
			type_,
		}
	}
}

pub(crate) struct ByteLoadOperation {
	update: Option<ByteLoadUpdate>,
}

impl ByteLoadOperation {
	pub(crate) fn new(update: Option<ByteLoadUpdate>) -> Self { Self { update } }

	pub(crate) fn no_update() -> Self {
		Self::new(None)
	}

	pub(crate) fn with_update(update: ByteLoadUpdate) -> Self {
		Self { update: Some(update) }
	}
}

impl ByteOperation for ByteLoadOperation {
	type C = Box<dyn Change>;

	fn execute(&self, cpu: &Cpu, src: &ByteSource, dst: &ByteDestination) -> Result<Self::C, ExecutionError> {
		let value = src.read(cpu)?;
		Ok(dst.change_destination(value))
	}
}

pub(crate) type ByteLoadInstruction = BaseByteInstruction<ByteLoadOperation>;

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug, PartialEq)]
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
				let value = cpu.mapped_ram.read_byte(address)?;
				Ok(value)
			},
			LoadAndUpdateRegister::Destination(_) => {
				Ok(cpu.register_bank.read_single_named(ACC_REGISTER))
			}
		}
	}

	fn write(&self, value: u8) -> Box<dyn Change> {
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
		let write_change = self.write(value);

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
	let result = ByteLoadOperation::no_update().execute(
		&cpu, &ByteSource::SingleRegister(SingleRegisters::B), &ByteDestination::Acc,
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

	let operation = ByteLoadOperation::no_update();

	let instruction = BaseByteInstruction::new(
		ByteSource::SingleRegister(SingleRegisters::B),
		ByteDestination::Acc,
		operation,
	);

	assert!(instruction.execute(&mut cpu).is_ok());
	assert_eq!(cpu, expected);
}

#[test]
fn load_and_update_instruction() {
	let address = WORKING_RAM_START + 10;
	let register = DoubleRegisters::HL;
	let value = 0x12u8;

	let mut cpu = Cpu::new();

	cpu.register_bank.write_double_named(register, address);
	cpu.mapped_ram.write_byte(address, value).expect("Write to mapped RAM");


	let instruction = LoadAndUpdateInstruction::load_from_register(
		register, CounterUpdate::Increment,
	);

	let result = instruction.compute_change(&cpu).expect("Instruction to execute");
	assert_eq!(
		result,
		ChangeList::new(vec![
			Box::new(SingleRegisterChange::new(ACC_REGISTER, value)),
			Box::new(DoubleRegisterChange::new(register, address + 1)),
		])
	);

	let mut cpu = Cpu::new();

	cpu.register_bank.write_double_named(register, address);
	cpu.register_bank.write_single_named(ACC_REGISTER, value);


	let instruction = LoadAndUpdateInstruction::load_to_register(
		register, CounterUpdate::Decrement,
	);

	let result = instruction.compute_change(&cpu).expect("Instruction to execute");
	assert_eq!(
		result,
		ChangeList::new(vec![
			Box::new(MemoryByteWriteChange::write_to_register(register, value)),
			Box::new(DoubleRegisterChange::new(register, address - 1)),
		])
	);
}