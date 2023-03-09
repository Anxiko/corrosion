use crate::hardware::cpu::Cpu;
use crate::hardware::ram::{Ram, WORKING_RAM_START};
use crate::hardware::register_bank::{BitFlags, DoubleRegisters};
use crate::instructions::changeset::{Change, ChangesetInstruction, NoChange, PcChange};
use crate::instructions::ExecutionError;

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum JumpInstructionDestination {
	Immediate(u16),
	AddressOnHl,
	Relative(i8),
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

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum JumpInstructionCondition {
	Unconditional,
	TestFlag { flag: BitFlags, branch_if_equals: bool },
}

impl JumpInstructionCondition {
	fn satisfied(&self, cpu: &Cpu) -> bool {
		match self {
			Self::Unconditional => true,
			Self::TestFlag { flag, branch_if_equals } => {
				cpu.register_bank.read_bit_flag(*flag) == *branch_if_equals
			}
		}
	}
}

pub(crate) struct JumpInstruction {
	dst: JumpInstructionDestination,
	condition: JumpInstructionCondition,
}

impl JumpInstruction {
	pub(crate) fn new(dst: JumpInstructionDestination, condition: JumpInstructionCondition) -> Self {
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
		JumpInstructionCondition::Unconditional,
	);

	let expected: Box<dyn Change> = Box::new(PcChange::new(0xABCD));
	let actual = instruction.compute_change(&cpu).expect("Compute change");

	assert_eq!(actual, expected);

	let instruction = JumpInstruction::new(
		JumpInstructionDestination::AddressOnHl,
		JumpInstructionCondition::TestFlag { flag: BitFlags::Zero, branch_if_equals: true },
	);

	let expected: Box<dyn Change> = Box::new(PcChange::new(0x5678));
	let actual = instruction.compute_change(&cpu).expect("Compute change");

	assert_eq!(actual, expected);

	let instruction = JumpInstruction::new(
		JumpInstructionDestination::Relative(-0x7F),
		JumpInstructionCondition::TestFlag { flag: BitFlags::Carry, branch_if_equals: false },
	);

	let expected: Box<dyn Change> = Box::new(PcChange::new(0x11B5));
	let actual = instruction.compute_change(&cpu).expect("Compute change");

	assert_eq!(actual, expected);

	let instruction = JumpInstruction::new(
		JumpInstructionDestination::Relative(-0x7F),
		JumpInstructionCondition::TestFlag { flag: BitFlags::Carry, branch_if_equals: true },
	);

	let expected: Box<dyn Change> = Box::new(NoChange::new());
	let actual = instruction.compute_change(&cpu).expect("Compute change");

	assert_eq!(actual, expected);
}