use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::{BitFlags, DoubleRegisters, SingleRegisters};
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::ACC_REGISTER;
use crate::instructions::arithmetic::operation::ArithmeticOperation;
use crate::instructions::base::{BinaryDoubleOperation, BinaryInstruction, BinaryOperation, ByteDestination, ByteSource, DoubleByteDestination, DoubleByteSource};
use crate::instructions::changeset::{ChangeList, ChangesetInstruction};

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum BinaryArithmeticOperationType {
	Add,
	Sub,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct BinaryArithmeticOperation {
	type_: BinaryArithmeticOperationType,
	with_carry: bool,
}

impl BinaryArithmeticOperation {
	pub(crate) fn new(type_: BinaryArithmeticOperationType, with_carry: bool) -> Self {
		Self{type_, with_carry}
	}
}

impl BinaryOperation for BinaryArithmeticOperation {
	type C = ChangeList;

	fn compute_changes(&self, cpu: &Cpu, left: &ByteSource, right: &ByteSource, dst: &ByteDestination) -> Result<Self::C, ExecutionError> {
		let carry = self.with_carry && cpu.register_bank.read_bit_flag(BitFlags::Carry);
		let left = left.read(cpu)?;
		let right = right.read(cpu)?;


		let operation = match self.type_ {
			BinaryArithmeticOperationType::Add => {
				ArithmeticOperation::add_with_carry(left, right, carry)
			},
			BinaryArithmeticOperationType::Sub => {
				ArithmeticOperation::sub_with_carry(left, right, carry)
			}
		};

		Ok(operation.change(dst))
	}
}

pub(crate) type BinaryArithmeticInstruction = BinaryInstruction<BinaryArithmeticOperation>;

pub(crate) struct Add {
	left: ByteSource,
	right: ByteSource,
	dst: ByteDestination,
}

impl Add {
	pub(crate) fn new(left: ByteSource, right: ByteSource, dst: ByteDestination) -> Self {
		Self { left, right, dst }
	}
}

impl ChangesetInstruction for Add {
	type C = ChangeList;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let dst_val = self.left.read(cpu)?;
		let src_val = self.right.read(cpu)?;

		Ok(ArithmeticOperation::add(dst_val, src_val).change(&self.dst))
	}
}

pub(crate) struct AddImmediate {}

impl AddImmediate {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl Instruction for AddImmediate {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let src_address = cpu.next_pc();
		let src_val = cpu.mapped_ram.read_byte(src_address)?;
		let dst_val = cpu.register_bank.read_single_named(ACC_REGISTER);

		ArithmeticOperation::add(dst_val, src_val).commit(cpu);

		Ok(())
	}
}

pub(crate) struct AddWithCarry {
	src: SingleRegisters,
}

impl AddWithCarry {
	pub(super) fn new(src: SingleRegisters) -> Self {
		Self { src }
	}
}

impl Instruction for AddWithCarry {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let src_val = cpu.register_bank.read_single_named(self.src);
		let carry_bit = cpu.register_bank.read_bit_flag(BitFlags::Carry);
		let dst_val = cpu.register_bank.read_single_named(ACC_REGISTER);

		ArithmeticOperation::add_with_carry(dst_val, src_val, carry_bit).commit(cpu);

		Ok(())
	}
}

pub struct AddHl;

impl AddHl {
	pub(super) fn new() -> Self {
		Self {}
	}
}

impl Instruction for AddHl {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let src_address = cpu.register_bank.read_double_named(DoubleRegisters::HL);
		let src_val = cpu.mapped_ram.read_byte(src_address)?;
		let dst_val = cpu.register_bank.read_single_named(ACC_REGISTER);

		ArithmeticOperation::add(dst_val, src_val).commit(cpu);

		Ok(())
	}
}

pub(crate) struct Increment {}

impl Increment {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl Instruction for Increment {
	fn execute(&self, cpu: &mut Cpu) -> Result<(), ExecutionError> {
		let dst_val = cpu.register_bank.read_single_named(ACC_REGISTER);

		ArithmeticOperation::add(dst_val, 1).commit(cpu);

		Ok(())
	}
}

struct AddDoubleInstruction;