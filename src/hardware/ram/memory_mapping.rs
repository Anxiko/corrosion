use std::fmt::Debug;

use crate::hardware::ram::{Ram, Rom};

use super::RamError;

pub(super) trait MemoryMappingEntryRegion: Copy + Clone + PartialEq + Eq + Debug {}

impl<T: Copy + Clone + PartialEq + Eq + Debug> MemoryMappingEntryRegion for T {}

pub(super) struct MemoryMappingEntry<R: MemoryMappingEntryRegion> {
	region: R,
	offset: u16,
	size: usize,
}

impl<R: MemoryMappingEntryRegion> MemoryMappingEntry<R> {
	pub(super) const fn new(region: R, offset: u16, size: usize) -> Self {
		Self {
			region,
			offset,
			size,
		}
	}

	fn mapped_here(&self, address: u16) -> bool {
		(address >= self.offset) && (address - self.offset < self.offset)
	}

	fn adjust_address(&self, address: u16) -> u16 {
		address - self.offset
	}

	fn bubble_error(&self, err: RamError) -> RamError {
		err.adjust_for_offset(self.offset)
	}
}

pub(super) struct MemoryMapping<const S: usize, R: MemoryMappingEntryRegion> {
	regions: [MemoryMappingEntry<R>; S],
}

impl<const S: usize, R: MemoryMappingEntryRegion> MemoryMapping<S, R> {
	fn new(regions: [MemoryMappingEntry<R>; S]) -> Self {
		Self { regions }
	}

	pub(super) fn find_mapping(&self, address: u16) -> Result<&MemoryMappingEntry<R>, RamError> {
		self.regions
			.iter()
			.find(|entry| entry.mapped_here(address))
			.ok_or(RamError::UnmappedRegion(address))
	}
}

pub(super) trait RegionToMemoryMapper {
	type R: MemoryMappingEntryRegion;
	fn matching_entry(&self, address: u16) -> Result<&MemoryMappingEntry<Self::R>, RamError>;

	fn get_rom(&self, region: Self::R) -> Result<Box<dyn Rom>, RamError>;
	fn get_ram(&self, region: Self::R) -> Result<Box<dyn Ram>, RamError>;
}

impl<M: RegionToMemoryMapper> Rom for M {
	fn read_byte(&self, address: u16) -> Result<u8, RamError> {
		let entry = self.matching_entry(address)?;
		self.get_rom(entry.region)
			.and_then(|rom| rom.read_byte(entry.adjust_address(address)))
			.map_err(|err| entry.bubble_error(err))
	}

	fn read_double_byte(&self, address: u16) -> Result<u16, RamError> {
		let entry = self.matching_entry(address)?;
		self.get_rom(entry.region)
			.and_then(|rom| rom.read_double_byte(entry.adjust_address(address)))
			.map_err(|err| entry.bubble_error(err))
	}
}

impl<M: RegionToMemoryMapper> Ram for M {
	fn write_byte(&mut self, address: u16, value: u8) -> Result<(), RamError> {
		let entry = self.matching_entry(address)?;
		self.get_ram(entry.region)
			.and_then(|mut ram| ram.write_byte(entry.adjust_address(address), value))
			.map_err(|err| entry.bubble_error(err))
	}

	fn write_double_byte(&mut self, address: u16, value: u16) -> Result<(), RamError> {
		let entry = self.matching_entry(address)?;
		self.get_ram(entry.region)
			.and_then(|mut ram| ram.write_double_byte(entry.adjust_address(address), value))
			.map_err(|err| entry.bubble_error(err))
	}
}
