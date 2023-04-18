use corrosion::decoder::fetch_and_decode;
use corrosion::hardware::cpu::Cpu;
use corrosion::instructions::ExecutionError;

fn update_cpu(cpu: &mut Cpu) -> Result<(), ExecutionError> {
	let instruction = fetch_and_decode(cpu)?;
	println!("{instruction}");
	instruction.execute(cpu)?;
	Ok(())
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
