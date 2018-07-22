use rand::{self, Rng};
// # TODO List:
// - Create VDriver Trait
// - Create ADriver Trait
// - Create KDriver Trait
// - Create basic drivers to operate with Chip8 implementation
// - Debug possible problems?
// - Create a Chip8 pseudo-assembler
// - Write Snake using Chip8 pseudo-assembler

// +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
// General Notes:
// 
// The keyboard Chip8 original / mapping layout:
// +------------+        +------------+
// + 1  2  3  C +   ->   + 1  2  3  4 +
// +------------+        +------------+
// + 4  5  6  D +   ->   + Q  W  E  R +
// +------------+        +------------+
// + 7  8  9  E +   ->   + A  S  D  F +
// +------------+        +------------+
// + A  0  B  F +   ->   + Z  X  C  V +
// +------------+        +------------+

// The original implementation of the Chip-8 language used a 64x32-pixel monochrome display
// with this format:
// (0,0)    (63,0)
// (0,31)   (63, 31)


// Some other interpreters, most notably the one on the ETI 660, also had 64x48 and 64x64 modes
// so we might try to implement that later. Maybe?
// +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

// This are sprites for the number and letters (hexadecimal).
pub const FONT_SET: [u8; 80] = 
[
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const OPCODE_SIZE: u16 = 2;
const CHIP8_WIDTH: usize = 64;
const CHIP8_HEIGHT: usize = 32;

pub struct Chip8 {
    // CHIP-8 allows for the usage of sixteen eight-bit general purpose registers
    // The registers are refered to as V0 to VF one for each hexadecimal digit but it 
    // should be noted that the VF register is often modified by certain instructions
    // to act as a flag.
    pub registers: [u8; 16],

    // The Chip-8 language is capable of accessing up to 4KB (4,096 bytes) of RAM
    // The first 512 bytes, from 0x000 to 0x1FF, are where the original interpreter
    // was located and should not be used.
    // Note: Program should be loadad to memory starting on the adress 0x200
    pub ram: [u8; 4096],

    // VRAM is the buffer used to store display data. The display driver will pull
    // the bytes from the vram to draw the screen.
    pub vram: [[u8; CHIP8_WIDTH]; CHIP8_HEIGHT],

    // This 16-bit register called I is generally used to store memory addresses
    // so only the lowest (rightmost) 12 bits are usually used.
    pub i: u16,

    // Chip-8 also has two special purpose 8-bit registers, for the delay and sound timers
    // When these registers are non-zero, they are automatically decremented at a rate of 60Hz
    pub sound_timer: u8,
    pub delay_timer: u8,

    // The program counter should be 16-bit, and is used to store the currently executing address
    pub pc: u16,

    // The stack is used to store the address that the interpreter shoud return to when finished
    // with a subroutine. Chip-8 allows for up to 16 levels of nested subroutines.
    pub stack: [u16; 16],

    // The stack pointer (SP) can be 8-bit, it is used to point to the topmost level of the stack
    pub stack_pointer: u8,

    // flag that is setted whenever the video driver should draw to the screen
    pub should_draw: bool,

    // this vector holds the keyboard keys currently down
    pub keypad: [bool; 16],

    // flag the is set if the keypad is waiting for a key to be pressed
    pub keypad_waiting: bool,

    // special register that holds a value where a keypress should be stored
    pub keypad_register: u8,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            registers: [0; 16],
            ram: [0; 4096],
            vram: [[0; CHIP8_WIDTH]; CHIP8_HEIGHT],
            i: 0x200,
            sound_timer: 0,
            delay_timer: 0,
            pc: 0x200,
            stack: [0; 16],
            stack_pointer: 0,
            should_draw: false,
            keypad: [false; 16],
            keypad_waiting: false,
            keypad_register: 0,
        };

        // Load the font set sprites into memory from the beggining of the memory address.
        // Remember: the addresses from 0x000 to 0x1FF are used by the interpreter
        for i in 0..FONT_SET.len() {
            chip8.ram[i] = FONT_SET[i];
        }

        chip8
    }

    // in order to retrieve a byte code we look at memory address specified in the
    // program counter. Because the memory is a contiguous vector of 1 byte of data
    // we need to retrieve two bytes to form an opcode (which is composed of two bytes).
    // To do that we retrieve the first byte: ram[pc] and cast it to u16 resultind in:
    // 00000000xxxxxxxx. Then we shift it 8 bits to the left to get xxxxxxxx00000000.
    // After that we retrieve the next byte: ram[pc+1] and to the same and then perform
    // bitwise OR (|) in the two to end up with: xxxxxxxxyyyyyyyy (16 bit OPCODE),
    pub fn opcode(&self) -> u16 {
        (self.ram[self.pc as usize] as u16) << 8 | (self.ram[(self.pc + 1) as usize] as u16)
    }

    pub fn run_opcode(&mut self, opcode: u16) {
        // First lets extract the nibbles from the opcode in a tuple of 4 elements (on for each nible)
        // and cast them to u8 so that 0x1230 would end up like (0b1, 0b2, 0b3, 0b0)
        let nibbles = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );

        println!("OPCODE 0x{:X}", opcode);

        let nnn = (opcode & 0x0FFF) as u16;
        let kk = (opcode & 0x00FF) as u8;
        let x = nibbles.1;
        let y = nibbles.2;
        let n = nibbles.3;

        match nibbles {
            (0x0, 0x0, 0xE, 0x0)    => self.op_00e0(),          // CLS
            (0x0, 0x0, 0xE, 0xE)    => self.op_00ee(),          // RET
            (0x1, _, _, _)          => self.op_1nnn(nnn),       // JP addr
            (0x2, _, _, _)          => self.op_2nnn(nnn),       // CALL addr
            (0x3, _, _, _)          => self.op_3xkk(x, kk),     // SE Vx, byte
            (0x4, _, _, _)          => self.op_4xkk(x, kk),     // SNE Vx, byte
            (0x5, _, _, 0x00)       => self.op_5xy0(x, y),      // SE Vx, Vy
            (0x6, _, _, _)          => self.op_6xkk(x, kk),     // LD Vx, byte
            (0x7, _, _, _)          => self.op_7xkk(x, kk),     // ADD Vx, byte
            (0x8, _, _, 0x00)       => self.op_8xy0(x, y),      // LD Vx, Vy
            (0x8, _, _, 0x01)       => self.op_8xy1(x, y),      // OR Vx, Vy
            (0x8, _, _, 0x02)       => self.op_8xy2(x, y),      // AND Vx, Vy
            (0x8, _, _, 0x03)       => self.op_8xy3(x, y),      // XOR Vx, Vy
            (0x8, _, _, 0x04)       => self.op_8xy4(x, y),      // ADD Vx, Vy
            (0x8, _, _, 0x05)       => self.op_8xy5(x, y),      // SUB Vx, Vy
            (0x8, _, _, 0x06)       => self.op_8xy6(x, y),      // SHR Vx {, Vy}
            (0x8, _, _, 0x07)       => self.op_8xy7(x, y),      // SUBN Vx, Vy
            (0x8, _, _, 0x0E)       => self.op_8xye(x, y),      // SHL Vx {, Vy}
            (0x9, _, _, 0x00)       => self.op_9xy0(x, y),      // SNE Vx, Vy
            (0xA, _, _, _)          => self.op_annn(nnn),       // LD I, addr
            (0xB, _, _, _)          => self.op_bnnn(nnn),       // JP V0, addr
            (0xC, _, _, _)          => self.op_cxkk(x, kk),     // RND Vx, byte
            (0xD, _, _, _)          => self.op_dxyn(x, y, n),   // DRW Vx, Vy, nibble
            (0xE, _, 0x09, 0x0E)    => self.op_ex9e(x),         // SKP Vx
            (0xE, _, 0x0A, 0x01)    => self.op_exa1(x),         // SKNP Vx
            (0xF, _, 0x00, 0x07)    => self.op_fx07(x),         // LD Vx, DT
            (0xF, _, 0x00, 0x0A)    => self.op_fx0a(x),         // LD Vx, K
            (0xF, _, 0x01, 0x05)    => self.op_fx15(x),         // LD DT, Vx
            (0xF, _, 0x01, 0x08)    => self.op_fx18(x),         // LD ST, Vx
            (0xF, _, 0x01, 0x0E)    => self.op_fx1e(x),         // ADD I, Vx
            (0xF, _, 0x02, 0x09)    => self.op_fx29(x),         // LD F, Vx
            (0xF, _, 0x03, 0x03)    => self.op_fx33(x),         // LD B, Vx
            (0xF, _, 0x05, 0x05)    => self.op_fx55(x),         // LD [I], Vx
            (0xF, _, 0x06, 0x05)    => self.op_fx65(x),         // LD Vx, [I]
            _ => unimplemented!("OPCODE MATCH {:X}", opcode)
        }

        // println!("{:?}", nibbles);
    }

    pub fn load_program(&mut self, program: &[u8]) {
        if program.len() > (4096 - 512) {
            panic!("This program is too big to run in this interpreter");
        }
        for addr in 0..program.len() {
            self.ram[0x200 + addr] = program[addr];
        }
        println!("{:#} bytes loaded to memory..", program.len());
    }

    pub fn pc_increse(&mut self) {
        self.pc += OPCODE_SIZE;
        // println!("PC INCREASED TO 0x{:X}", self.pc);
    }

    pub fn skip(&mut self) {
        self.pc += 2 * OPCODE_SIZE;
        // println!("PC SKIPPED TO 0x{:X}", self.pc);
    }

    pub fn jump(&mut self, nnn: u16) {
        self.pc = nnn;
        // println!("PROGRAM COUNTER IS NOW 0x{:X}", self.pc);
    }

    pub fn tick(&mut self, keypad: [bool; 16]) {
        self.should_draw = false;

        for i in 0..keypad.len() {
            self.keypad[i] = keypad[i];
        }

        if self.keypad_waiting {
            // println!("WAITING FOR KEY");
            for i in 0..keypad.len() {
                if keypad[i] {
                    self.keypad_waiting = false;
                    self.registers[self.keypad_register as usize] = i as u8;
                    break;
                }
            }
        } else {
            if self.delay_timer > 0 {
                self.delay_timer -= 1
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1
            }
            let opcode = self.opcode();
            self.run_opcode(opcode);
        }
    }
}

// Lets implement the OPCODES
// I chose to make a separated block of impl to improve readability.
// Let's mantain Chip8 geral logic from opcodes execution.
impl Chip8 {
    // SYS addr (Deprecated in modern computers)
    pub fn op_0nnn (&self) {
        panic!("THIS OPCODE SHOULD BE DEPRECATED IN MODERN COMPUTERS (0x0NNN)");
    }

    // 00E0 - CLS: Clear Display
    pub fn op_00e0(&mut self) {
        self.vram = [[0; CHIP8_WIDTH]; CHIP8_HEIGHT];
        self.should_draw = true;
        self.pc_increse();
    }

    // 00EE - RET: Return from a subroutine.
    pub fn op_00ee(&mut self) {
        self.pc = self.stack[self.stack_pointer as usize];
        self.stack_pointer -= 1;
        // println!("PROGRAM COUNTER SET TO 0x{:X}", self.pc);
        // println!("STACK POINTER SUBTRACTED (-1). Current 0x{:X}", self.stack_pointer);
        self.pc_increse();
    }

    // 1nnn - JP addr: Jump to location nnn.
    pub fn op_1nnn(&mut self, nnn: u16) {
        self.jump(nnn);
    }

    // 2nnn - CALL addr: Call subroutine at nnn.
    pub fn op_2nnn(&mut self, nnn: u16) {
        self.stack_pointer += 1;
        // println!("STACK POINTER INCREMENTED TO 0x{:X}", self.stack_pointer);

        self.stack[self.stack_pointer as usize] = self.pc;
        // println!("PUSH 0x{:X} INTO THE STACK", self.pc);

        self.jump(nnn);
    }

    // 3xkk - SE Vx, byte: Skip next instruction if Vx = kk.
    pub fn op_3xkk(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] == kk {
            self.skip();
        } else {
            self.pc_increse();
        }
    }

    // 4xkk - SNE Vx, byte: Skip next instruction if Vx != kk.
    pub fn op_4xkk(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] != kk {
            self.skip();
        } else {
            self.pc_increse();
        }
    }

    // 5xy0 - SE Vx, Vy: Skip next instruction if Vx = Vy.
    pub fn op_5xy0(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] == self.registers[y as usize] {
            self.skip();
        } else {
            self.pc_increse();
        }
    }

    // 6xkk - LD Vx, byte: The interpreter puts the value kk into register Vx.
    pub fn op_6xkk(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = kk;
        // println!("STORE VALUE 0x{:X} IN REGISTER 0x{:X}", kk, x);
        self.pc_increse();
    }

    // 7xkk - ADD Vx, byte: Set Vx = Vx + kk.
    pub fn op_7xkk(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = self.registers[x as usize].wrapping_add(kk);
        // println!("REGISTER 0x{:X} NOW HAVE VALUE 0x{:X}", x, self.registers[x as usize]);
        self.pc_increse();
    }

    // 8xy0 - LD Vx, Vy: Set Vx = Vy.
    pub fn op_8xy0(&mut self, x: u8, y: u8) {
        self.registers[x as usize] = self.registers[y as usize];
        // println!("REGISTER 0x{:X} NOW HAVE VALUE 0x{:X}", x, self.registers[x as usize]);
        self.pc_increse();
    }

    // 8xy1 - OR Vx, Vy: Set Vx = Vx OR Vy.
    pub fn op_8xy1(&mut self, x: u8, y: u8) {
        self.registers[x as usize] = self.registers[x as usize] | self.registers[y as usize];
        // println!("REGISTER 0x{:X} NOW HAVE VALUE 0x{:X}", x, self.registers[x as usize]);
        self.pc_increse();
    }

    // 8xy2 - AND Vx, Vy: Set Vx = Vx AND Vy.
    pub fn op_8xy2(&mut self, x: u8, y: u8) {
        self.registers[x as usize] = self.registers[x as usize] & self.registers[y as usize];
        // println!("REGISTER 0x{:X} NOW HAVE VALUE 0x{:X}", x, self.registers[x as usize]);
        self.pc_increse();
    }

    // 8xy3 - XOR Vx, Vy: Set Vx = Vx XOR Vy.
    pub fn op_8xy3(&mut self, x: u8, y: u8) {
        self.registers[x as usize] = self.registers[x as usize] ^ self.registers[y as usize];
        // println!("REGISTER 0x{:X} NOW HAVE VALUE 0x{:X}", x, self.registers[x as usize]);
        self.pc_increse();
    }

    // 8xy4 - ADD Vx, Vy: Set Vx = Vx + Vy, set VF = carry
    pub fn op_8xy4(&mut self, x: u8, y: u8) {
        let result = self.registers[x as usize] as u16 + self.registers[y as usize] as u16;
        self.registers[x as usize] = result as u8;
        // println!("REGISTER 0x{:X} NOW HAVE VALUE 0x{:X}", x, self.registers[x as usize]);
        self.registers[0x0F] = if result > 0xFF { 1 } else { 0 };
        // println!("(CARRIE) REGISTER 0x0F NOW HAVE VALUE 0x{:X}", self.registers[0x0F]);
        self.pc_increse();
    }

    // 8xy5 - SUB Vx, Vy: Set Vx = Vx - Vy, set VF = NOT borrow.
    pub fn op_8xy5(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        self.registers[0x0F] = if vx > vy { 1 } else { 0 };
        self.registers[x as usize] = vx.wrapping_sub(vy);
        // println!("REGISTER 0x{:X} NOW HAVE VALUE 0x{:X}", x, self.registers[x as usize]);
        // println!("(CARRIE) REGISTER 0x0F NOW HAVE VALUE 0x{:X}", self.registers[0x0F]);
        self.pc_increse();
    }

    // 8xy6 - SHR Vx {, Vy}: Set Vx = Vx SHR 1.
    pub fn op_8xy6(&mut self, x: u8, _y: u8) {
        self.registers[0x0F] = self.registers[x as usize] & 0x1;
        self.registers[x as usize] >>= 1;
        // println!("REGISTER 0x{:X} NOW HAVE VALUE 0x{:X}", x, self.registers[x as usize]);
        // println!("(CARRIE) REGISTER 0x0F NOW HAVE VALUE 0x{:X}", self.registers[0x0F]);
        self.pc_increse();
    }

    // 8xy7 - SUBN Vx, Vy: Set Vx = Vy - Vx, set VF = NOT borrow.
    pub fn op_8xy7(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];
        self.registers[0x0F] = if vy > vx { 1 } else { 0 };
        self.registers[x as usize] = self.registers[x as usize].wrapping_sub(self.registers[y as usize]);
        // println!("REGISTER 0x{:X} NOW HAVE VALUE 0x{:X}", x, self.registers[x as usize]);
        // println!("(CARRIE) REGISTER 0x0F NOW HAVE VALUE 0x{:X}", self.registers[0x0F]);
        self.pc_increse();
    }

    // 8xyE - SHL Vx {, Vy}: Set Vx = Vx SHL 1.
    pub fn op_8xye(&mut self, x: u8, _y: u8) {
        self.registers[0x0F] = (self.registers[x as usize] >> 7) & 0x1;
        self.registers[x as usize] <<= 1;
        // println!("REGISTER 0x{:X} NOW HAVE VALUE 0x{:X}", x, self.registers[x as usize]);
        // println!("(CARRIE) REGISTER 0x0F NOW HAVE VALUE 0x{:X}", self.registers[0x0F]);
        self.pc_increse();
    }

    // 9xy0 - SNE Vx, Vy: Skip next instruction if Vx != Vy.
    pub fn op_9xy0(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] != self.registers[y as usize] {
            self.skip();
        } else {
            self.pc_increse();
        }
    }

    // Annn - LD I, addr: Set I = nnn.
    pub fn op_annn(&mut self, nnn: u16) {
        // println!("SPECIAL REGISTER I WAS SET TO 0x{:X}", nnn);
        self.i = nnn;
        self.pc_increse();
    }

    // Bnnn - JP V0, addr: Jump to location nnn + V0.
    pub fn op_bnnn(&mut self, nnn: u16) {
        let addr = self.registers[0x0] as u16 + nnn;
        self.jump(addr);
    }

    // Cxkk - RND Vx, byte: Set Vx = random byte AND kk.
    pub fn op_cxkk(&mut self, x: u8, kk: u8) {
        let mut rng = rand::thread_rng();
        self.registers[x as usize] = rng.gen::<u8>() & kk;
        // println!("REGISTER 0x{:X} NOW HAVE VALUE 0x{:X}", x, self.registers[x as usize]);
        self.pc_increse();
    }

    // Dxyn - DRW Vx, Vy, nibble: Display n-byte sprite starting at memory location I at (Vx, Vy)
    // set VF = collision.
    // The interpreter reads n bytes from memory, starting at the address
    // stored in I. These bytes are then displayed as sprites on screen at
    // coordinates (Vx, Vy). Sprites are XORed onto the existing screen.
    // If this causes any pixels to be erased, VF is set to 1, otherwise
    // it is set to 0. If the sprite is positioned so part of it is outside
    // the coordinates of the display, it wraps around to the opposite side
    // of the screen.
    pub fn op_dxyn(&mut self, x: u8, y: u8, n: u8) {
        for byte in 0..n as usize {
            let y = (self.registers[y as usize] as usize + byte) % CHIP8_HEIGHT;
            for bit in 0..8 {
                let x = (self.registers[x as usize] as usize + bit) % CHIP8_WIDTH;
                let color = (self.ram[self.i as usize + byte] >> (7 - bit)) & 1;
                self.registers[0x0F] |= color & self.vram[y as usize][x as usize];
                self.vram[y as usize][x as usize] ^= color;
            }
        }
        self.should_draw = true;
        self.pc_increse();
    }

    // Ex9E - SKP Vx: Skip next instruction if key with the value of Vx is pressed.
    pub fn op_ex9e(&mut self, x: u8) {
        if self.keypad[self.registers[x as usize] as usize] {
            self.skip();
        } else {
            self.pc_increse();
        }
    }

    // ExA1 - SKNP Vx: Skip next instruction if key with the value of Vx is not pressed.
    pub fn op_exa1(&mut self, x: u8) {
        if !self.keypad[self.registers[x as usize] as usize] {
            self.skip();
        } else {
            self.pc_increse();
        }
    }

    // Fx07 - LD Vx, DT: Set Vx = delay timer value.
    pub fn op_fx07(&mut self, x: u8) {
        self.registers[x as usize] = self.delay_timer;
        // println!("REGISTER 0x{:X} NOW HAVE VALUE 0x{:X}", x, self.registers[x as usize]);
        self.pc_increse();
    }

    // Fx0A - LD Vx, K: Wait for a key press, store the value of the key in Vx.
    pub fn op_fx0a(&mut self, x: u8) {
        self.keypad_waiting = true;
        self.keypad_register = x;
        self.pc_increse();
        // println!("PROGRAM IS NOW WAITING FOR A KEY TO BE PRESSED");
    }

    // Fx15 - LD DT, Vx: Set delay timer = Vx.
    pub fn op_fx15(&mut self, x: u8) {
        self.delay_timer = self.registers[x as usize];
        // println!("DELAY TIMER IS SET TO 0x{:X}", self.delay_timer);
        self.pc_increse();
    }

    // Fx18 - LD ST, Vx: Set sound timer = Vx.
    pub fn op_fx18(&mut self, x: u8) {
        self.sound_timer = self.registers[x as usize];
        // println!("SOUND TIMER IS SET TO 0x{:X}", self.delay_timer);
        self.pc_increse();
    }

    // Fx1E - ADD I, Vx: Set I = I + Vx.
    pub fn op_fx1e(&mut self, x: u8) {
        self.i = self.i.wrapping_add(self.registers[x as usize].into());
        // println!("I REGISTER NOW HAVE VALUE 0x{:X}", self.i);
        self.pc_increse();
    }

    // Fx29 - LD F, Vx: Set I = location of sprite for digit Vx.
    pub fn op_fx29(&mut self, x: u8) {
        self.i = ((self.registers[x as usize] as usize) * 5) as u16;
        // println!("I REGISTER NOW HAVE VALUE 0x{:X}", self.i);
        self.pc_increse();
    }

    // Fx33 - LD B, Vx: Store BCD representation of Vx in memory locations I, I+1, and I+2.
    pub fn op_fx33(&mut self, x: u8) {
        self.ram[self.i as usize] = self.registers[x as usize] / 100;
        self.ram[(self.i + 1) as usize] = (self.registers[x as usize] % 100) / 10;
        self.ram[(self.i + 2) as usize] = self.registers[x as usize] % 10;
        self.pc_increse();
    }

    // Fx55 - LD [I], Vx: Store registers V0 through Vx in memory starting at location I.
    pub fn op_fx55(&mut self, x: u8) {
        for i in 0..(x as u16 + 1) {
            self.ram[(self.i + i) as usize] = self.registers[i as usize];
        }
        self.pc_increse();
    }

    // Fx65 - LD Vx, [I]: Read registers V0 through Vx from memory starting at location I.
    pub fn op_fx65(&mut self, x: u8) {
        for i in 0..(x as u16 + 1) {
            self.registers[i as usize] = self.ram[(self.i + i) as usize];
        }
        self.pc_increse();
    }
}

// #[cfg(test)]
// mod tests {

//     use chip8::*;

//     #[test]
//     fn op_00e0() {
//         let mut c8 = Chip8::new();
//         c8.op_00e0();
//         for x in 0..CHIP8_WIDTH as usize {
//             for y in 0..CHIP8_HEIGHT as usize {
//                 assert_eq!(c8.vram[y][x], 0);
//             }
//         }
//         assert!(c8.should_draw);
//         assert_eq!(c8.pc, 0x202);
//     }

//     #[test]
//     fn op_00ee() {
//         let mut c8 = Chip8::new();
//         c8.stack_pointer = 0x1;
//         c8.stack[0x1] = 0xCCC;
//         c8.op_00ee();
        
//         assert_eq!(c8.pc, 0xCCC);
//         assert_eq!(c8.stack_pointer, 0x0);
//     }
// }

