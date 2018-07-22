use std::sync::{Arc, Mutex};
use hardware::{AudioDriver, KeyboardDriver};
use memory::Memory;
use cpu::Cpu;
use glutin_window::GlutinWindow as Window;
use piston::window::WindowSettings;
use piston::event_loop::{Events, EventSettings, EventLoop};
use opengl_graphics::{OpenGL, GlGraphics};
use piston::input::{RenderEvent, RenderArgs, Button, ButtonState, ButtonEvent, Key}; // self, Button, Event, Input, 
use graphics::{self, Transformed};

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

pub const OPCODE_SIZE: u16 = 2;
pub const CHIP8_WIDTH: usize = 64;
pub const CHIP8_HEIGHT: usize = 32;
pub const CLOCK_FREQ: u64 = 2;
pub const SCALE: usize = 10;

pub struct Chip8<A, K>
    where
        A: AudioDriver + Sync + Send,
        K: KeyboardDriver + Sync + Send,
{
    pub memory: Arc<Mutex<Memory>>,
    pub cpu: Cpu,
    pub keyboard: K,
    pub audio: A,
    pub clock: u64,
    pub gfx: GlGraphics,
    pub window: Window,
    pub program: Vec<u8>,
    pub debug: bool,
}

impl<A: 'static, K: 'static> Chip8<A, K>
    where
        A: AudioDriver + Sync + Send,
        K: KeyboardDriver + Sync + Send,
{
    pub fn new(audio: A, keyboard: K) -> Self {
        let memory = Arc::new(Mutex::new(Memory::new()));
        let cpu = Cpu::new(memory.clone());
        let clock = 1000 / CLOCK_FREQ;

        let opengl = OpenGL::V3_2;
        let width = CHIP8_WIDTH * SCALE;
        let height = CHIP8_HEIGHT * SCALE;

        let window: Window = WindowSettings::new("Chip8", [width as u32, height as u32])
            .opengl(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

        let gfx = GlGraphics::new(opengl);
        let program = Vec::new();
        let debug = false;

        Chip8 {
            memory,
            cpu,
            audio,
            keyboard,
            clock,
            gfx,
            window,
            debug,
            program,
        }
    }

    pub fn toggle_debuger(&mut self) {
        self.debug = !self.debug;
        self.cpu.debug = !self.cpu.debug;
    }


    pub fn load_program(&mut self, program: &[u8]) {
        if program.len() > (4096 - 512) {
            panic!("This program is too big to run in this interpreter");
        }
        self.program = program.to_vec();
        let mut memory = self.memory.lock().unwrap();
        for addr in 0..program.len() {
            let offset = 0x200;
            memory.ram[addr + offset] = program[addr];
            // if (addr + offset) & 1 == 0 && addr + 1 < program.len()  {
            //     let opcode = (program[addr] as u16) << 8 | (program[addr + 1] as u16);
            //     println!("0x{:X}: {:X}", 0x200 + addr, opcode);
            // }
        }

        println!("{:#} bytes loaded to memory..", program.len());
    }

    // pub fn render(&mut self, cxt: [[f64; 3]; 2], gfx: &mut GlGraphics) {
    pub fn render(&mut self, args: &RenderArgs) {
        let memory = self.memory.lock().unwrap();

        const BACKGROUND: [f32; 4] = [0.3, 0.3, 0.3, 1.0];
        const FOREGROUND: [f32; 4] = [1.0, 0.13, 0.43, 1.0];

        self.gfx.draw(args.viewport(), |_ctx, gfx| {
            graphics::clear(BACKGROUND, gfx);
        });


        let square = graphics::rectangle::square(0.0, 0.0, SCALE as f64);

        for y in 0..CHIP8_HEIGHT {
            for x in 0..CHIP8_WIDTH {
                let pos_x = x * SCALE;
                let pos_y = y * SCALE;
                if memory.vram[y][x] == 1 {
                    self.gfx.draw(args.viewport(), |c, gfx| {
                        graphics::rectangle(FOREGROUND, square, c.transform.trans(pos_x as f64, pos_y as f64), gfx);
                    })
                }
            }
        }
    }

    pub fn boot(&mut self) {
        println!("Booting Chip8..");

        let mut events = Events::new(EventSettings::new()).ups(self.clock);

        while let Some(e) = events.next(&mut self.window) {
            if let Some(args) = e.render_args() {
                self.render(&args);
            }

            if !self.debug {
                self.cpu.tick(&self.keyboard);
            }

            if let Some(k) = e.button_args() {
                if k.state == ButtonState::Press || k.state == ButtonState::Release {
                    let index = match &k.button {
                        &Button::Keyboard(Key::D1)  => Some(0x1),
                        &Button::Keyboard(Key::D2)  => Some(0x2),
                        &Button::Keyboard(Key::D3)  => Some(0x3),
                        &Button::Keyboard(Key::D4)  => Some(0xc),
                        &Button::Keyboard(Key::Q)   => Some(0x4),
                        &Button::Keyboard(Key::W)   => Some(0x5),
                        &Button::Keyboard(Key::E)   => Some(0x6),
                        &Button::Keyboard(Key::R)   => Some(0xd),
                        &Button::Keyboard(Key::A)   => Some(0x7),
                        &Button::Keyboard(Key::S)   => Some(0x8),
                        &Button::Keyboard(Key::D)   => Some(0x9),
                        &Button::Keyboard(Key::F)   => Some(0xe),
                        &Button::Keyboard(Key::Z)   => Some(0xa),
                        &Button::Keyboard(Key::X)   => Some(0x0),
                        &Button::Keyboard(Key::C)   => Some(0xb),
                        &Button::Keyboard(Key::V)   => Some(0xf),
                        &Button::Keyboard(Key::N)   => {
                            if k.state == ButtonState::Press && self.debug {
                                self.cpu.tick(&self.keyboard);
                            }
                            None
                        }
                        &Button::Keyboard(Key::P) if k.state == ButtonState::Press  => {
                            self.toggle_debuger();
                            None
                        }
                        &Button::Keyboard(Key::M) if k.state == ButtonState::Press  => {
                            let memory = self.memory.lock().unwrap();
                            for i in 0..memory.ram.len() {
                                println!("0x{:X}: {:X}", i, memory.ram[i]);
                            }
                            None
                        }
                        &Button::Keyboard(Key::O) if k.state == ButtonState::Press  => {
                            let program = &self.program;
                            for addr in 0..program.len() {
                                let offset = 0x200;
                                if (addr + offset) & 1 == 0 && addr + 1 < program.len()  {
                                    let opcode = (program[addr] as u16) << 8 | (program[addr + 1] as u16);
                                    println!("0x{:X}: {:X}", 0x200 + addr, opcode);
                                } 
                            }
                            None
                        }
                        _ => None,
                    };

                    if let Some(i) = index {
                        match k.state {
                            ButtonState::Press => self.keyboard.press(i),
                            ButtonState::Release => self.keyboard.release(i),
                        }
                    }
                }
            }
        }
    }
}