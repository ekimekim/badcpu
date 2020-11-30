mod cpu;
mod memory;

fn main() {
	let mut memory = memory::NullMemory{};
	let mut cpu = cpu::Cpu::new(&mut memory);
	// null memory means these will always be nops (0x00 -> immd 0)
	cpu.step();
}
