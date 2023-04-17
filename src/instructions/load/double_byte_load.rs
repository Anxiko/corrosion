use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Rom;
use crate::instructions::base::double_byte::{
	DoubleByteDestination, DoubleByteSource, UnaryDoubleByteInstruction, UnaryDoubleByteOperation,
};
use crate::instructions::changeset::{
	Change, ChangeList, ChangesetExecutable, MemoryDoubleByteWriteChange, SpChange,
};
use crate::instructions::ExecutionError;

#[derive(Debug)]
pub(crate) struct DoubleByteLoadOperation;

impl DoubleByteLoadOperation {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl UnaryDoubleByteOperation for DoubleByteLoadOperation {
	type C = Box<dyn Change>;

	fn execute(
		&self,
		cpu: &Cpu,
		src: &DoubleByteSource,
		dst: &DoubleByteDestination,
	) -> Result<Self::C, ExecutionError> {
		let value = src.read(cpu)?;
		Ok(dst.change_destination(value))
	}
}

pub(crate) type DoubleByteLoadInstruction = UnaryDoubleByteInstruction<DoubleByteLoadOperation>;

#[derive(Debug)]
pub(crate) struct PushInstruction {
	source: DoubleByteSource,
}

impl PushInstruction {
	pub(crate) fn new(source: DoubleByteSource) -> Self {
		Self { source }
	}
}

impl ChangesetExecutable for PushInstruction {
	type C = ChangeList;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let address = cpu.sp.read();
		let address = address.wrapping_sub(2);
		let value = self.source.read(cpu)?;

		Ok(ChangeList::new(vec![
			Box::new(SpChange::new(address)),
			Box::new(MemoryDoubleByteWriteChange::write_to_immediate(
				address, value,
			)),
		]))
	}
}

#[derive(Debug)]
pub(crate) struct PopInstruction {
	destination: DoubleByteDestination,
}

impl PopInstruction {
	pub(crate) fn new(destination: DoubleByteDestination) -> Self {
		Self { destination }
	}
}

impl ChangesetExecutable for PopInstruction {
	type C = ChangeList;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let address = cpu.sp.read();
		let value = cpu.mapped_ram.read_double_byte(address)?;
		let address = address.wrapping_add(2);

		Ok(ChangeList::new(vec![
			self.destination.change_destination(value),
			Box::new(SpChange::new(address)),
		]))
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::cpu::Cpu;
	use crate::hardware::ram::{Ram, WORKING_RAM_START};
	use crate::hardware::register_bank::DoubleRegisters;
	use crate::instructions::base::double_byte::{DoubleByteDestination, DoubleByteSource};
	use crate::instructions::changeset::{
		Change, ChangeList, ChangesetExecutable, DoubleRegisterChange,
		MemoryDoubleByteWriteChange, SpChange,
	};
	use crate::instructions::load::double_byte_load::{
		DoubleByteLoadInstruction, DoubleByteLoadOperation, PopInstruction, PushInstruction,
	};

	#[test]
	fn load() {
		let mut cpu = Cpu::new();
		cpu.sp.write(0x1234);

		let instruction = DoubleByteLoadInstruction::new(
			DoubleByteSource::StackPointer,
			DoubleByteDestination::AddressInImmediate(WORKING_RAM_START),
			DoubleByteLoadOperation::new(),
		);

		let expected: Box<dyn Change> = Box::new(MemoryDoubleByteWriteChange::write_to_immediate(
			WORKING_RAM_START,
			0x1234,
		));
		let actual = instruction.compute_change(&cpu).expect("Compute changes");

		assert_eq!(actual, expected);
	}

	#[test]
	fn push() {
		let mut cpu = Cpu::new();
		cpu.sp.write(WORKING_RAM_START + 2);
		cpu.register_bank
			.write_double_named(DoubleRegisters::BC, 0x1234);

		let instruction =
			PushInstruction::new(DoubleByteSource::DoubleRegister(DoubleRegisters::BC));

		let expected = ChangeList::new(vec![
			Box::new(SpChange::new(WORKING_RAM_START)),
			Box::new(MemoryDoubleByteWriteChange::write_to_immediate(
				WORKING_RAM_START,
				0x1234,
			)),
		]);
		let actual = instruction.compute_change(&cpu).expect("Compute changes");

		assert_eq!(actual, expected);
	}

	#[test]
	fn pop() {
		let mut cpu = Cpu::new();
		cpu.sp.write(WORKING_RAM_START);
		cpu.mapped_ram
			.write_double_byte(WORKING_RAM_START, 0x1234)
			.expect("Write to RAM");

		let instruction =
			PopInstruction::new(DoubleByteDestination::DoubleRegister(DoubleRegisters::BC));

		let expected = ChangeList::new(vec![
			Box::new(DoubleRegisterChange::new(DoubleRegisters::BC, 0x1234)),
			Box::new(SpChange::new(WORKING_RAM_START + 2)),
		]);
		let actual = instruction.compute_change(&cpu).expect("Compute changes");

		assert_eq!(actual, expected);
	}
}
