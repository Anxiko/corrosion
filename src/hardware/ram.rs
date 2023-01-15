const WORKING_RAM_START: u16 = 0xC000;
const ECHO_RAM_START: u16 = 0xE000;

const WORKING_RAM_SIZE: usize = (ECHO_RAM_START - WORKING_RAM_START) as usize;


trait Ram {
	fn read(&self, address: u16) -> Result<u8, RamError>;
	fn write(&mut self, address: u16, value: u8) -> Result<(), RamError>;
}

enum RamError {}

pub struct MappedRam {
	working_ram: WorkingRam,
}

impl MappedRam {
	pub(crate) fn new() -> Self {
		Self {
			working_ram: WorkingRam::new()
		}
	}
}

impl Ram for MappedRam {
	fn read(&self, address: u16) -> Result<u8, RamError> {
		todo!()
	}

	fn write(&mut self, address: u16, value: u8) -> Result<(), RamError> {
		todo!()
	}
}

struct WorkingRam {
	memory: Box<[u8; WORKING_RAM_SIZE]>,
}

impl WorkingRam {
	fn new() -> Self {
		Self {
			memory: Box::new([0; WORKING_RAM_SIZE])
		}
	}
}