use crate::bits::bits_to_byte;
use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::BitFlags;
use crate::instructions::base::double_byte::DoubleByteSource;
use crate::instructions::changeset::{Change, ChangeIme, ChangeList, ChangesetInstruction, MemoryDoubleByteWriteChange, PcChange, SpChange};
use crate::instructions::ExecutionError;
use crate::instructions::flow::BranchCondition;

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
			changes.push(Box::new(MemoryDoubleByteWriteChange::write_to_source(
				DoubleByteSource::StackPointer, old_pc,
			)));

			changes.push(Box::new(PcChange::new(
				self.address
			)))
		}

		Ok(ChangeList::new(changes))
	}
}