use operation::ArithmeticOperation;

use crate::hardware::cpu::Cpu;
use crate::hardware::ram::Ram;
use crate::hardware::register_bank::{DoubleRegisters, RegisterFlags, SingleRegisters};
use crate::instructions::{ExecutionError, Instruction};

#[cfg(test)]
mod tests;

mod operation;
mod add;

pub(crate) const ACC_REGISTER: SingleRegisters = SingleRegisters::A;