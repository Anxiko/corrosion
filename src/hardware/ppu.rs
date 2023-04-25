/*
The PpuDevice defines the interface that the PPU uses to access the rest of the device.
This makes what the PPU can access explicit, while decoupling the PPU from the rest of the device.
 */

use crate::hardware::ppu::lcd_control::DecodedLcdControl;

mod lcd_control;

pub(crate) trait PpuDevice {
	fn get_lcd_control(&self) -> u8;
}

pub(crate) struct Ppu {}

impl Ppu {
	pub(crate) fn new() -> Self {
		Self {}
	}

	pub(crate) fn render(&mut self, ppu_device: &impl PpuDevice) {
		let lcd_control = DecodedLcdControl::from(ppu_device.get_lcd_control());
	}
}
