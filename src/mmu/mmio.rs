/// Trait for read / write functions separate from the bus
///
/// Ideal for memory mapped registers.
#[allow(clippy::upper_case_acronyms)]
pub trait MMIO {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);

    fn write_with_callback<F: FnMut()>(&mut self, address: u16, value: u8, _cb: F) {
        self.write(address, value);
    }
}
