pub trait AudioDriver {

}

pub trait KeyboardDriver {
    fn is_key_pressed(&self, key: u8) -> bool;
    fn get_key(&self) -> Option<u8>;
    fn press(&mut self, key: u8);
    fn release(&mut self, key: u8);
}