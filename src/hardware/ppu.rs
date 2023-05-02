/*
The PpuDevice defines the interface that the PPU uses to access the rest of the device.
This makes what the PPU can access explicit, while decoupling the PPU from the rest of the device.
 */

use crate::hardware::ppu::lcd_control::{DecodedLcdControl, TileDataAddressMode, TileMapAddressMode};
use crate::hardware::screen::position::ScreenCord;

const SCREEN_SIZE_WIDTH: usize = 160;
const SCREEN_SIZE_HEIGHT: usize = 144;

const TILE_MAP_SIZE: usize = 32; // Tile maps are square, so this is the width and height in tiles
const BITS_PER_TILE: usize = 8; // Tiles themselves are also squares


const VRAM_TILE_MAP_OFFSET: usize = 0x1800;


mod lcd_control;
mod tile_index;

pub(crate) trait PpuDevice {
	fn get_lcd_control(&self) -> u8;
	fn get_bg_screen_cord(&self) -> &ScreenCord;
	fn get_window_screen_cord(&self) -> &ScreenCord;
}

pub(crate) struct Ppu {}

impl Ppu {
	pub(crate) fn new() -> Self {
		Self {}
	}

	pub(crate) fn render(&mut self, ppu_device: &impl PpuDevice) {
		let lcd_control = DecodedLcdControl::from(ppu_device.get_lcd_control());

		if !lcd_control.display_enabled {
			self.screen_off()
		}
		self.clear_screen();

		if lcd_control.bg_and_window_priority {
			self.draw_bg(
				lcd_control.tile_data,
				lcd_control.bg_tile_map,
				ppu_device.get_bg_screen_cord(),
			);

			if lcd_control.window_enable {
				self.draw_window(
					lcd_control.tile_data,
					lcd_control.window_tile_map,
					ppu_device.get_window_screen_cord(),
				);
			}
		}
	}

	fn clear_screen(&self) {
		todo!()
	}

	fn screen_off(&self) {
		todo!()
	}

	fn draw_window(&self, tile_data: TileDataAddressMode, tile_map: TileMapAddressMode, coord: &ScreenCord) {
		todo!()
	}

	fn draw_bg(&self, tile_data: TileDataAddressMode, tile_map: TileMapAddressMode, coord: &ScreenCord) {
		todo!()
	}
}
