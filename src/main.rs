extern crate sdl2;

use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use corrosion::decoder::fetch_and_decode;
use corrosion::hardware::cpu::Cpu;
use corrosion::instructions::ExecutionError;

fn update_cpu(cpu: &mut Cpu) -> Result<(), ExecutionError> {
	let instruction = fetch_and_decode(cpu)?;
	instruction.execute(cpu)?;
	Ok(())
}

pub fn main() -> Result<(), String> {
	let mut cpu = Cpu::new();

	let sdl_context = sdl2::init()?;
	let video_subsystem = sdl_context.video()?;

	let window = video_subsystem
		.window("rust-sdl2 demo: Video", 800, 600)
		.position_centered()
		.opengl()
		.build()
		.map_err(|e| e.to_string())?;

	let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

	canvas.set_draw_color(Color::RGB(255, 0, 0));
	canvas.clear();
	canvas.present();
	let mut event_pump = sdl_context.event_pump()?;

	let mut running = true;

	while running {
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit { .. }
				| Event::KeyDown {
					keycode: Some(Keycode::Escape),
					..
				} => running = false,
				_ => {}
			}
		}

		canvas.clear();
		canvas.present();
		std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));

		let execution_result = update_cpu(&mut cpu);
		if let Err(err) = execution_result {
			println!("{err}");
			running = false;
		}
	}

	Ok(())
}
