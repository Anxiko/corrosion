use crate::hardware::ram::bootstrap::BOOTSTRAP_DATA;
use crate::hardware::ram::io_registers::IoRegistersMemoryMapping;
use chips::{RamChip, RomChip};

pub(crate) const BOOTSTRAP_RAM_START: u16 = 0x0000;

pub(crate) const VIDEO_RAM_START: u16 = 0x8000;
#[allow(unused)]
pub(crate) const VRAM_TILE_DATA_START: u16 = 0x800;

pub(crate) const WORKING_RAM_START: u16 = 0xC000;
pub(crate) const ECHO_RAM_START: u16 = 0xE000;

pub(crate) const OAM_START: u16 = 0xFE00;

pub(crate) const IO_REGISTERS_MAPPING_START: u16 = 0xFF00;

mod bootstrap;
mod chips;
mod error;
mod io_registers;
mod memory_mapping;
mod traits;

use crate::hardware::ram::memory_mapping::{MemoryMapping, MemoryMappingEntry, RegionToMemoryMapper, RegionToMemoryMapperError};
pub(crate) use error::RamError;
pub(crate) use traits::{Ram, Rom};

const BOOTSTRAP_RAM_SIZE: usize = 0x100;
const WORKING_RAM_SIZE: usize = (ECHO_RAM_START - WORKING_RAM_START) as usize;
const VIDEO_RAM_SIZE: usize = 8 * 1024;
const IO_REGISTERS_MAPPING_SIZE: usize = 0x80;
const OAM_SIZE: usize = 0xA0;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum MappedMemoryRegion {
	Bootstrap,
	WorkingRam,
	VideoRam,
	IoRegisters,
	Oam,
}

const MEMORY_MAPPING_SIZE: usize = 5;
const MEMORY_MAPPING_REGIONS: [MemoryMappingEntry<MappedMemoryRegion>; MEMORY_MAPPING_SIZE] = [
	MemoryMappingEntry::new(MappedMemoryRegion::Bootstrap, BOOTSTRAP_RAM_START, BOOTSTRAP_RAM_SIZE),
	MemoryMappingEntry::new(MappedMemoryRegion::WorkingRam, WORKING_RAM_START, WORKING_RAM_SIZE),
	MemoryMappingEntry::new(MappedMemoryRegion::VideoRam, VIDEO_RAM_START, VIDEO_RAM_SIZE),
	MemoryMappingEntry::new(
		MappedMemoryRegion::IoRegisters,
		IO_REGISTERS_MAPPING_START,
		IO_REGISTERS_MAPPING_SIZE,
	),
	MemoryMappingEntry::new(MappedMemoryRegion::Oam, OAM_START, OAM_SIZE),
];

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MappedMemory {
	mapping: MemoryMapping<MEMORY_MAPPING_SIZE, MappedMemoryRegion>,
	boostrap_ram: RomChip<'static, BOOTSTRAP_RAM_SIZE>,
	working_ram: RamChip<WORKING_RAM_SIZE>,
	video_ram: RamChip<VIDEO_RAM_SIZE>,
	mapped_io_registers: IoRegistersMemoryMapping,
	oam: RamChip<OAM_SIZE>,
}

impl MappedMemory {
	pub(crate) fn new() -> Self {
		Self {
			mapping: MemoryMapping::new(MEMORY_MAPPING_REGIONS),
			boostrap_ram: RomChip::new(BOOTSTRAP_DATA),
			working_ram: RamChip::default(),
			video_ram: RamChip::default(),
			mapped_io_registers: IoRegistersMemoryMapping::default(),
			oam: RamChip::default(),
		}
	}
}

impl RegionToMemoryMapper for MappedMemory {
	type R = MappedMemoryRegion;

	fn matching_entry(&self, address: u16) -> Result<MemoryMappingEntry<Self::R>, RamError> {
		self.mapping.find_mapping(address).copied()
	}

	fn get_rom(&self, region: Self::R) -> Result<&dyn Rom, RegionToMemoryMapperError> {
		Ok(match region {
			MappedMemoryRegion::Bootstrap => &self.boostrap_ram,
			MappedMemoryRegion::WorkingRam => &self.working_ram,
			MappedMemoryRegion::VideoRam => &self.video_ram,
			MappedMemoryRegion::IoRegisters => &self.mapped_io_registers,
			MappedMemoryRegion::Oam => &self.oam,
		})
	}

	fn get_ram(&mut self, region: Self::R) -> Result<&mut dyn Ram, RegionToMemoryMapperError> {
		match region {
			MappedMemoryRegion::Bootstrap => Err(RegionToMemoryMapperError::WriteOnRom),
			MappedMemoryRegion::WorkingRam => Ok(&mut self.working_ram),
			MappedMemoryRegion::VideoRam => Ok(&mut self.video_ram),
			MappedMemoryRegion::IoRegisters => Ok(&mut self.mapped_io_registers),
			MappedMemoryRegion::Oam => Ok(&mut self.oam),
		}
	}
}
