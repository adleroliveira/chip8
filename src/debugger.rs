use cpu::Cpu;
use memory::Memory;
use std::sync::{Arc, Mutex};
use piston::input::{Button, Key};
use hardware::KeyboardDriver;

#[derive(PartialEq, Debug)]
pub enum DebugMode {
    Disabled,
    Step,
    CpuInfo,
    OpcodeInfo
}

pub struct Debugger {
    pub cpu: Arc<Mutex<Cpu>>,
    pub memory: Arc<Mutex<Memory>>,
    pub program: Vec<u8>,
    pub mode: DebugMode,
}

impl Debugger {
    pub fn new(cpu: Arc<Mutex<Cpu>>, memory: Arc<Mutex<Memory>>) -> Self {
        Debugger {
            cpu,
            memory,
            program: Vec::new(),
            mode: DebugMode::Disabled
        }
    }

    pub fn debug(&self, cpu: &Cpu, opcode: u16) {
        match self.mode {
            DebugMode::Step => {
                // let cpu = self.cpu.lock().unwrap();
                println!("{:?} - 0x{:X}: {:?}", cpu.steps, cpu.pc, Self::describe_opcode(opcode));
            }
            
            DebugMode::CpuInfo => {
                // let cpu = self.cpu.lock().unwrap();
                // print!("{}[2J", 27 as char);
                println!("======================================================");
                println!("ADDR: 0x{:X} | OPCODE: 0x{:X}: {:?}", cpu.pc, opcode, Self::describe_opcode(opcode));
                for i in 0..16 {
                    println!("V{:?}: 0x{:X} ({:?})", i, cpu.v[i], cpu.v[i]);
                }
                println!("I: 0x{:X} ({:?})", cpu.i, cpu.i);
                println!("PC: 0x{:X} ({:?})", cpu.pc, cpu.pc);
                println!("SP: 0x{:X} ({:?})", cpu.sp, cpu.sp);
                println!("DT: 0x{:X} ({:?})", cpu.delay_timer, cpu.delay_timer);
            }

            _ => {} // TODO: Finish this
        }               
    }

    pub fn input_key<K>(&mut self, key: &Button, keyboard: &K)
        where K: KeyboardDriver + Sync + Send
    {
        match key {
            &Button::Keyboard(Key::D0) => {
                self.mode = DebugMode::Disabled;
                println!("DEBUG MODE: {:?}", self.mode);
            }

            &Button::Keyboard(Key::D9) => {
                self.mode = DebugMode::Step;
                println!("DEBUG MODE: {:?}", self.mode);
            }

            &Button::Keyboard(Key::D8) => {
                self.mode = DebugMode::CpuInfo;
                println!("DEBUG MODE: {:?}", self.mode);
            }

            &Button::Keyboard(Key::D7) => {
                self.mode = DebugMode::OpcodeInfo;
                println!("DEBUG MODE: {:?}", self.mode);
            }

            &Button::Keyboard(Key::N) => {
                if self.mode == DebugMode::Step {
                    let mut cpu = self.cpu.lock().unwrap();
                    cpu.tick(keyboard, &self);
                }
            }

            &Button::Keyboard(Key::I) => {
                if self.mode == DebugMode::Step {
                    let mut cpu = self.cpu.lock().unwrap();
                    cpu.tick(keyboard, &self);
                    println!("======================================================");
                    for i in 0..16 {
                        println!("V{:?}: 0x{:X} ({:?})", i, cpu.v[i], cpu.v[i]);
                    }
                    println!("I: 0x{:X} ({:?})", cpu.i, cpu.i);
                    println!("PC: 0x{:X} ({:?})", cpu.pc, cpu.pc);
                    println!("SP: 0x{:X} ({:?})", cpu.sp, cpu.sp);
                    println!("DT: 0x{:X} ({:?})", cpu.delay_timer, cpu.delay_timer);
                }
            }

            &Button::Keyboard(Key::L) => {
                if self.mode == DebugMode::Step {
                    let mut cpu = self.cpu.lock().unwrap();
                    for i in 0..16 {
                        println!("V{:?}: 0x{:X} ({:?})", i, cpu.v[i], cpu.v[i]);
                    }
                    println!("I: 0x{:X} ({:?})", cpu.i, cpu.i);
                    println!("PC: 0x{:X} ({:?})", cpu.pc, cpu.pc);
                    println!("SP: 0x{:X} ({:?})", cpu.sp, cpu.sp);
                }
            }

            &Button::Keyboard(Key::M) => {
                if self.mode == DebugMode::Step {
                    let memory = self.memory.lock().unwrap();
                    for i in 0..memory.ram.len() {
                        println!("0x{:X}: {:X}", i, memory.ram[i]);
                    }
                }
            }

            &Button::Keyboard(Key::O) => {
                if self.mode == DebugMode::Step {
                    let program = &self.program;
                    for addr in 0..program.len() {
                        let offset = 0x200;
                        if (addr + offset) & 1 == 0 && addr + 1 < program.len()  {
                            let opcode = (program[addr] as u16) << 8 | (program[addr + 1] as u16);
                            println!("0x{:X}: {:?}", 0x200 + addr, Self::describe_opcode(opcode));
                        } 
                    }
                }
            }

            _ => {}
        }
    }

    pub fn describe_opcode(opcode: u16) -> String {
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

        match nibbles {
            (0x0, 0x0, 0x0, 0x0)    => String::from("HALT"),
            (0x0, 0x0, 0xE, 0x0)    => String::from("CLS"),
            (0x0, 0x0, 0xE, 0xE)    => String::from("RET"),
            (0x1, _, _, _)          => format!("JP 0x{:X} ({:?})", nnn, nnn),           // JP addr
            (0x2, _, _, _)          => format!("CALL 0x{:X} ({:?})", nnn, nnn),           // CALL addr
            (0x3, _, _, _)          => format!("SE V{:?} 0x{:X} ({:?})", x, kk, kk),         // SE Vx, byte
            (0x4, _, _, _)          => format!("SNE V{:?} 0x{:X} ({:?})", x, kk, kk),         // SNE Vx, byte
            (0x5, _, _, 0x00)       => format!("SE V{:?} V{:?}", x, y),          // SE Vx, Vy
            (0x6, _, _, _)          => format!("LD V{:?} 0x{:X} ({:?})", x, kk, kk),         // LD Vx, byte
            (0x7, _, _, _)          => format!("ADD V{:?} 0x{:X} ({:?})", x, kk, kk),         // ADD Vx, byte
            (0x8, _, _, 0x00)       => format!("LD V{:?} V{:?}", x, y),          // LD Vx, Vy
            (0x8, _, _, 0x01)       => format!("OR V{:?} V{:?}", x, y),          // OR Vx, Vy
            (0x8, _, _, 0x02)       => format!("AND V{:?} V{:?}", x, y),          // AND Vx, Vy
            (0x8, _, _, 0x03)       => format!("XOR V{:?} V{:?}", x, y),          // XOR Vx, Vy
            (0x8, _, _, 0x04)       => format!("ADD V{:?} V{:?}", x, y),          // ADD Vx, Vy
            (0x8, _, _, 0x05)       => format!("SUB V{:?} V{:?}", x, y),          // SUB Vx, Vy
            (0x8, _, _, 0x06)       => format!("SHR V{:?} [, V{:?}]", x, y),          // SHR Vx {, Vy}
            (0x8, _, _, 0x07)       => format!("SUBN V{:?} V{:?}", x, y),          // SUBN Vx, Vy
            (0x8, _, _, 0x0E)       => format!("SHL V{:?} [, V{:?}]", x, y),          // SHL Vx {, Vy}
            (0x9, _, _, 0x00)       => format!("SNE V{:?}, V{:?}", x, y),          // SNE Vx, Vy
            (0xA, _, _, _)          => format!("LDI, 0x{:X} ({:?})", nnn, nnn),           // LDI, addr
            (0xB, _, _, _)          => format!("JP V0, {:?} ({:?})", nnn, nnn),           // JP V0, addr
            (0xC, _, _, _)          => format!("RND V{:?}, 0x{:X} ({:?})", x, kk, kk),         // RND Vx, byte
            (0xD, _, _, _)          => format!("DRW V{:?}, V{:?}, 0x{:X} ({:X})", x, y, n, n),       // DRW Vx, Vy, nibble
            (0xE, _, 0x09, 0x0E)    => format!("SKP V{:?}", x),   // SKP Vx
            (0xE, _, 0x0A, 0x01)    => format!("SKNP V{:?}", x),   // SKNP Vx
            (0xF, _, 0x00, 0x07)    => format!("LD V{:?}, DT", x),         // LD Vx, DT
            (0xF, _, 0x00, 0x0A)    => format!("LD V{:?}, K", x),         // LD Vx, K
            (0xF, _, 0x01, 0x05)    => format!("LD DT, V{:?}", x),         // LD DT, Vx
            (0xF, _, 0x01, 0x08)    => format!("LD ST, V{:?}", x),         // LD ST, Vx
            (0xF, _, 0x01, 0x0E)    => format!("ADD I, V{:?}", x),         // ADD I, Vx
            (0xF, _, 0x02, 0x09)    => format!("LD F, V{:?}", x),         // LD F, Vx
            (0xF, _, 0x03, 0x03)    => format!("LD B, V{:?}", x),         // LD B, Vx
            (0xF, _, 0x05, 0x05)    => format!("LD [I], V{:?}", x),         // LD [I], Vx
            (0xF, _, 0x06, 0x05)    => format!("LD V{:?}, [I]", x),         // LD Vx, [I]
            _ => format!("0x{:X}", opcode)
        }
    }
}