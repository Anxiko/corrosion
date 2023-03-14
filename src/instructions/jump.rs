use crate::hardware::cpu::Cpu;
use crate::hardware::ram::{Ram, WORKING_RAM_START};
use crate::hardware::register_bank::{BitFlags, DoubleRegisters};
use crate::instructions::{ExecutionError, Instruction};
use crate::instructions::changeset::{Change, ChangeIme, ChangeList, ChangesetInstruction, NoChange, PcChange, SpChange};

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum BranchCondition {
	Unconditional,
	TestFlag { flag: BitFlags, branch_if_equals: bool },
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum JumpInstructionDestination {
	Immediate(u16),
	AddressOnHl,
	Relative(i8),
}

impl BranchCondition {
	fn satisfied(&self, cpu: &Cpu) -> bool {
		match self {
			Self::Unconditional => true,
			Self::TestFlag { flag, branch_if_equals } => {
				cpu.register_bank.read_bit_flag(*flag) == *branch_if_equals
			}
		}
	}
}

impl JumpInstructionDestination {
	fn resolve(&self, cpu: &Cpu) -> Result<u16, ExecutionError> {
		match self {
			Self::Immediate(address) => Ok(*address),
			Self::AddressOnHl => {
				let address = cpu.register_bank.read_double_named(DoubleRegisters::HL);
				let destination = cpu.mapped_ram.read_double_byte(address)?;
				Ok(destination)
			},
			Self::Relative(delta) => Ok(cpu.pc.read().wrapping_add_signed((*delta).into()))
		}
	}
}



pub(crate) struct JumpInstruction {
	dst: JumpInstructionDestination,
	condition: BranchCondition,
}

impl JumpInstruction {
	pub(crate) fn new(dst: JumpInstructionDestination, condition: BranchCondition) -> Self {
		Self { dst, condition }
	}
}

impl ChangesetInstruction for JumpInstruction {
	type C = Box<dyn Change>;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		if self.condition.satisfied(cpu) {
			let destination = self.dst.resolve(cpu)?;
			Ok(Box::new(PcChange::new(destination)))
		} else {
			Ok(Box::new(NoChange::new()))
		}
	}
}

pub(crate) struct ReturnInstruction {
	condition: BranchCondition,
	enable_interrupts: bool,
}

impl ReturnInstruction {
	pub(crate) fn new(condition: BranchCondition, enable_interrupts: bool) -> Self {
		Self { condition, enable_interrupts }
	}

	pub(crate) fn ret() -> Self {
		Self::new(BranchCondition::Unconditional, false)
	}

	pub(crate) fn ret_conditional(flag: BitFlags, value: bool) -> Self {
		Self::new(BranchCondition::TestFlag { flag, branch_if_equals: value }, false)
	}

	pub(crate) fn ret_and_enable_interrupts() -> Self {
		Self::new(BranchCondition::Unconditional, true)
	}
}

impl ChangesetInstruction for ReturnInstruction {
	type C = ChangeList;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let mut changes: Vec<Box<dyn Change>> = Vec::new();
		if self.condition.satisfied(cpu) {
			let sp_value = cpu.sp.read();
			let address = cpu.mapped_ram.read_double_byte(sp_value)?;

			changes.push(Box::new(PcChange::new(address)));
			changes.push(Box::new(SpChange::new(sp_value + 2)));

			if self.enable_interrupts {
				changes.push(Box::new(ChangeIme::new(true)));
			}
		}

		Ok(ChangeList::new(changes))
	}
}

#[test]
fn jump() {
	let mut cpu = Cpu::new();
	cpu.pc.write(0x1234);
	cpu.register_bank.write_double_named(DoubleRegisters::HL, WORKING_RAM_START);
	cpu.mapped_ram.write_double_byte(WORKING_RAM_START, 0x5678).expect("Write to RAM");
	cpu.register_bank.write_bit_flag(BitFlags::Zero, true);
	let cpu = cpu;

	let instruction = JumpInstruction::new(
		JumpInstructionDestination::Immediate(0xABCD),
		BranchCondition::Unconditional,
	);

	let expected: Box<dyn Change> = Box::new(PcChange::new(0xABCD));
	let actual = instruction.compute_change(&cpu).expect("Compute change");

	assert_eq!(actual, expected);

	let instruction = JumpInstruction::new(
		JumpInstructionDestination::AddressOnHl,
		BranchCondition::TestFlag { flag: BitFlags::Zero, branch_if_equals: true },
	);

	let expected: Box<dyn Change> = Box::new(PcChange::new(0x5678));
	let actual = instruction.compute_change(&cpu).expect("Compute change");

	assert_eq!(actual, expected);

	let instruction = JumpInstruction::new(
		JumpInstructionDestination::Relative(-0x7F),
		BranchCondition::TestFlag { flag: BitFlags::Carry, branch_if_equals: false },
	);

	let expected: Box<dyn Change> = Box::new(PcChange::new(0x11B5));
	let actual = instruction.compute_change(&cpu).expect("Compute change");

	assert_eq!(actual, expected);

	let instruction = JumpInstruction::new(
		JumpInstructionDestination::Relative(-0x7F),
		BranchCondition::TestFlag { flag: BitFlags::Carry, branch_if_equals: true },
	);

	let expected: Box<dyn Change> = Box::new(NoChange::new());
	let actual = instruction.compute_change(&cpu).expect("Compute change");

	assert_eq!(actual, expected);
}