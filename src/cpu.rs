use chip8::{OPCODE_SIZE, CHIP8_WIDTH, CHIP8_HEIGHT};
use memory::Memory;
use hardware::KeyboardDriver;
use std::sync::{Arc, Mutex};
use rand::{self, Rng};

pub enum Action {
    Next,
    Skip,
    Halt,
    Jump(u16),
}

pub struct Cpu {
    pub debug: bool,
    pub v: [u8; 16],
    pub i: u16,
    pub pc: u16,
    pub sp: u8,
    pub sound_timer: u8,
    pub delay_timer: u8,
    pub memory: Arc<Mutex<Memory>>,
    pub halt: bool,
    pub keypad_waiting: bool,
    pub keypad_register: u8,
}

impl Cpu {
    pub fn new(memory: Arc<Mutex<Memory>>) -> Self {
        Cpu {
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            sound_timer: 0,
            delay_timer: 0,
            memory,
            halt: false,
            keypad_waiting: false,
            keypad_register: 0,
            debug: false,
        }
    }

    pub fn debug(&self, data: &str) {
        if self.debug {
            println!("{:?}", data);
        }
    }

    pub fn opcode(&self) -> u16 {
        let memory = self.memory.lock().unwrap();
        (memory.ram[self.pc as usize] as u16) << 8 | (memory.ram[(self.pc + 1) as usize] as u16)
    }

    pub fn tick<K>(&mut self, keyboard: &K) where K: KeyboardDriver {
        if !self.halt {
            if !self.keypad_waiting {
                if self.delay_timer > 0 {
                    self.delay_timer -= 1
                }
                if self.sound_timer > 0 {
                    self.sound_timer -= 1
                }
                let opcode = self.opcode();
                self.run_opcode(opcode, keyboard);
            } else {
                // println!("Waiting for key");
                let key = keyboard.get_key();
                if let Some(k) = key {
                    self.keypad_waiting = false;
                    self.v[self.keypad_register as usize] = k;
                }
            }
        }
    }

    pub fn run_opcode<K>(&mut self, opcode: u16, keyboard: &K) where K: KeyboardDriver {
        // println!("OPCODE 0x{:X}", opcode);

        let nibbles = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );

        let nnn = (opcode & 0x0FFF) as u16;
        let kk = (opcode & 0x00FF) as u8;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3 as usize;

        let action = match nibbles {
            (0x0, 0x0, 0x0, 0x0)    => self.halt(),
            (0x0, 0x0, 0xE, 0x0)    => self.op_00e0(),              // CLS
            (0x0, 0x0, 0xE, 0xE)    => self.op_00ee(),              // RET
            (0x1, _, _, _)          => self.op_1nnn(nnn),           // JP addr
            (0x2, _, _, _)          => self.op_2nnn(nnn),           // CALL addr
            (0x3, _, _, _)          => self.op_3xkk(x, kk),         // SE Vx, byte
            (0x4, _, _, _)          => self.op_4xkk(x, kk),         // SNE Vx, byte
            (0x5, _, _, 0x00)       => self.op_5xy0(x, y),          // SE Vx, Vy
            (0x6, _, _, _)          => self.op_6xkk(x, kk),         // LD Vx, byte
            (0x7, _, _, _)          => self.op_7xkk(x, kk),         // ADD Vx, byte
            (0x8, _, _, 0x00)       => self.op_8xy0(x, y),          // LD Vx, Vy
            (0x8, _, _, 0x01)       => self.op_8xy1(x, y),          // OR Vx, Vy
            (0x8, _, _, 0x02)       => self.op_8xy2(x, y),          // AND Vx, Vy
            (0x8, _, _, 0x03)       => self.op_8xy3(x, y),          // XOR Vx, Vy
            (0x8, _, _, 0x04)       => self.op_8xy4(x, y),          // ADD Vx, Vy
            (0x8, _, _, 0x05)       => self.op_8xy5(x, y),          // SUB Vx, Vy
            (0x8, _, _, 0x06)       => self.op_8xy6(x, y),          // SHR Vx {, Vy}
            (0x8, _, _, 0x07)       => self.op_8xy7(x, y),          // SUBN Vx, Vy
            (0x8, _, _, 0x0E)       => self.op_8xye(x, y),          // SHL Vx {, Vy}
            (0x9, _, _, 0x00)       => self.op_9xy0(x, y),          // SNE Vx, Vy
            (0xA, _, _, _)          => self.op_annn(nnn),           // LDI, addr
            (0xB, _, _, _)          => self.op_bnnn(nnn),           // JP V0, addr
            (0xC, _, _, _)          => self.op_cxkk(x, kk),         // RND Vx, byte
            (0xD, _, _, _)          => self.op_dxyn(x, y, n),       // DRW Vx, Vy, nibble
            (0xE, _, 0x09, 0x0E)    => self.op_ex9e(x, keyboard),   // SKP Vx
            (0xE, _, 0x0A, 0x01)    => self.op_exa1(x, keyboard),   // SKNP Vx
            (0xF, _, 0x00, 0x07)    => self.op_fx07(x),         // LD Vx, DT
            (0xF, _, 0x00, 0x0A)    => self.op_fx0a(x),         // LD Vx, K
            (0xF, _, 0x01, 0x05)    => self.op_fx15(x),         // LD DT, Vx
            (0xF, _, 0x01, 0x08)    => self.op_fx18(x),         // LD ST, Vx
            (0xF, _, 0x01, 0x0E)    => self.op_fx1e(x),         // ADD I, Vx
            (0xF, _, 0x02, 0x09)    => self.op_fx29(x),         // LD F, Vx
            (0xF, _, 0x03, 0x03)    => self.op_fx33(x),         // LD B, Vx
            (0xF, _, 0x05, 0x05)    => self.op_fx55(x),         // LD [I], Vx
            (0xF, _, 0x06, 0x05)    => self.op_fx65(x),         // LD Vx, [I]
            _ => unimplemented!("OPCODE {:X}", opcode)
        };

        match action {
            Action::Next        => self.pc += OPCODE_SIZE,
            Action::Skip        => self.pc += 2 * OPCODE_SIZE,
            Action::Halt        => self.halt = true,
            Action::Jump(addr)  => self.pc = addr,
        }
    }
}

// Implement OPCODES
impl Cpu {
    pub fn halt(&mut self) -> Action {
        println!("Halting...");
        Action::Halt
    }

    pub fn op_00e0(&mut self) -> Action {
        self.debug("CLS");
        let mut mem = self.memory.lock().unwrap();
        mem.vram = [[0; CHIP8_WIDTH]; CHIP8_HEIGHT];
        mem.vram_changed = true;
        Action::Next
    }

    pub fn op_00ee(&mut self) -> Action {
        let memory = self.memory.lock().unwrap();
        self.sp -= 1;
        self.pc = memory.stack[self.sp as usize];
        self.debug(&format!("RET 0x{:X}", self.pc));
        Action::Next
    }

    pub fn op_1nnn(&mut self, nnn: u16) -> Action {
        self.debug(&format!("JP 0x{:X}", nnn));
        Action::Jump(nnn)
    }

    pub fn op_2nnn(&mut self, nnn: u16) -> Action {
        self.debug(&format!("CALL 0x{:X}", nnn));
        let mut memory = self.memory.lock().unwrap();
        memory.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        Action::Jump(nnn)
    }

    pub fn op_3xkk(&mut self, x: usize, kk: u8) -> Action {
        self.debug(&format!("SE V{:?} [0x{:X} ({:?})] 0x{:X} ({:?})", x, self.v[x], self.v[x], kk, kk));
        if self.v[x] == kk {
            Action::Skip
        } else {
            Action::Next
        }
    }

    pub fn op_4xkk(&mut self, x: usize, kk: u8) -> Action {
        self.debug(&format!("SNE V{:?} 0x{:X}", x, kk));
        if self.v[x] != kk {
            Action::Skip
        } else {
            Action::Next
        }
    }

    pub fn op_5xy0(&mut self, x: usize, y: usize) -> Action {
        self.debug(&format!("SE V{:?} V{:?}", x, y));
        if self.v[x] == self.v[y] {
            Action::Skip
        } else {
            Action::Next
        }
    }

    pub fn op_6xkk(&mut self, x: usize, kk: u8) -> Action {
        self.debug(&format!("LD V{:?} 0x{:X} ({:?})", x, kk, kk));
        self.v[x] = kk;
        Action::Next
    }

    pub fn op_7xkk(&mut self, x: usize, kk: u8) -> Action {
        self.debug(&format!("ADD V{:?} 0x{:X}", x, kk));
        self.v[x] = self.v[x].wrapping_add(kk);
        Action::Next
    }

    pub fn op_8xy0(&mut self, x: usize, y: usize) -> Action {
        self.debug(&format!("LD V{:?} V{:?}", x, y));
        self.v[x] = self.v[y];
        Action::Next
    }

    pub fn op_8xy1(&mut self, x: usize, y: usize) -> Action {
        self.debug(&format!("OR V{:?} V{:?}", x, y));
        self.v[x] = self.v[x] | self.v[y];
        Action::Next
    }

    pub fn op_8xy2(&mut self, x: usize, y: usize) -> Action {
        self.debug(&format!("AND V{:?} V{:?}", x, y));
        self.v[x] = self.v[x] & self.v[y];
        Action::Next
    }

    pub fn op_8xy3(&mut self, x: usize, y: usize) -> Action {
        self.debug(&format!("XOR V{:?} V{:?}", x, y));
        self.v[x] = self.v[x] ^ self.v[y];
        Action::Next
    }

    pub fn op_8xy4(&mut self, x: usize, y: usize) -> Action {
        self.debug(&format!("ADD V{:?} V{:?}", x, y));
        let vx = self.v[x] as u16;
        let vy = self.v[y] as u16;
        let result = vx + vy;
        self.v[x] = self.v[x].wrapping_add(self.v[y]);
        self.v[0xF] = if result > 0xFF { 1 } else { 0 };
        Action::Next
    }

    pub fn op_8xy5(&mut self, x: usize, y: usize) -> Action {
        self.debug(&format!("SUB V{:?} V{:?}", x, y));
        self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 };
        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
        Action::Next
    }

    pub fn op_8xy6(&mut self, x: usize, _y: usize) -> Action {
        self.debug(&format!("SHR V{:?}", x));
        self.v[0xF] = self.v[x] & 0x1;
        self.v[x] >>= 1;
        Action::Next
    }

    pub fn op_8xy7(&mut self, x: usize, y: usize) -> Action {
        let vx = self.v[x];
        let vy = self.v[y];
        self.v[0xF] = if vy > vx { 1 } else { 0 };
        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
        Action::Next
    }

    pub fn op_8xye(&mut self, x: usize, _y: usize) -> Action {
        self.v[0xF] = ((self.v[x] & 0x80) >> 7) & 0x1;
        self.v[x] <<= 1;
        Action::Next
    }

    pub fn op_9xy0(&mut self, x: usize, y: usize) -> Action {
        if self.v[x] != self.v[y] {
            Action::Skip
        } else {
            Action::Next
        }
    }

    pub fn op_annn(&mut self, nnn: u16) -> Action {
        self.debug(&format!("LDI 0x{:X} ({:?})", nnn, nnn));
        self.i = nnn;
        Action::Next
    }

    pub fn op_bnnn(&mut self, nnn: u16) -> Action {
        Action::Jump(self.v[0x0] as u16 + nnn)
    }

    pub fn op_cxkk(&mut self, x: usize, kk: u8) -> Action {
        let mut rng = rand::thread_rng();
        self.v[x] = rng.gen::<u8>() & kk;
        Action::Next
    }

    pub fn op_dxyn(&mut self, x: usize, y: usize, n: usize) -> Action {
        self.debug(&format!("DRW V{:?} V{:?} {:X} ({:?})", x, y, n, n));
        let mut memory = self.memory.lock().unwrap();
        for byte in 0..n as usize {
            let y = (self.v[y] as usize + byte) % CHIP8_HEIGHT;
            for bit in 0..8 {
                let x = (self.v[x].wrapping_add(bit)) as usize % CHIP8_WIDTH;
                let color = (memory.ram[self.i as usize + byte] >> (7 - bit)) & 1;
                self.v[0xF] |= color & memory.vram[y][x];
                memory.vram[y][x] ^= color;
            }
        }
        memory.vram_changed = true;
        Action::Next
    }

    pub fn op_ex9e<K>(&mut self, x: usize, keyboard: &K) -> Action
        where K: KeyboardDriver
    {
        if keyboard.is_key_pressed(self.v[x]) {
            Action::Skip
        } else {
            Action::Next
        }
    }

    pub fn op_exa1<K>(&mut self, x: usize, keyboard: &K) -> Action
        where K: KeyboardDriver
    {
        if !keyboard.is_key_pressed(self.v[x]) {
            Action::Skip
        } else {
            Action::Next
        }
    }

    pub fn op_fx07(&mut self, x: usize) -> Action {
        self.v[x] = self.delay_timer;
        Action::Next
    }

    pub fn op_fx0a(&mut self, x: usize) -> Action {
        self.keypad_waiting = true;
        self.keypad_register = x as u8;
        Action::Next
    }

    pub fn op_fx15(&mut self, x: usize) -> Action {
        self.delay_timer = self.v[x];
        Action::Next
    }

    pub fn op_fx18(&mut self, x: usize) -> Action {
        self.sound_timer = self.v[x];
        Action::Next
    }

    pub fn op_fx1e(&mut self, x: usize) -> Action {
        self.i = self.i.wrapping_add(self.v[x].into());
        Action::Next
    }

    pub fn op_fx29(&mut self, x: usize) -> Action {
        self.i = ((self.v[x] as usize) * 5) as u16;
        Action::Next
    }

    pub fn op_fx33(&mut self, x: usize) -> Action {
        let mut memory = self.memory.lock().unwrap();
        memory.ram[self.i as usize] = self.v[x] / 100;
        memory.ram[(self.i + 1) as usize] = (self.v[x] / 10) % 10;
        memory.ram[(self.i + 2) as usize] = (self.v[x] % 100) % 10;
        Action::Next
    }

    pub fn op_fx55(&mut self, x: usize) -> Action {
        let mut memory = self.memory.lock().unwrap();
        for i in 0..(x as u16 + 1) {
            memory.ram[(self.i + i) as usize] = self.v[i as usize];
        }
        Action::Next
    }

    pub fn op_fx65(&mut self, x: usize) -> Action {
        let memory = self.memory.lock().unwrap();
        for i in 0..(x as u16 + 1) {
            self.v[i as usize] = memory.ram[(self.i + i) as usize];
        }
        Action::Next
    }
}