use super::Tick;
use crate::hardware::ram::{Ram, RamError, Rom};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive, Default)]
#[repr(u8)]
enum InputClockSelect {
	#[default]
	C00 = 0b00,
	C01 = 0b01,
	C10 = 0b10,
	C11 = 0b11,
}

impl InputClockSelect {
	fn freq(&self) -> u16 {
		match self {
			InputClockSelect::C00 => 1024,
			InputClockSelect::C01 => 16,
			InputClockSelect::C10 => 64,
			InputClockSelect::C11 => 256,
		}
	}
}

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub(crate) struct Timer {
	enabled: bool,
	selected_clock_speed: InputClockSelect,
	counter: u8, // The actual counter, incremented each time ticks reaches the value selected by the control
	modulo: u8,
	ticks: u16, // Ticks between counter updates
}

impl Timer {
	fn get_timer_control(&self) -> u8 {
		u8::from(self.selected_clock_speed) | (if self.enabled { 1 << 2 } else { 0 })
	}

	fn set_timer_control(&mut self, value: u8) {
		let masked_clock_speed = value & 0b11;
		let selected_clock_speed = InputClockSelect::try_from(masked_clock_speed).unwrap();
		let enabled = (value & (1 << 2)) != 0;

		self.selected_clock_speed = selected_clock_speed;
		self.enabled = enabled;

		self.ticks = self.ticks % self.selected_clock_speed.freq();
	}
}

impl Timer {}

impl Tick for Timer {
	fn tick(&mut self) {
		let freq = self.selected_clock_speed.freq();
		let new_ticks = (self.ticks + 1) % freq;
		let update_counter = ((self.ticks + 1) / freq) > 0;

		self.ticks = new_ticks;

		if update_counter && self.enabled {
			let (new_counter, overflow) = self.counter.overflowing_add(1);
			if overflow {
				// TODO: trigger interrupt
				self.counter = self.modulo;
			} else {
				self.counter = new_counter;
			}
		}
	}
}

impl Rom for Timer {
	fn read_byte(&self, address: u16) -> Result<u8, RamError> {
		match address {
			0x0 => Ok(self.counter),
			0x1 => Ok(self.modulo),
			0x2 => Ok(self.get_timer_control()),
			_ => Err(RamError::InvalidAddress(address)),
		}
	}

	fn read_double_byte(&self, address: u16) -> Result<u16, RamError> {
		Err(RamError::InvalidAddress(address))
	}
}

impl Ram for Timer {
	fn write_byte(&mut self, address: u16, value: u8) -> Result<(), RamError> {
		match address {
			0x0 => Err(RamError::WriteOnRom(address)),
			0x1 => {
				self.modulo = value;
				Ok(())
			}
			0x2 => {
				self.set_timer_control(value);
				Ok(())
			}
			_ => Err(RamError::InvalidAddress(address)),
		}
	}

	fn write_double_byte(&mut self, address: u16, _value: u16) -> Result<(), RamError> {
		Err(RamError::InvalidAddress(address))
	}
}
