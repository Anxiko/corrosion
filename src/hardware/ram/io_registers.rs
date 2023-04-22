use crate::hardware::counters::divider::DividerRegister;
use crate::hardware::ram::{Ram, RamChip, RamError, Rom};
use sdl2::sys::off_t;

use super::memory_mapping::{MemoryMapping, MemoryMappingEntry, RegionToMemoryMapper};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum IoRegistersMemoryMappingRegion {
	JoypadInput,
	SerialTransfer,
	DividerRegister,
}

const IO_REGISTER_MAPPING_SIZE: usize = 3;
const IO_REGISTER_MAPPING_ENTRIES: [MemoryMappingEntry<IoRegistersMemoryMappingRegion>; IO_REGISTER_MAPPING_SIZE] = [
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::JoypadInput, 0, 1),
	MemoryMappingEntry::new(
		IoRegistersMemoryMappingRegion::SerialTransfer,
		1,
		IO_REGISTER_SERIAL_TRANSFER_SIZE,
	),
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::DividerRegister, 4, 1),
];

const IO_REGISTER_SERIAL_TRANSFER_SIZE: usize = 2;

struct IoRegistersMemoryMapping {
	mapping: MemoryMapping<1, IoRegistersMemoryMappingRegion>,
	joypad_input: u8,
	serial_transfer: RamChip<IO_REGISTER_SERIAL_TRANSFER_SIZE>,
	divider_register: DividerRegister,
}

impl RegionToMemoryMapper for IoRegistersMemoryMapping {
	type R = IoRegistersMemoryMappingRegion;

	fn matching_entry(&self, address: u16) -> Result<&MemoryMappingEntry<Self::R>, RamError> {
		self.mapping.find_mapping(address)
	}

	fn get_rom(&self, region: Self::R) -> Result<&dyn Rom, RamError> {
		match region {
			IoRegistersMemoryMappingRegion::JoypadInput => Ok(&self.joypad_input),
			IoRegistersMemoryMappingRegion::SerialTransfer => Ok(&self.serial_transfer),
			IoRegistersMemoryMappingRegion::DividerRegister => Ok(&self.divider_register),
		}
	}

	fn get_ram(&self, region: Self::R) -> Result<&dyn Ram, RamError> {
		match region {
			IoRegistersMemoryMappingRegion::JoypadInput => Ok(&self.joypad_input),
			IoRegistersMemoryMappingRegion::SerialTransfer => Ok(&self.serial_transfer),
			IoRegistersMemoryMappingRegion::DividerRegister => Ok(&self.serial_transfer),
		}
	}
}
