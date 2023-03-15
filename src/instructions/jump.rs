use crate::bits::bits_to_byte;
use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::BitFlags;
use crate::instructions::base::DoubleByteSource;
use crate::instructions::changeset::{Change, ChangeIme, ChangeList, ChangesetInstruction, MemoryDoubleByteWriteChange, NoChange, PcChange, SpChange};
use crate::instructions::ExecutionError;

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum BranchCondition {
	Unconditional,
	TestFlag { flag: BitFlags, branch_if_equals: bool },
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum JumpInstructionDestination {
	FromSource(DoubleByteSource),
	RelativeToPc(i8),
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
			Self::FromSource(source) => {
				source.read(cpu)
			}
			Self::RelativeToPc(delta) => Ok(cpu.pc.read().wrapping_add_signed((*delta).into()))
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

pub(crate) struct CallInstruction {
	condition: BranchCondition,
	address: u16,
}

impl CallInstruction {
	pub(crate) fn new(condition: BranchCondition, address: u16) -> Self {
		Self { condition, address }
	}

	pub(crate) fn call(address: u16) -> Self {
		Self::new(BranchCondition::Unconditional, address)
	}

	pub(crate) fn call_conditional(flag: BitFlags, branch_if_equals: bool, address: u16) -> Self {
		Self::new(BranchCondition::TestFlag { flag, branch_if_equals }, address)
	}

	pub(crate) fn restart(bits: [bool; 3]) -> Self {
		let bits_as_byte: u16 = bits_to_byte(&bits).into();
		let address = 8u16 * bits_as_byte;
		Self::call(address)
	}
}

impl ChangesetInstruction for CallInstruction {
	type C = ChangeList;

	fn compute_change(&self, cpu: &Cpu) -> Result<Self::C, ExecutionError> {
		let mut changes: Vec<Box<dyn Change>> = Vec::new();

		if self.condition.satisfied(cpu) {
			let mut sp = cpu.sp.read();
			sp = sp.wrapping_add_signed(-2);
			changes.push(Box::new(SpChange::new(sp)));

			let old_pc = cpu.pc.read();
			changes.push(Box::new(MemoryDoubleByteWriteChange::new(
				DoubleByteSource::StackPointer, old_pc,
			)));

			changes.push(Box::new(PcChange::new(
				self.address
			)))
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
		JumpInstructionDestination::FromSource(DoubleByteSource::Immediate(0xABCD)),
		BranchCondition::Unconditional,
	);

	let expected: Box<dyn Change> = Box::new(PcChange::new(0xABCD));
	let actual = instruction.compute_change(&cpu).expect("Compute change");

	assert_eq!(actual, expected);

	let instruction = JumpInstruction::new(
		JumpInstructionDestination::FromSource(DoubleByteSource::AddressInRegister(DoubleRegisters::HL)),
		BranchCondition::TestFlag { flag: BitFlags::Zero, branch_if_equals: true },
	);

	let expected: Box<dyn Change> = Box::new(PcChange::new(0x5678));
	let actual = instruction.compute_change(&cpu).expect("Compute change");

	assert_eq!(actual, expected);

	let instruction = JumpInstruction::new(
		JumpInstructionDestination::RelativeToPc(-0x7F),
		BranchCondition::TestFlag { flag: BitFlags::Carry, branch_if_equals: false },
	);

	let expected: Box<dyn Change> = Box::new(PcChange::new(0x11B5));
	let actual = instruction.compute_change(&cpu).expect("Compute change");

	assert_eq!(actual, expected);

	let instruction = JumpInstruction::new(
		JumpInstructionDestination::RelativeToPc(-0x7F),
		BranchCondition::TestFlag { flag: BitFlags::Carry, branch_if_equals: true },
	);

	let expected: Box<dyn Change> = Box::new(NoChange::new());
	let actual = instruction.compute_change(&cpu).expect("Compute change");

	assert_eq!(actual, expected);
}