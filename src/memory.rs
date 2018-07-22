use chip8::{CHIP8_WIDTH, CHIP8_HEIGHT, FONT_SET};

pub struct Memory {
    pub ram: [u8; 4096],
    pub stack: [u16; 16],
    pub vram: [[u8; CHIP8_WIDTH]; CHIP8_HEIGHT],
    pub vram_changed: bool
}

impl Memory {
    pub fn new() -> Self {
        let stack = [0; 16];
        let vram = [[0; CHIP8_WIDTH]; CHIP8_HEIGHT];
        let mut ram = [0; 4096];
        let vram_changed = true;

        for i in 0..FONT_SET.len() {
            ram[i] = FONT_SET[i];
        }

        Memory { ram, stack, vram, vram_changed }
    }
}