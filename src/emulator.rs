use corrosion::decoder::fetch_and_decode;
use corrosion::hardware::cpu::Cpu;
use corrosion::instructions::ExecutionError;

fn update_cpu(cpu: &mut Cpu) -> Result<(), ExecutionError> {
	let pc = cpu.current_pc();
	let instruction = fetch_and_decode(cpu)?;
	println!("{pc:#06X}: {instruction}");
	instruction.execute(cpu)
}

fn main() -> Result<(), String> {
	let mut cpu = Cpu::new();

	loop {
		let execution_result = update_cpu(&mut cpu);
		if let Err(err) = execution_result {
			println!("ERROR: {err}");
			break;
		}
	}

	Ok(())
}
