use std::env;
use std::fs;

mod cpu;
mod memory;

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 2 {
		println!("Usage: {} ROM", &args[0]);
		return;
	}
	let rom = fs::read(args[1]).unwrap();
	let mut memory = memory::SimpleMemory::new(rom);
	let mut cpu = cpu::Cpu::new(&mut memory);
	// null memory means these will always be nops (0x00 -> immd 0)
	cpu.step();
}
