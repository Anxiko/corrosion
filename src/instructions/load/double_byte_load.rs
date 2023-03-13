use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::DoubleRegisters;
use crate::instructions::base::{BaseDoubleByteInstruction, DoubleByteDestination, DoubleByteOperation, DoubleByteSource};
use crate::instructions::changeset::{Change, ChangeList, ChangesetInstruction, DoubleRegisterChange, MemoryDoubleByteWriteChange, SpChange};
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

struct PushInstruction {
	source: DoubleRegisters,
}

impl ChangesetInstruction for PushInstruction {
	type C = ChangeList;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let address = cpu.sp.read();
		let address = address.wrapping_sub(2);
		let value = cpu.register_bank.read_double_named(self.source);

		Ok(ChangeList::new(vec![
			Box::new(SpChange::new(address)),
			Box::new(MemoryDoubleByteWriteChange::write_to_immediate_address(address, value)),
		]))
	}
}

struct PopInstruction {
	destination: DoubleRegisters,
}

impl ChangesetInstruction for PopInstruction {
	type C = ChangeList;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let address = cpu.sp.read();
		let value = cpu.mapped_ram.read_double_byte(address)?;
		let address = address.wrapping_add(2);

		Ok(ChangeList::new(vec![
			Box::new(DoubleRegisterChange::new(self.destination, value)),
			Box::new(SpChange::new(address)),
		]))
	}
}