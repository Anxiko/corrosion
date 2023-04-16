pub(crate) use branch::{JumpInstruction, JumpInstructionDestination};
pub(crate) use condition::BranchCondition;
pub(crate) use function::{CallInstruction, ReturnInstruction};

pub(crate) mod branch;
pub(crate) mod condition;
pub(crate) mod function;