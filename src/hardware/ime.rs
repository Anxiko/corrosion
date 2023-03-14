#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Ime {
	interruptions_enabled: bool,
}


impl Ime {
	pub(crate) fn new() -> Self {
		Self { interruptions_enabled: true }
	}

	pub(crate) fn read(&self) -> bool {
		self.interruptions_enabled
	}

	pub(crate) fn write(&mut self, interruptions_enabled: bool) {
		self.interruptions_enabled = interruptions_enabled;
	}
}