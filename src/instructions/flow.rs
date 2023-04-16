pub(crate) use branch::{JumpInstruction, JumpInstructionDestination};
pub(crate) use call::CallInstruction;
pub(crate) use condition::BranchCondition;
pub(crate) use return_::ReturnInstruction;

pub(crate) mod branch;
pub(crate) mod condition;
pub(crate) mod return_;
pub(crate) mod call;