use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::{DoubleRegisters, SingleRegisters};
use super::{ACC_REGISTER, ExecutionError};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum InstructionByteSource {
	Acc,
	SingleRegister { single_reg: SingleRegisters },
	Memory { address_register: DoubleRegisters },
}

impl InstructionByteSource {
	fn read_from_acc() -> Self {
		Self::Acc
	}

	fn read_from_single(single_reg: SingleRegisters) -> Self {
		Self::SingleRegister { single_reg }
	}

	fn read_from_hl_address() -> Self {
		Self::Memory { address_register: DoubleRegisters::HL }
	}

	fn read(&self, cpu: &Cpu) -> Result<u8, ExecutionError> {
		match self {
			Self::Acc => Ok(cpu.register_bank.read_single_named(ACC_REGISTER)),
			Self::SingleRegister { single_reg } => Ok(cpu.register_bank.read_single_named(*single_reg)),
			Self::Memory { address_register } => {
				let address = cpu.register_bank.read_double_named(*address_register);
				let result = cpu.mapped_ram.read(address)?;
				Ok(result)
			}
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum InstructionByteDestination {
	Acc,
	SingleRegister { single_reg: SingleRegisters },
}

impl InstructionByteDestination {
	fn write_to_acc() -> Self {
		Self::Acc
	}

	fn write_to_single(single_reg: SingleRegisters) -> Self {
		Self::SingleRegister { single_reg }
	}

	fn write(&self, cpu: &mut Cpu, value: u8) {
		let reg = match self {
			Self::Acc => ACC_REGISTER,
			Self::SingleRegister { single_reg } => *single_reg
		};

		cpu.register_bank.write_single_named(reg, value);
	}
}

struct BaseByteInstruction {

}