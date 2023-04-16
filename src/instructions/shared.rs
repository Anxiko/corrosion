#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IndexUpdateType {
	Increment,
	Decrement,
}

impl IndexUpdateType {
	pub(crate) fn to_delta(self) -> i8 {
		match self {
			Self::Increment => 1,
			Self::Decrement => -1
		}
	}
}