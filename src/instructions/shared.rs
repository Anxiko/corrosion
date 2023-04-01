#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IndexUpdateType {
	Increment,
	Decrement,
}

impl IndexUpdateType {
	pub(crate) fn to_delta(&self) -> i8 {
		match self {
			Self::Increment => 1,
			Self::Decrement => -1
		}
	}

	pub(crate) fn is_sub(&self) -> bool {
		self.to_delta() < 0
	}
}