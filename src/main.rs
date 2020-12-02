use std::env;
use std::fs;

mod cpu;
mod memory;

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 3 {
		eprintln!("Usage: {} ROM STEPS", &args[0]);
		return;
	}
	let rom = fs::read(&args[1]).unwrap();
	let steps: u32 = args[2].parse().unwrap();
	let mut memory = memory::SimpleMemory::new(&rom);
	let mut cpu = cpu::Cpu::new(&mut memory);
	println!("Initial state: {:?}", &cpu);
	for step in 0..steps {
		cpu.step();
		println!("After step {}: {:?}", step, &cpu);
	}
}
