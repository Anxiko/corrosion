use std::fmt::{Display, Formatter};

use crate::hardware::ram::bootstrap::BOOTSTRAP_DATA;

pub(crate) const BOOSTRAP_RAM_START: u16 = 0x0000;

pub(crate) const VIDEO_RAM_START: u16 = 0x8000;
#[allow(unused)]
pub(crate) const VRAM_TILE_DATA_START: u16 = 0x800;

pub(crate) const WORKING_RAM_START: u16 = 0xC000;
pub(crate) const ECHO_RAM_START: u16 = 0xE000;

pub(crate) const OAM_START: u16 = 0xFE00;

pub(crate) const IO_REGISTERS_MAPPING_START: u16 = 0xFF00;

mod bootstrap;

const BOOTSTRAP_RAM_SIZE: usize = 0x100;
const WORKING_RAM_SIZE: usize = (ECHO_RAM_START - WORKING_RAM_START) as usize;
const VIDEO_RAM_SIZE: usize = 8 * 1024;
const IO_REGISTERS_MAPPING_SIZE: usize = 0x80;
const OAM_SIZE: usize = 0xA0;

pub(crate) trait Rom {
	fn read_byte(&self, address: u16) -> Result<u8, RamError>;

	fn read_double_byte(&self, address: u16) -> Result<u16, RamError> {
		let low = self.read_byte(address)?;
		let high = self.read_byte(address.wrapping_add(1))?;

		Ok(u16::from_be_bytes([high, low]))
	}
}

pub(crate) trait Ram: Rom {
	fn write_byte(&mut self, address: u16, value: u8) -> Result<(), RamError>;
	fn write_double_byte(&mut self, address: u16, value: u16) -> Result<(), RamError> {
		let [high, low] = value.to_be_bytes();
		self.write_byte(address, low)?;
		self.write_byte(address.wrapping_add(1), high)?;

		Ok(())
	}
}

#[derive(Debug, Copy, Clone)]
pub enum RamError {
	InvalidAddress(u16),
	UnmappedRegion(u16),
	WriteOnRom(u16),
}

impl RamError {
	fn adjust_for_offset(self, offset: u16) -> Self {
		match self {
			Self::InvalidAddress(address) => Self::InvalidAddress(address + offset),
			Self::UnmappedRegion(address) => Self::UnmappedRegion(address + offset),
			Self::WriteOnRom(address) => Self::WriteOnRom(address + offset),
		}
	}
}

impl Display for RamError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::UnmappedRegion(address) => {
				write!(f, "No mapped RAM region for {address}")
			}
			Self::InvalidAddress(address) => {
				write!(f, "Attempted to access invalid address {address}")
			}
			Self::WriteOnRom(address) => {
				write!(f, "Attempted write to ROM address {address}")
			}
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct MappedRam {
	boostrap_ram: RomChip<'static, BOOTSTRAP_RAM_SIZE>,
	working_ram: RamChip<WORKING_RAM_SIZE>,
	video_ram: RamChip<VIDEO_RAM_SIZE>,
	mapped_io_registers: MappedIoRegisters,
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
			working_ram: RamChip::new(),
			video_ram: RamChip::new(),
			mapped_io_registers: MappedIoRegisters::new(),
			oam: RamChip::new(),
		}
	}

	fn mapping_for_address(address: u16) -> Option<&'static RamMapping> {
		RAM_MAPPINGS
			.iter()
			.find(|mapping| mapping.mapped_here(address))
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
		let ram_mapping =
			MappedRam::mapping_for_address(address).ok_or(RamError::UnmappedRegion(address))?;
		let mapped_ram = self.get_mapped_ram(ram_mapping.region);

		let region_address = address - ram_mapping.offset;
		mapped_ram
			.read_byte(region_address)
			.map_err(|ram_error| ram_error.adjust_for_offset(ram_mapping.offset))
	}
}

impl Ram for MappedRam {
	fn write_byte(&mut self, address: u16, value: u8) -> Result<(), RamError> {
		let ram_mapping =
			MappedRam::mapping_for_address(address).ok_or(RamError::UnmappedRegion(address))?;
		let mapped_ram = self.get_mapped_ram_mut(ram_mapping.region);

		let region_address = address - ram_mapping.offset;
		mapped_ram
			.write_byte(region_address, value)
			.map_err(|ram_error| ram_error.adjust_for_offset(ram_mapping.offset))
	}
}

#[derive(Debug, Clone, PartialEq)]
struct RomChip<'a, const S: usize> {
	ref_memory: &'a [u8; S],
}

impl<'a, const S: usize> RomChip<'a, S> {
	fn new(ref_memory: &'a [u8; S]) -> Self {
		Self { ref_memory }
	}
}

impl<'a, const S: usize> Rom for RomChip<'a, S> {
	fn read_byte(&self, address: u16) -> Result<u8, RamError> {
		self.ref_memory
			.get(usize::from(address))
			.copied()
			.ok_or(RamError::InvalidAddress(address))
	}
}

#[derive(Debug, PartialEq, Clone)]
struct RamChip<const S: usize> {
	memory: Box<[u8; S]>,
}

impl<const S: usize> RamChip<S> {
	fn new() -> Self {
		Self {
			memory: Box::new([0; S]),
		}
	}
}

impl<const S: usize> Rom for RamChip<S> {
	fn read_byte(&self, address: u16) -> Result<u8, RamError> {
		self.memory
			.get(usize::from(address))
			.copied()
			.ok_or(RamError::InvalidAddress(address))
	}
}

impl<const S: usize> Ram for RamChip<S> {
	fn write_byte(&mut self, address: u16, value: u8) -> Result<(), RamError> {
		let ptr = self
			.memory
			.get_mut(usize::from(address))
			.ok_or(RamError::InvalidAddress(address))?;

		*ptr = value;
		Ok(())
	}
}

#[derive(Copy, Clone)]
enum IoRegisters {
	LcdControl,
}

#[derive(Default, Debug, PartialEq, Clone)]
struct MappedIoRegisters {
	lcd_control: u8,
}

impl MappedIoRegisters {
	fn new() -> Self {
		Self::default()
	}

	fn resolve_address(address: u16) -> Result<IoRegisters, RamError> {
		match address {
			0x40 => Ok(IoRegisters::LcdControl),
			address if address < 0x80 => Err(RamError::UnmappedRegion(address)),
			_ => Err(RamError::InvalidAddress(address)),
		}
	}

	fn get_io_register(&self, io_register: IoRegisters) -> &u8 {
		match io_register {
			IoRegisters::LcdControl => &self.lcd_control,
		}
	}

	fn get_io_register_mut(&mut self, io_register: IoRegisters) -> &mut u8 {
		match io_register {
			IoRegisters::LcdControl => &mut self.lcd_control,
		}
	}
}

impl Rom for MappedIoRegisters {
	fn read_byte(&self, address: u16) -> Result<u8, RamError> {
		MappedIoRegisters::resolve_address(address)
			.map(|io_register| *self.get_io_register(io_register))
	}
}

impl Ram for MappedIoRegisters {
	fn write_byte(&mut self, address: u16, value: u8) -> Result<(), RamError> {
		MappedIoRegisters::resolve_address(address)
			.map(|io_register| self.get_io_register_mut(io_register))
			.map(|ptr| {
				*ptr = value;
			})
	}
}
