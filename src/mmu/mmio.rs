/// Trait for read / write functions separate from the bus
///
/// Ideal for memory mapped registers.
pub trait MMIO {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}
