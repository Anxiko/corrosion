use crate::hardware::alu::delta_u8;
use crate::hardware::cpu::Cpu;
use crate::instructions::base::{BaseByteInstruction, ByteDestination, ByteOperation, ByteSource};
use crate::instructions::changeset::{BitFlagsChange, Change, ChangeList};
use crate::instructions::ExecutionError;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IncOrDecOperationType {
	Increment,
	Decrement,
}

impl IncOrDecOperationType {
	fn delta(&self) -> i8 {
		match self {
			Self::Increment => 1,
			Self::Decrement => -1
		}
	}

	fn is_sub(&self) -> bool {
		match self {
			Self::Increment => false,
			Self::Decrement => true
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct IncOrDecOperation {
	type_: IncOrDecOperationType,
}

impl IncOrDecOperation {
	pub fn new(type_: IncOrDecOperationType) -> Self {
		Self { type_ }
	}
}

impl ByteOperation for IncOrDecOperation {
	type C = Box<dyn Change>;

	fn execute(&self, cpu: &Cpu, src: &ByteSource, dst: &ByteDestination) -> Result<Self::C, ExecutionError> {
		let value = src.read(cpu)?;
		let delta = self.type_.delta();

		let alu_result = delta_u8(value, delta);
		let result = alu_result.result;
		let bitflags_change = BitFlagsChange::from(alu_result).keep_carry_flag();

		Ok(Box::new(ChangeList::new(vec![
			dst.change_destination(result),
			Box::new(bitflags_change),
		])))
	}
}

pub(crate) type IncOrDecInstruction = BaseByteInstruction<IncOrDecOperation>;
