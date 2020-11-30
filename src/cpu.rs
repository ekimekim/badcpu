use super::memory::Memory;

pub struct CPU<'a, M: Memory> {
	condition: bool,
	regI: u8,
	regA: u8,
	regIP: u8,
	regP: u8,
	bankIP: u8,
	bankP: u8,
	memory: &'a mut M,
}


impl<'a, M: Memory> CPU<'a, M> {
	pub fn new(memory: &'a mut M) -> Self {
		CPU {
			// Can I do something more interesting with initial conditions?
			condition: false,
			regI: 0,
			regA: 0,
			regIP: 0,
			regP: 0,
			bankIP: 0,
			bankP: 0,
			memory,
		}
	}

	fn readReg(&self, selector: u8) -> u8 {
		match selector {
			0 => self.regA,
			1 => self.regIP,
			2 => self.regP,
			3 => self.memory.read(self.bankP, self.regP),
			_ => panic!("Bad selector value"),
		}
	}

	fn writeReg(&mut self, selector: u8, value: u8) {
		match selector {
			0 => { self.regA = value; },
			1 => { self.regIP = value; },
			2 => { self.regP = value; },
			3 => { self.memory.write(self.bankP, self.regP, value); },
			_ => panic!("Bad selector value"),
		}
	}

	pub fn step(&mut self) {
		let instruction = self.memory.read(self.bankIP, self.regIP);
		let mut newImmediate = 0u8;
		// instruction only takes effect if condition flag matches first bit of instruction
		if (instruction & 0x80) >> 7 == self.condition as u8 {
			let newCond: bool;
			// Bits 4 and 5 indicate major opcode
			match (instruction & 0x30) >> 4 {
				// immd VALUE - bits 0-3 are the immediate value
				0 => {
					newImmediate = instruction & 0x0f;
					newCond = false; // immd instruction always results in 0 cond
				},
				// 2-op instructions - bits 2-3 are arg1, bits 0-1 are arg2
				2 | 3 => {
					// both args being [P] is not allowed and instead sets banks
					if instruction & 0x0f == 0x0f {
						// onto [P] [P] sets IP bank, bit [P] [P] sets P bank
						let destBank = if instruction & 0x30 == 0x30 {&mut self.bankIP} else {&mut self.bankP};
						*destBank = self.regI;
						newCond = false;
					} else {
						let base = self.readReg((instruction & 0x0c) >> 2);
						let arg = self.readReg(instruction & 0x03);
						let result = if instruction & 0x30 == 0x30 {
							// onto BASE ARG: BASE += ARG + I, set cond if any overflow
							let (value, first_overflow) = base.overflowing_add(arg);
							let (value, second_overflow) = value.overflowing_add(self.regI);
							newCond = first_overflow || second_overflow;
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
								let result_bit = (self.regI >> lookup_bit) & 1;
								value += result_bit << bit;
							}
							// Top 4 bits vbbb of I select condition output - bbb selects
							// a bit from resulting value, if this bit == v then set cond
							let condBit = self.regI & 0x70 >> 4;
							let condValue = (value >> condBit) & 1;
							newCond = condValue == ((self.regI & 0x80) >> 7);
							value
						};
						self.writeReg((instruction & 0x0c) >> 2, result);
					}
				},
				// All other ops are encoded under the 0x10 opcode,
				// look up minor opcode from bits 2-3
				1 => match (instruction & 0x0c) >> 2 {
					// inc|dec ARG - ARG += I or -= I respectively
					2 | 3 => {
						let arg = self.readReg(instruction & 0x03);
						let (result, overflow) = if (instruction & 0x0c) >> 2 == 2 {
							arg.overflowing_add(self.regI)
						} else {
							arg.overflowing_sub(self.regI)
						};
						self.writeReg(instruction & 0x03, result);
						newCond = overflow;
					},
					// mix ARG - for each pair of bits in ARG, set to another pair of bits
					// in ARG specified by the same pair in I. ie. if I = 11100100 this is a noop,
					// I = 10010011 is a 2-bit rotate left, I=0 expands bottom 2 bits to all
					// positions (0 => 0, 1 => 0x55, 2 => 0xaa, 3 => 0xff).
					// Condition flag is set if the output == 0.
					1 => {
						let arg = self.readReg(instruction & 0x03);
						let mut result = 0u8;
						// for each pair in output, look up address from I then look up value from arg
						for pair in 0..4 {
							let address = (self.regI >> (2 * pair)) & 0x03;
							let value = (arg >> (2 * address)) & 0x03;
							result += value << (2 * pair)
						}
						newCond = result == 0;
					},
					// 4 no-op instructions - currently unused
					0 => {
						newCond = false;
					},
					_ => unreachable!(),
				},
				_ => unreachable!(),
			}
			// update condition if bit 6 is set
			if (instruction & 0x40) != 0 {
				self.condition = newCond;
			}
		}
		// advance IP and immediate regs
		self.regIP = self.regIP.wrapping_add(1);
		self.regI = (self.regI << 4) + newImmediate;
	}
}
