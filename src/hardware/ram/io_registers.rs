use crate::hardware::ram::{Ram, RamError, Rom};

use super::memory_mapping::{MemoryMapping, MemoryMappingEntry, RegionToMemoryMapper};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum IoRegistersMemoryMappingRegion {
	JoypadInput,
}

const IO_REGISTER_MAPPING_SIZE: usize = 1;
const IO_REGISTER_MAPPING_ENTRIES: [MemoryMappingEntry<IoRegistersMemoryMappingRegion>;
	IO_REGISTER_MAPPING_SIZE] = [MemoryMappingEntry::new(
	IoRegistersMemoryMappingRegion::JoypadInput,
	0,
	1,
)];

struct IoRegistersMemoryMapping {
	mapping: MemoryMapping<1, IoRegistersMemoryMappingRegion>,
	joypad_input: u8,
}

impl RegionToMemoryMapper for IoRegistersMemoryMapping {
	type R = IoRegistersMemoryMappingRegion;

	fn matching_entry(&self, address: u16) -> Result<&MemoryMappingEntry<Self::R>, RamError> {
		self.mapping.find_mapping(address)
	}

	fn get_rom(&self, region: Self::R) -> Result<Box<dyn Rom>, RamError> {
		match region {
			IoRegistersMemoryMappingRegion::JoypadInput => Ok(Box::new(self.joypad_input)),
		}
	}

	fn get_ram(&self, region: Self::R) -> Result<Box<dyn Ram>, RamError> {
		match region {
			IoRegistersMemoryMappingRegion::JoypadInput => Ok(Box::new(self.joypad_input)),
		}
	}
}
