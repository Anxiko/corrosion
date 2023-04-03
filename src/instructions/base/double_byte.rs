use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::DoubleRegisters;
use crate::instructions::changeset::{Change, ChangesetInstruction, DoubleRegisterChange, MemoryDoubleByteWriteChange, SpChange};
use crate::instructions::ExecutionError;

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum DoubleByteSource {
	DoubleRegister(DoubleRegisters),
	Immediate(u16),
	StackPointer,
	AddressInRegister(DoubleRegisters),
}

impl DoubleByteSource {
	fn read_from_double_register(double_register: DoubleRegisters) -> Self {
		Self::DoubleRegister(double_register)
	}

	fn read_from_immediate(immediate: u16) -> Self {
		Self::Immediate(immediate)
	}

	fn read_from_sp() -> Self {
		Self::StackPointer
	}

	pub(in crate::instructions) fn read(&self, cpu: &Cpu) -> Result<u16, ExecutionError> {
		match self {
			Self::DoubleRegister(double_register) => {
				Ok(cpu.register_bank.read_double_named(*double_register))
			}
			Self::Immediate(immediate) => {
				Ok(*immediate)
			}
			Self::StackPointer => {
				Ok(cpu.sp.read())
			}
			Self::AddressInRegister(double_register) => {
				let address = cpu.register_bank.read_double_named(*double_register);
				let value = cpu.mapped_ram.read_double_byte(address)?;
				Ok(value)
			}
		}
	}
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum DoubleByteDestination {
	DoubleRegister(DoubleRegisters),
	StackPointer,
	AddressInImmediate(u16),
	AddressInRegister(DoubleRegisters),
}

impl DoubleByteDestination {
	pub(crate) fn change_destination(&self, value: u16) -> Box<dyn Change> {
		match self {
			Self::DoubleRegister(double_register) => {
				Box::new(DoubleRegisterChange::new(*double_register, value))
			}
			Self::StackPointer => {
				Box::new(SpChange::new(value))
			}
			Self::AddressInImmediate(address) => {
				Box::new(MemoryDoubleByteWriteChange::write_to_immediate_address(*address, value))
			}
			_ => todo!()
		}
	}
}

pub(crate) trait UnaryDoubleByteOperation {
	type C: Change;

	fn execute(
		&self,
		cpu: &Cpu,
		src: &DoubleByteSource,
		dst: &DoubleByteDestination,
	) -> Result<Self::C, ExecutionError>;
}

pub(crate) struct UnaryDoubleByteInstruction<O>
	where O: UnaryDoubleByteOperation {
	src: DoubleByteSource,
	dst: DoubleByteDestination,
	op: O,
}

impl<O> UnaryDoubleByteInstruction<O> where O: UnaryDoubleByteOperation {
	pub(crate) fn new(src: DoubleByteSource, dst: DoubleByteDestination, op: O) -> Self {
		Self { src, dst, op }
	}
}

impl<O> ChangesetInstruction for UnaryDoubleByteInstruction<O>
	where O: UnaryDoubleByteOperation {
	type C = O::C;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		self.op.execute(cpu, &self.src, &self.dst)
	}
}

pub(crate) trait BinaryDoubleByteOperation {
	type C: Change;

	fn compute_changes(
		&self, cpu: &Cpu, left: &DoubleByteSource, right: &DoubleByteSource, dst: &DoubleByteDestination,
	) -> Result<Self::C, ExecutionError>;
}
