use crate::hardware::audio::Audio;
use crate::hardware::counters::divider::DividerRegister;
use crate::hardware::counters::timer::Timer;
use crate::hardware::ram::{Ram, RamChip, RamError, Rom};

use super::memory_mapping::{MemoryMapping, MemoryMappingEntry, RegionToMemoryMapper};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum IoRegistersMemoryMappingRegion {
	JoypadInput,
	SerialTransfer,
	DividerRegister,
	Timers,
	Audio,
}

const IO_REGISTER_MAPPING_SIZE: usize = 5;
const IO_REGISTER_MAPPING_ENTRIES: [MemoryMappingEntry<IoRegistersMemoryMappingRegion>; IO_REGISTER_MAPPING_SIZE] = [
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::JoypadInput, 0x0, 1),
	MemoryMappingEntry::new(
		IoRegistersMemoryMappingRegion::SerialTransfer,
		1,
		IO_REGISTER_SERIAL_TRANSFER_SIZE,
	),
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::DividerRegister, 0x4, 1),
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::Timers, 0x5, 3),
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::Audio, 0x10, 0x16),
];

const IO_REGISTER_SERIAL_TRANSFER_SIZE: usize = 2;

struct IoRegistersMemoryMapping {
	mapping: MemoryMapping<1, IoRegistersMemoryMappingRegion>,
	joypad_input: u8,
	serial_transfer: RamChip<IO_REGISTER_SERIAL_TRANSFER_SIZE>,
	divider_register: DividerRegister,
	timer: Timer,
	audio: Audio,
}

impl RegionToMemoryMapper for IoRegistersMemoryMapping {
	type R = IoRegistersMemoryMappingRegion;

	fn matching_entry(&self, address: u16) -> Result<MemoryMappingEntry<Self::R>, RamError> {
		self.mapping.find_mapping(address).copied()
	}

	fn get_rom(&self, region: Self::R) -> Result<&dyn Rom, RamError> {
		match region {
			IoRegistersMemoryMappingRegion::JoypadInput => Ok(&self.joypad_input),
			IoRegistersMemoryMappingRegion::SerialTransfer => Ok(&self.serial_transfer),
			IoRegistersMemoryMappingRegion::DividerRegister => Ok(&self.divider_register),
			IoRegistersMemoryMappingRegion::Timers => Ok(&self.timer),
			IoRegistersMemoryMappingRegion::Audio => Ok(&self.audio),
		}
	}

	fn get_ram(&mut self, region: Self::R) -> Result<&mut dyn Ram, RamError> {
		match region {
			IoRegistersMemoryMappingRegion::JoypadInput => Ok(&mut self.joypad_input),
			IoRegistersMemoryMappingRegion::SerialTransfer => Ok(&mut self.serial_transfer),
			IoRegistersMemoryMappingRegion::DividerRegister => Ok(&mut self.serial_transfer),
			IoRegistersMemoryMappingRegion::Timers => Ok(&mut self.timer),
			IoRegistersMemoryMappingRegion::Audio => Ok(&mut self.audio),
		}
	}
}
