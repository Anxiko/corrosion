const WORKING_RAM_START: u16 = 0xC000;
const ECHO_RAM_START: u16 = 0xE000;
const OAM_START: u16 = 0xFE00;

const WORKING_RAM_SIZE: usize = (ECHO_RAM_START - WORKING_RAM_START) as usize;


pub(crate) trait Ram {
	fn read(&self, address: u16) -> Result<u8, RamError>;
	fn write(&mut self, address: u16, value: u8) -> Result<(), RamError>;
}

pub(crate) enum RamError {
	InvalidAddress(u16),
	UnmappedRegion(u16),
}

pub struct MappedRam {
	working_ram: WorkingRam,
}

#[derive(Copy, Clone)]
enum RamRegion {
	WorkingRam,
}

struct RamMapping {
	region: RamRegion,
	offset: u16,
	size: usize,

}

impl RamMapping {
	fn mapped_here(&self, address: u16) -> bool {
		usize::from(address - self.offset) < self.size
	}
}

const RAM_MAPPINGS: [RamMapping; 1] = [
	RamMapping {
		region: RamRegion::WorkingRam,
		offset: WORKING_RAM_START,
		size: WORKING_RAM_SIZE,
	},
];

impl MappedRam {
	pub(crate) fn new() -> Self {
		Self {
			working_ram: WorkingRam::new()
		}
	}

	fn mapping_for_address(address: u16) -> Option<&'static RamMapping> {
		RAM_MAPPINGS
			.iter()
			.find(|mapping| mapping.mapped_here(address))
	}

	fn get_mapped_ram(&self, region: RamRegion) -> &impl Ram {
		match region {
			RamRegion::WorkingRam => &self.working_ram,
		}
	}

	fn get_mapped_ram_mut(&mut self, region: RamRegion) -> &mut impl Ram {
		match region {
			RamRegion::WorkingRam => &mut self.working_ram
		}
	}
}

impl Ram for MappedRam {
	fn read(&self, address: u16) -> Result<u8, RamError> {
		let ram_mapping = MappedRam::mapping_for_address(address)
			.ok_or(RamError::UnmappedRegion(address))?;
		let mapped_ram = self.get_mapped_ram(ram_mapping.region);


		let region_address = address - ram_mapping.offset;
		mapped_ram.read(region_address)
	}

	fn write(&mut self, address: u16, value: u8) -> Result<(), RamError> {
		let ram_mapping = MappedRam::mapping_for_address(address)
			.ok_or(RamError::UnmappedRegion(address))?;
		let mapped_ram = self.get_mapped_ram_mut(ram_mapping.region);


		let region_address = address - ram_mapping.offset;
		mapped_ram.write(region_address, value)
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

impl Ram for WorkingRam {
	fn read(&self, address: u16) -> Result<u8, RamError> {
		self.memory
			.get(usize::from(address))
			.copied()
			.ok_or(RamError::InvalidAddress(address))
	}

	fn write(&mut self, address: u16, value: u8) -> Result<(), RamError> {
		let ptr = self.memory
			.get_mut(usize::from(address))
			.ok_or(RamError::InvalidAddress(address))?;

		*ptr = value;
		Ok(())
	}
}