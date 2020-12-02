
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

	fn write_reg(&mut self, selector: u8, value: u8) {
		match selector {
			0 => { self.reg_a = value; },
			1 => { self.reg_ip = value; },
			2 => { self.reg_p = value; },
			3 => { self.memory.write(self.bank_p, self.reg_p, value); },
			_ => panic!("Bad selector value"),
		}
	}

	pub fn step(&mut self) {
		let instruction = self.memory.read(self.bank_ip, self.reg_ip);
		let mut new_immediate = 0u8;
		// instruction only takes effect if condition flag matches first bit of instruction
		if (instruction & 0x80) >> 7 == self.condition as u8 {
			let new_cond: bool;
			// Bits 4 and 5 indicate major opcode
			match (instruction & 0x30) >> 4 {
				// immd VALUE - bits 0-3 are the immediate value
				0 => {
					new_immediate = instruction & 0x0f;
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
							// eg. I=1000 is bitwise AND, I=111 is OR
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
						self.write_reg((instruction & 0x0c) >> 2, result);
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
						self.write_reg(instruction & 0x03, result);
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
					// 4 no-op instructions - currently unused
					0 => {
						new_cond = false;
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
		// advance IP and immediate regs
		self.reg_ip = self.reg_ip.wrapping_add(1);
		self.reg_i = (self.reg_i << 4) + new_immediate;
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
