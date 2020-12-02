
pub trait Memory {
	fn read(&self, bank: u8, addr: u8) -> u8;
	fn write(&mut self, bank: u8, addr: u8, value: u8);
}


// A dummy memory target that ignores writes and always reads 0
pub struct NullMemory {}

impl Memory for NullMemory {
	fn read(&self, _bank: u8, _addr: u8) -> u8 {
		0
	}
	fn write(&mut self, _bank: u8, _addr: u8, _value: u8) {}
}


// A straightforward directly mapped memory, initialized to 0 if not otherwise given data
pub struct SimpleMemory {
	data: [u8; 65536],
}

impl Memory for SimpleMemory {
	fn read(&self, bank: u8, addr: u8) -> u8 {
		self.data[Self::to_index(bank, addr)]
	}
	fn write(&mut self, bank: u8, addr: u8, value: u8) {
		self.data[Self::to_index(bank, addr)] = value;
	}
}

impl SimpleMemory {
	pub fn new(initial: &[u8]) -> Self {
		let mut data = [0u8; 65536];
		// This will panic if initial.len() > 65536
		data[..initial.len()].copy_from_slice(initial);
		SimpleMemory{data}
	}
	fn to_index(bank: u8, addr: u8) -> usize {
		(((bank as u16) << 8) | (addr as u16)).into()
	}
}
