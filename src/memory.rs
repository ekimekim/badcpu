
pub trait Memory {
	fn read(&self, bank: u8, addr: u8) -> u8;
	fn write(&mut self, bank: u8, addr: u8, value: u8);
}
