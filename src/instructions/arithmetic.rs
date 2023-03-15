use crate::hardware::cpu::Cpu;
use crate::hardware::register_bank::BitFlags;
use crate::instructions::{ACC_REGISTER, ExecutionError};
use crate::instructions::changeset::{BitFlagsChange, Change, ChangeList, ChangesetInstruction, SingleRegisterChange};

pub(crate) mod add_or_sub;
pub(crate) mod inc_or_dec;
pub(crate) mod compare;
pub(crate) mod bcd;