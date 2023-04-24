/*
The PpuDevice defines the interface that the PPU uses to access the rest of the device.
This makes what the PPU can access explicit, while decoupling the PPU from the rest of the device.
 */
pub(crate) trait PpuDevice {
	fn get_lcd_control(&self) -> u8;
}

pub(crate) struct Ppu {}

impl Ppu {
	pub(crate) fn new() -> Self {
		Self {}
	}

	pub(crate) fn render(&mut self, ppu_device: &dyn PpuDevice) {

	}
}
