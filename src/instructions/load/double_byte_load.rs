use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::instructions::base::{BaseDoubleByteInstruction, DoubleByteDestination, DoubleByteOperation, DoubleByteSource};
use crate::instructions::changeset::{Change, ChangeList, ChangesetInstruction, MemoryDoubleByteWriteChange, SpChange};
use crate::instructions::ExecutionError;

pub(crate) struct DoubleByteLoadOperation;

impl DoubleByteLoadOperation {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl DoubleByteOperation for DoubleByteLoadOperation {
	type C = Box<dyn Change>;

	fn execute(&self, cpu: &Cpu, src: &DoubleByteSource, dst: &DoubleByteDestination) -> Result<Self::C, ExecutionError> {
		let value = src.read(cpu)?;
		Ok(dst.change_destination(value))
	}
}

pub(crate) type DoubleByteLoadInstruction = BaseDoubleByteInstruction<DoubleByteLoadOperation>;

pub(crate) struct PushInstruction {
	source: DoubleByteSource,
}

impl PushInstruction {
	pub(crate) fn new(source: DoubleByteSource) -> Self {
		Self{source}
	}
}

impl ChangesetInstruction for PushInstruction {
	type C = ChangeList;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let address = cpu.sp.read();
		let address = address.wrapping_sub(2);
		let value = self.source.read(cpu)?;

		Ok(ChangeList::new(vec![
			Box::new(SpChange::new(address)),
			Box::new(MemoryDoubleByteWriteChange::write_to_immediate_address(address, value)),
		]))
	}
}

pub(crate) struct PopInstruction {
	destination: DoubleByteDestination,
}

impl PopInstruction {
	pub(crate) fn new(destination: DoubleByteDestination) -> Self {
		Self { destination }
	}
}

impl ChangesetInstruction for PopInstruction {
	type C = ChangeList;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let address = cpu.sp.read();
		let value = cpu.mapped_ram.read_double_byte(address)?;
		let address = address.wrapping_add(2);

		Ok(ChangeList::new(vec![
			Box::new(self.destination.change_destination(value)),
			Box::new(SpChange::new(address)),
		]))
	}
}