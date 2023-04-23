use crate::hardware::audio::Audio;
use crate::hardware::counters::divider::DividerRegister;
use crate::hardware::counters::timer::Timer;
use crate::hardware::ram::{Ram, RamError, Rom};
use crate::hardware::ram::chips::RamChip;
use crate::hardware::screen::position::ScreenCord;

use super::memory_mapping::{MemoryMapping, MemoryMappingEntry, RegionToMemoryMapper};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(super) enum IoRegistersMemoryMappingRegion {
	JoypadInput,
	SerialTransfer,
	DividerRegister,
	Timers,
	Audio,
	Wave,
	LcdControl,
	LcdStatus,
	ScreenScroll,
	ScreenPosition,
	Bgp,
}

const IO_REGISTER_MAPPING_SIZE: usize = 11;
const IO_REGISTER_MAPPING_ENTRIES: [MemoryMappingEntry<IoRegistersMemoryMappingRegion>; IO_REGISTER_MAPPING_SIZE] = [
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::JoypadInput, 0x0, 1),
	MemoryMappingEntry::new(
		IoRegistersMemoryMappingRegion::SerialTransfer,
		1,
		IO_REGISTER_SERIAL_TRANSFER_SIZE,
	),
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::DividerRegister, 0x4, 0x1),
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::Timers, 0x5, IO_REGISTER_TIMERS_SIZE),
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::Audio, 0x10, IO_REGISTER_AUDIO_SIZE),
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::Wave, 0x30, IO_REGISTER_WAVE_SIZE),
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::LcdControl, 0x40, 0x1),
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::LcdStatus, 0x41, 0x1),
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::ScreenScroll, 0x42, 0x2),
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::ScreenPosition, 0x4A, 0x2),
	MemoryMappingEntry::new(IoRegistersMemoryMappingRegion::Bgp, 0x47, 0x1),
];

const IO_REGISTER_SERIAL_TRANSFER_SIZE: usize = 0x2;
const IO_REGISTER_TIMERS_SIZE: usize = 0x3;
const IO_REGISTER_AUDIO_SIZE: usize = 0x17;
const IO_REGISTER_WAVE_SIZE: usize = 0x10;

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub(super) struct IoRegistersMemoryMapping {
	mapping: MemoryMapping<IO_REGISTER_MAPPING_SIZE, IoRegistersMemoryMappingRegion>,
	joypad_input: u8,
	serial_transfer: RamChip<IO_REGISTER_SERIAL_TRANSFER_SIZE>,
	divider_register: DividerRegister,
	timer: Timer,
	audio: Audio,
	wave: RamChip<IO_REGISTER_WAVE_SIZE>,
	lcd_control: u8,
	lcd_status: u8,
	screen_scroll: ScreenCord,
	screen_position: ScreenCord,
	bgp: u8,
}

impl Default for MemoryMapping<IO_REGISTER_MAPPING_SIZE, IoRegistersMemoryMappingRegion> {
	fn default() -> Self {
		Self::new(IO_REGISTER_MAPPING_ENTRIES)
	}
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
			IoRegistersMemoryMappingRegion::Wave => Ok(&self.wave),
			IoRegistersMemoryMappingRegion::LcdControl => Ok(&self.lcd_control),
			IoRegistersMemoryMappingRegion::LcdStatus => Ok(&self.lcd_status),
			IoRegistersMemoryMappingRegion::ScreenPosition => Ok(&self.screen_position),
			IoRegistersMemoryMappingRegion::ScreenScroll => Ok(&self.screen_scroll),
			IoRegistersMemoryMappingRegion::Bgp => Ok(&self.bgp),
		}
	}

	fn get_ram(&mut self, region: Self::R) -> Result<&mut dyn Ram, RamError> {
		match region {
			IoRegistersMemoryMappingRegion::JoypadInput => Ok(&mut self.joypad_input),
			IoRegistersMemoryMappingRegion::SerialTransfer => Ok(&mut self.serial_transfer),
			IoRegistersMemoryMappingRegion::DividerRegister => Ok(&mut self.serial_transfer),
			IoRegistersMemoryMappingRegion::Timers => Ok(&mut self.timer),
			IoRegistersMemoryMappingRegion::Audio => Ok(&mut self.audio),
			IoRegistersMemoryMappingRegion::Wave => Ok(&mut self.wave),
			IoRegistersMemoryMappingRegion::LcdControl => Ok(&mut self.lcd_control),
			IoRegistersMemoryMappingRegion::LcdStatus => Ok(&mut self.lcd_status),
			IoRegistersMemoryMappingRegion::ScreenPosition => Ok(&mut self.screen_position),
			IoRegistersMemoryMappingRegion::ScreenScroll => Ok(&mut self.screen_scroll),
			IoRegistersMemoryMappingRegion::Bgp => Ok(&mut self.bgp),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_audio_entry() {
		let memory_mapping = IoRegistersMemoryMapping::default();

		let actual = memory_mapping.matching_entry(0x26).expect("Get mapping entry");
		let expected = IO_REGISTER_MAPPING_ENTRIES[4];

		assert_eq!(actual, expected);
	}

	#[test]
	fn write_to_audio() {
		let mut memory_mapping = IoRegistersMemoryMapping::default();
		memory_mapping.write_byte(0x26, 0x0).expect("Write to Audio")
	}
}
