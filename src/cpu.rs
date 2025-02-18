
use std::fmt;

use super::memory::Memory;

pub struct Cpu<'a, M: Memory> {
	condition: bool,
	reg_i: u8,
	reg_a: u8,
	reg_ip: u8,
	reg_p: u8,
	bank_ip: u8,
	bank_p: u8,
	memory: &'a mut M,
}


impl<'a, M: Memory> Cpu<'a, M> {
	pub fn new(memory: &'a mut M) -> Self {
		Cpu {
			// Can I do something more interesting with initial conditions?
			condition: false,
			reg_i: 0,
			reg_a: 0,
			reg_ip: 0,
			reg_p: 0,
			bank_ip: 0,
			bank_p: 0,
			memory,
		}
	}

	fn read_reg(&self, selector: u8) -> u8 {
		match selector {
			0 => self.reg_a,
			1 => self.reg_ip,
			2 => self.reg_p,
			3 => self.memory.read(self.bank_p, self.reg_p),
			_ => panic!("Bad selector value"),
		}
	}

	// Note that instead of updating reg_ip directly, we take in a &mut new_ip that may be modified.
	// This is to assist the logic in step().
	fn write_reg(&mut self, new_ip: &mut u8, selector: u8, value: u8) {
		match selector {
			0 => { self.reg_a = value; },
			1 => { *new_ip = value; },
			2 => { self.reg_p = value; },
			3 => { self.memory.write(self.bank_p, self.reg_p, value); },
			_ => panic!("Bad selector value"),
		}
	}

	// Returns true if this instruction triggered a halt
	pub fn step(&mut self) -> bool {
		let instruction = self.memory.read(self.bank_ip, self.reg_ip);
		// these values are defaults, specific instructions might override these
		let mut new_immediate = self.reg_i << 4;
		let mut new_ip = self.reg_ip.wrapping_add(1);
		let mut halt = false;
		// instruction only takes effect if condition flag matches first bit of instruction
		if (instruction & 0x80) >> 7 == self.condition as u8 {
			let new_cond: bool; // Compiler will ensure we explicitly set this in every branch
			// Bits 4 and 5 indicate major opcode
			match (instruction & 0x30) >> 4 {
				// immd VALUE - bits 0-3 are the immediate value
				0 => {
					new_immediate += instruction & 0x0f;
					new_cond = false; // immd instruction always results in 0 cond
				},
				// 2-op instructions - bits 2-3 are arg1, bits 0-1 are arg2
				2 | 3 => {
					// both args being [P] is not allowed and instead sets banks
					if instruction & 0x0f == 0x0f {
						// onto [P] [P] sets IP bank, bit [P] [P] sets P bank
						let dest_bank = if instruction & 0x30 == 0x30 {&mut self.bank_ip} else {&mut self.bank_p};
						*dest_bank = self.reg_i;
						new_cond = false;
					} else {
						let base = self.read_reg((instruction & 0x0c) >> 2);
						let arg = self.read_reg(instruction & 0x03);
						let result = if instruction & 0x30 == 0x30 {
							// onto BASE ARG: BASE += ARG + I, set cond if any overflow
							let (value, first_overflow) = base.overflowing_add(arg);
							let (value, second_overflow) = value.overflowing_add(self.reg_i);
							new_cond = first_overflow || second_overflow;
							value
						} else {
							// bit BASE ARG: bitwise op on each bit of base and arg,
							// maps (base bit, arg bit) through bits 0-3 of I
							// eg. I=1000 is bitwise AND, I=1110 is OR
							let mut value = 0u8;
							for bit in 0..8 {
								// combine the two bits into a value 0-3
								let lookup_bit =
									(((base >> bit) & 1) << 1) +
									((arg >> bit) & 1);
								let result_bit = (self.reg_i >> lookup_bit) & 1;
								value += result_bit << bit;
							}
							// Top 4 bits vbbb of I select condition output - bbb selects
							// a bit from resulting value, if this bit == v then set cond
							let cond_bit = self.reg_i & 0x70 >> 4;
							let cond_value = (value >> cond_bit) & 1;
							new_cond = cond_value == ((self.reg_i & 0x80) >> 7);
							value
						};
						self.write_reg(&mut new_ip, (instruction & 0x0c) >> 2, result);
					}
				},
				// All other ops are encoded under the 0x10 opcode,
				// look up minor opcode from bits 2-3
				1 => match (instruction & 0x0c) >> 2 {
					// inc|dec ARG - ARG += I or -= I respectively
					2 | 3 => {
						let arg = self.read_reg(instruction & 0x03);
						let (result, overflow) = if (instruction & 0x0c) >> 2 == 2 {
							arg.overflowing_add(self.reg_i)
						} else {
							arg.overflowing_sub(self.reg_i)
						};
						self.write_reg(&mut new_ip, instruction & 0x03, result);
						new_cond = overflow;
					},
					// mix ARG - for each pair of bits in ARG, set to another pair of bits
					// in ARG specified by the same pair in I. ie. if I = 11100100 this is a noop,
					// I = 10010011 is a 2-bit rotate left, I=0 expands bottom 2 bits to all
					// positions (0 => 0, 1 => 0x55, 2 => 0xaa, 3 => 0xff).
					// Condition flag is set if the output == 0.
					1 => {
						let arg = self.read_reg(instruction & 0x03);
						let mut result = 0u8;
						// for each pair in output, look up address from I then look up value from arg
						for pair in 0..4 {
							let address = (self.reg_i >> (2 * pair)) & 0x03;
							let value = (arg >> (2 * address)) & 0x03;
							result += value << (2 * pair)
						}
						new_cond = result == 0;
					},
					// Last 4 no-argument ops are encoded under 0x10-0x13, look up subminor opcode from bits 0-1.
					0 => match instruction & 0x03 {
						// load - I = A. Also flips condition flag.
						0 => {
							new_immediate = self.reg_a;
							new_cond = !self.condition;
						},
						// halt - Stop execution. It may be resumed later, in which case this instruction
						// will be observed to have behaved the same as immd 0 (ie. nop).
						3 => {
							halt = true;
							new_cond = false;
						},
						// 2 no-op instructions - currently unused
						1 | 2 => {
							new_cond = false;
						},
						_ => unreachable!(),
					},
					_ => unreachable!(),
				},
				_ => unreachable!(),
			}
			// update condition if bit 6 is set
			if (instruction & 0x40) != 0 {
				self.condition = new_cond;
			}
		}
		// update IP and immediate regs
		self.reg_ip = new_ip;
		self.reg_i = new_immediate;
		// return whether we're halting
		halt
	}

	// Alternate version of step, which is intended to be more "hardware-like" in execution,
	// with variables expressed as mostly branchless results of other variables.
	// This isn't as readable, but is intended to test how "hardware-sympathetic" the ISA design is.
	// Instead of mutating in place, returns a halt bool + new version of Cpu.
	// It should always be true that both these lines have the same values for halt and cpu after:
	//   halt = cpu.step()
	//   halt, cpu = cpu.step_hw()
	pub fn step_hw(self) -> (bool, Self) {
		let instruction = self.memory.read(self.bank_ip, self.reg_ip);
		let match_condition = (instruction & 0x80) >> 7 == self.condition;
		let opcode = instruction & 0x3f;

		// TODO simplify by swapping base and arg regs in 2-op opcodes
		let base_reg =
			if opcode & 0x20 == 0x20 { (opcode & 0x0c) >> 2 } else { opcode & 0x03 };
		let arg_reg = opcode & 0x03;

		let read_value = self.memory.read(self.bank_p, self.reg_p);
		let reg_to_value = |reg| match reg {
			0 => self.reg_a,
			1 => self.reg_ip,
			2 => self.reg_p,
			3 => read_value,
			_ => unreachable!(),
		};
		let base = reg_to_value(base_reg);
		let arg = reg_to_value(arg_reg);

		// All opcodes write to their base reg, except zero-reg ops + bank special cases
		let is_write = opcode & 0x3c != 0x10 && opcode & 0x2f != 0x2f;

		// This is the "ALU" section, where the actual operations are defined.
		// Since many of these need to output to both the write reg and the condition,
		// we return both from this block.
		let (result, result_cond) =
			if opcode & 0x30 == 0x30 {
				// onto BASE ARG: BASE += ARG + I, set cond if any overflow
				let (value, first_overflow) = base.overflowing_add(arg);
				let (value, second_overflow) = value.overflowing_add(self.reg_i);
				(value, first_overflow || second_overflow)
			} else if opcode & 0x30 == 0x20 {
				// bit BASE ARG: bitwise op on each bit of base and arg,
				// maps (base bit, arg bit) through bits 0-3 of I
				// eg. I=1000 is bitwise AND, I=1110 is OR
				let value = (0..8).into_iter().map(|bit| {
					// combine the two bits into a value 0-3
					let lookup_bit =
						(((base >> bit) & 1) << 1) +
						((arg >> bit) & 1);
					let result_bit = (self.reg_i >> lookup_bit) & 1;
					result_bit << bit
				}).sum();
				// Top 4 bits vbbb of I select condition output - bbb selects
				// a bit from resulting value, if this bit == v then set cond
				let cond_bit = self.reg_i & 0x70 >> 4;
				let cond_value = (value >> cond_bit) & 1;
				let cond = cond_value == ((self.reg_i & 0x80) >> 7);
				(value, cond)
			} else if opcode & 0x3c == 0x14 {
				// mix BASE - for each pair of bits in BASE, set to another pair of bits
				// in BASE specified by the same pair in I. ie. if I = 11100100 this is a noop,
				// I = 10010011 is a 2-bit rotate left, I=0 expands bottom 2 bits to all
				// positions (0 => 0, 1 => 0x55, 2 => 0xaa, 3 => 0xff).
				// Condition flag is set if the output == 0.
				let result = (0..4).into_iter().map(|pair| {
					// for each pair in output, look up address from I then look up value from base
					let address = (self.reg_i >> (2 * pair)) & 0x03;
					let value = (base >> (2 * address)) & 0x03;
					value << (2 * pair)
				}).sum();
				(result, result == 0)
			} else if opcode & 0x3c == 0x18 {
				// inc BASE - BASE += I
				base.overflowing_add(self.reg_i)
			} else if opcode & 0x3c == 0x1c{
				// dec BASE - BASE -= I
				base.overflowing_sub(self.reg_i)
			} else {
				// result should be unused here, return dummy value
				assert!(!is_write);
				(0, false)
			};

		// This is the only actual conditional mutation in this function, and represents
		// whether to dispatch a write to memory or not.
		if match_condition && is_write && base_reg == 0x03 {
			self.memory.write(self.bank_p, self.reg_p, result);
		}

		let reg_a = if match_condition && is_write && base_reg == 0 { result } else { self.reg_a };
		// Note the default here if not modified is to advance the IP.
		// We do this even if the instruction is skipped entirely.
		let reg_ip = if match_condition && is_write && base_reg == 1 { result } else { self.reg_ip.wrapping_add(1) };
		let reg_p = if match_condition && is_write && base_reg == 2 { result } else { self.reg_p };

		// We handled reg-based instruction conditions above, here we only need to handle
		// special cases.
		let new_cond =
			if is_write { result_cond }
			// load flips the condition
			else if opcode == 0x10 { !self.condition }
			// all other cases just unset it
			else { false };

		let reg_i =
			// Skipped instructions don't change I
			if !match_condition { self.reg_i }
			// I is normally shifted left 4. A load instruction overrides this and sets it to A.
			else if opcode == 0x10 { self.reg_a }
			// Shift left, and add value if it was an immd instruction
			else { self.reg_i << 4 | if opcode & 0x30 == 0 { opcode & 0x0f } else { 0 } };
		// Bank values only change with specific bank set opcodes
		let bank_p = if match_condition && opcode == 0x2f { self.reg_i } else { self.bank_p };
		let bank_ip = if match_condition && opcode == 0x3f { self.reg_i } else { self.bank_ip };
		// A halt only occurs on a halt instruction.
		let halt = match_condition && opcode == 0x13;

		let new_cpu = Cpu {
			condition: if match_condition && instruction & 0x40 == 0x40 { new_cond } else { self.condition },
			reg_i,
			reg_a,
			reg_ip,
			reg_p,
			bank_ip,
			bank_p,
			memory: self.memory,
		};
		(halt, new_cpu)
	}
}

impl<'a, M: Memory> fmt::Debug for Cpu<'a, M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Cpu")
			.field("condition", &self.condition)
			.field("reg_i", &self.reg_i)
			.field("reg_a", &self.reg_a)
			.field("reg_ip", &self.reg_ip)
			.field("reg_p", &self.reg_p)
			.field("bank_ip", &self.bank_ip)
			.field("bank_p", &self.bank_p)
			.finish()
	}
}
