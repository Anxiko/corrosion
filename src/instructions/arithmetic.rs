use crate::hardware::register_bank::SingleRegisters;

pub(crate) const ACC_REGISTER: SingleRegisters = SingleRegisters::A;

#[cfg(test)]
mod tests;

mod operation;
mod add;
mod sub;
