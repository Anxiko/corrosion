use std::fmt::{Display, Formatter, LowerHex, UpperHex};

use crate::hardware::cpu::Cpu;
use crate::instructions::base::double_byte::DoubleByteSource;
use crate::instructions::changeset::{Change, ChangesetExecutable, NoChange, PcChange};
use crate::instructions::flow::BranchCondition;
use crate::instructions::ExecutionError;

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum JumpInstructionDestination {
	FromSource(DoubleByteSource),
	RelativeToPc(i8),
}

impl JumpInstructionDestination {
	fn resolve(&self, cpu: &Cpu) -> Result<u16, ExecutionError> {
		match self {
			Self::FromSource(source) => source.read(cpu),
			Self::RelativeToPc(delta) => Ok(cpu.pc.read().wrapping_add_signed((*delta).into())),
		}
	}

	fn is_relative(&self) -> bool {
		match self {
			Self::FromSource(_) => false,
			Self::RelativeToPc(_) => true,
		}
	}
}

struct SignedI8 {
	value: i8,
}

impl SignedI8 {
	fn signed_hex_format(&self, f: &mut Formatter<'_>, use_upper: bool) -> std::fmt::Result {
		if self.value < 0 {
			write!(f, "-")?;
		} else {
			write!(f, "+")?;
		}

		if use_upper {
			write!(f, "{:#X}", self.value.abs())
		} else {
			write!(f, "{:#x}", self.value.abs())
		}
	}
}

impl LowerHex for SignedI8 {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.signed_hex_format(f, false)
	}
}

impl UpperHex for SignedI8 {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.signed_hex_format(f, true)
	}
}

impl Display for JumpInstructionDestination {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::FromSource(s) => write!(f, "{s}"),
			Self::RelativeToPc(r) => {
				let signed_i8 = SignedI8 { value: *r };
				write!(f, "PC{signed_i8:#04X}")
			}
		}
	}
}

#[derive(Debug)]
pub(crate) struct JumpInstruction {
	dst: JumpInstructionDestination,
	condition: BranchCondition,
}

impl JumpInstruction {
	pub(crate) fn new(dst: JumpInstructionDestination, condition: BranchCondition) -> Self {
		Self { dst, condition }
	}
}

impl ChangesetExecutable for JumpInstruction {
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

impl Display for JumpInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		if self.dst.is_relative() {
			write!(f, "jr")?;
		} else {
			write!(f, "jp")?;
		}

		if let Some(condition) = self.condition.as_maybe_string() {
			write!(f, " {condition}, ")?;
		}

		write!(f, "{}", self.dst)?;

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::hardware::ram::{Ram, WORKING_RAM_START};
	use crate::hardware::register_bank::{BitFlags, DoubleRegisters};

	use super::*;

	fn get_cpu() -> Cpu {
		let mut cpu = Cpu::new();
		cpu.pc.write(0x1234);
		cpu.register_bank
			.write_double_named(DoubleRegisters::HL, WORKING_RAM_START);
		cpu.mapped_ram
			.write_double_byte(WORKING_RAM_START, 0x5678)
			.expect("Write to RAM");
		cpu.register_bank.write_bit_flag(BitFlags::Zero, true);
		cpu
	}

	#[test]
	fn unconditional_jump_to_immediate() {
		let cpu = get_cpu();

		let instruction = JumpInstruction::new(
			JumpInstructionDestination::FromSource(DoubleByteSource::Immediate(0xABCD)),
			BranchCondition::Unconditional,
		);

		let expected: Box<dyn Change> = Box::new(PcChange::new(0xABCD));
		let actual = instruction.compute_change(&cpu).expect("Compute change");

		assert_eq!(actual, expected);
	}

	#[test]
	fn flag_test_relative_jump() {
		let cpu = get_cpu();

		let instruction = JumpInstruction::new(
			JumpInstructionDestination::RelativeToPc(-0x7F),
			BranchCondition::TestFlag {
				flag: BitFlags::Carry,
				branch_if_equals: false,
			},
		);

		let expected: Box<dyn Change> = Box::new(PcChange::new(0x11B5));
		let actual = instruction.compute_change(&cpu).expect("Compute change");

		assert_eq!(actual, expected);
	}

	#[test]
	fn flag_test_relative_jump_failed() {
		let cpu = get_cpu();

		let instruction = JumpInstruction::new(
			JumpInstructionDestination::RelativeToPc(-0x7F),
			BranchCondition::TestFlag {
				flag: BitFlags::Carry,
				branch_if_equals: true,
			},
		);

		let expected: Box<dyn Change> = Box::new(NoChange::new());
		let actual = instruction.compute_change(&cpu).expect("Compute change");

		assert_eq!(actual, expected);
	}
}
