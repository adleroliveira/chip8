use hardware::KeyboardDriver;

pub struct Keyboard {
    keys: [bool; 16]
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            keys: [false; 16]
        }
    }
}

impl KeyboardDriver for Keyboard {
    fn is_key_pressed(&self, key: u8) -> bool {
        self.keys[key as usize]
    }

    fn get_key(&self) -> Option<u8> {
        for i in 0..16 {
            if self.keys[i] {
                return Some(i as u8)
            }
        }

        None
    }

    fn press(&mut self, key: u8) {
        self.keys[key as usize] = true;

    }

    fn release(&mut self, key: u8) {
        self.keys[key as usize] = false;

    }
}