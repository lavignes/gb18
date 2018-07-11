pub trait Mmu {
    fn read(&self, address: u16) -> u8;

    fn write(&mut self, address: u16, value: u8);
}

pub struct Mbc0 {

}

impl Mmu for Mbc0 {
    fn write(&mut self, address: u16, value: u8) {

    }

    fn read(&self, address: u16) -> u8 {
        0
    }
}