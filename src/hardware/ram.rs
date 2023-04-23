use crate::hardware::ram::bootstrap::BOOTSTRAP_DATA;
use crate::hardware::ram::io_registers::IoRegistersMemoryMapping;
use chips::{RamChip, RomChip};

pub(crate) const BOOSTRAP_RAM_START: u16 = 0x0000;

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

pub(crate) use error::RamError;
pub(crate) use traits::{Ram, Rom};

const BOOTSTRAP_RAM_SIZE: usize = 0x100;
const WORKING_RAM_SIZE: usize = (ECHO_RAM_START - WORKING_RAM_START) as usize;
const VIDEO_RAM_SIZE: usize = 8 * 1024;
const IO_REGISTERS_MAPPING_SIZE: usize = 0x80;
const OAM_SIZE: usize = 0xA0;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MappedRam {
	boostrap_ram: RomChip<'static, BOOTSTRAP_RAM_SIZE>,
	working_ram: RamChip<WORKING_RAM_SIZE>,
	video_ram: RamChip<VIDEO_RAM_SIZE>,
	mapped_io_registers: IoRegistersMemoryMapping,
	oam: RamChip<OAM_SIZE>,
}

#[derive(Copy, Clone)]
enum RamRegion {
	Bootstrap,
	WorkingRam,
	VideoRam,
	IoRegisters,
	Oam,
}

struct RamMapping {
	region: RamRegion,
	offset: u16,
	size: usize,
}

impl RamMapping {
	fn mapped_here(&self, address: u16) -> bool {
		address >= self.offset && usize::from(address - self.offset) < self.size
	}
}

const RAM_MAPPINGS: [RamMapping; 5] = [
	RamMapping {
		region: RamRegion::Bootstrap,
		offset: BOOSTRAP_RAM_START,
		size: BOOTSTRAP_RAM_SIZE,
	},
	RamMapping {
		region: RamRegion::WorkingRam,
		offset: WORKING_RAM_START,
		size: WORKING_RAM_SIZE,
	},
	RamMapping {
		region: RamRegion::VideoRam,
		offset: VIDEO_RAM_START,
		size: VIDEO_RAM_SIZE,
	},
	RamMapping {
		region: RamRegion::IoRegisters,
		offset: IO_REGISTERS_MAPPING_START,
		size: IO_REGISTERS_MAPPING_SIZE,
	},
	RamMapping {
		region: RamRegion::Oam,
		offset: OAM_START,
		size: OAM_SIZE,
	},
];

impl MappedRam {
	pub(crate) fn new() -> Self {
		Self {
			boostrap_ram: RomChip::new(BOOTSTRAP_DATA),
			working_ram: RamChip::default(),
			video_ram: RamChip::default(),
			mapped_io_registers: IoRegistersMemoryMapping::default(),
			oam: RamChip::default(),
		}
	}

	fn mapping_for_address(address: u16) -> Option<&'static RamMapping> {
		RAM_MAPPINGS.iter().find(|mapping| mapping.mapped_here(address))
	}

	fn get_mapped_ram(&self, region: RamRegion) -> &dyn Rom {
		match region {
			RamRegion::Bootstrap => &self.boostrap_ram,
			RamRegion::WorkingRam => &self.working_ram,
			RamRegion::VideoRam => &self.video_ram,
			RamRegion::IoRegisters => &self.mapped_io_registers,
			RamRegion::Oam => &self.oam,
		}
	}

	fn get_mapped_ram_mut(&mut self, region: RamRegion) -> &mut dyn Ram {
		match region {
			RamRegion::Bootstrap => panic!("Attempted to obtain write access to a ROM chip"),
			RamRegion::WorkingRam => &mut self.working_ram,
			RamRegion::VideoRam => &mut self.video_ram,
			RamRegion::IoRegisters => &mut self.mapped_io_registers,
			RamRegion::Oam => &mut self.oam,
		}
	}
}

impl Rom for MappedRam {
	fn read_byte(&self, address: u16) -> Result<u8, RamError> {
		let ram_mapping = MappedRam::mapping_for_address(address).ok_or(RamError::UnmappedRegion(address))?;
		let mapped_ram = self.get_mapped_ram(ram_mapping.region);

		let region_address = address - ram_mapping.offset;
		mapped_ram
			.read_byte(region_address)
			.map_err(|ram_error| ram_error.adjust_for_offset(ram_mapping.offset))
	}
}

impl Ram for MappedRam {
	fn write_byte(&mut self, address: u16, value: u8) -> Result<(), RamError> {
		let ram_mapping = MappedRam::mapping_for_address(address).ok_or(RamError::UnmappedRegion(address))?;
		let mapped_ram = self.get_mapped_ram_mut(ram_mapping.region);

		let region_address = address - ram_mapping.offset;
		mapped_ram
			.write_byte(region_address, value)
			.map_err(|ram_error| ram_error.adjust_for_offset(ram_mapping.offset))
	}
}
