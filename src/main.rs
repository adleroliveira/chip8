extern crate chip8;
use std::io::Read;
use std::path::Path;
use std::env;
use std::process;
use std::fs::File;
use chip8::Chip8;
use chip8::drivers::{Keyboard, Audio};


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.get(1).is_none() {
        eprintln!("{}", "Usage: chip8 /path/to/program.rom");
        process::exit(1);
    }

    let filepath = &args[1];
    let filepath = Path::new(filepath);

    let mut f = File::open(filepath).expect("file not found");
    let mut rom = Vec::new();

    f.read_to_end(&mut rom).expect("failed to read .rom file");
    let mut vm = Chip8::new(Audio {}, Keyboard::new());
    vm.toggle_debuger();
    vm.load_program(&rom);
    vm.boot();
}



// use chip8::chip8_old::Chip8;
// extern crate glutin_window;
// extern crate piston;
// extern crate opengl_graphics;
// extern crate graphics;

// use glutin_window::GlutinWindow;
// use piston::window::WindowSettings;
// use piston::event_loop::{Events, EventSettings, EventLoop};
// use piston::input::*;
// use opengl_graphics::{OpenGL, GlGraphics};

// struct Game {
//     gl: GlGraphics,
//     pub chip8: Chip8,
//     pub rom: Vec<u8>,
//     pub keys_pressed: [bool; 16]
// }

// impl Game {
//     fn render(&mut self, args: &RenderArgs) {
//         use graphics;

//         const GREY: [f32; 4] = [0.3, 0.3, 0.3, 1.0];
//         const BLACK: [f32; 4] = [1.0, 0.13, 0.43, 1.0];

//         self.gl.draw(args.viewport(), |_c, gl| {
//             graphics::clear(GREY, gl);
//         });

//         let width = 10;

//         for y in 0..self.chip8.vram.len() {
//             for x in 0..self.chip8.vram[y as usize].len() {
//                 let pos_x = x * width;
//                 let pos_y = y * width;
//                 if self.chip8.vram[y][x] == 1 {
//                     let square = graphics::rectangle::square(pos_x as f64, pos_y as f64, width as f64);
//                     self.gl.draw(args.viewport(), |c, gl| {
//                         graphics::rectangle(BLACK, square, c.transform, gl)
//                     });
//                 }
//             }
//         }
//     }

//     pub fn load(&mut self) {
//         self.chip8.load_program(&self.rom);
//     }
// }

// fn main() {
//     let opengl = OpenGL::V3_2;

//     let args: Vec<String> = env::args().collect();

//     if args.get(1).is_none() {
//         eprintln!("{}", "Usage: chip8 /path/to/program.rom");
//         process::exit(1);
//     }

//     let filepath = &args[1];
//     let filepath = Path::new(filepath);

//     let mut f = File::open(filepath).expect("file not found");
//     let mut rom = Vec::new();

//     f.read_to_end(&mut rom).expect("failed to read .rom file");

//     // c8.load_program(&contents);

//     const COLS: u32 = 64;
//     const ROWS: u32 = 32;
//     const SQUARE_WIDTH: u32 = 10;

//     let width = COLS * SQUARE_WIDTH;
//     let height = ROWS * SQUARE_WIDTH;

//     let mut window: GlutinWindow = WindowSettings::new("Chip8", [width, height])
//         .opengl(opengl)
//         .exit_on_esc(true)
//         .build()
//         .unwrap();

//     let mut game = Game {
//         gl: GlGraphics::new(opengl),
//         chip8: Chip8::new(),
//         rom: rom,
//         keys_pressed: [false; 16]
//     };

//     game.load();

//     let mut events = Events::new(EventSettings::new()).ups(350);

//     while let Some(e) = events.next(&mut window) {
//         {
//             game.chip8.tick(game.keys_pressed);
//         }

//         if let Some(r) = e.render_args() {
//             game.render(&r);
//         }

//         if let Some(_u) = e.update_args() {

//             // game.keys_pressed = [false; 16];
//             // println!("{:?}", output.should_draw);
//         }

//         if let Some(k) = e.button_args() {
//             if k.state == ButtonState::Press {
//                 // println!("Button pressed {:?}", &k.button);
//                 let index = match &k.button {
//                     &Button::Keyboard(Key::D1)  => Some(0x1),
//                     &Button::Keyboard(Key::D2)  => Some(0x2),
//                     &Button::Keyboard(Key::D3)  => Some(0x3),
//                     &Button::Keyboard(Key::D4)  => Some(0xc),
//                     &Button::Keyboard(Key::Q)   => Some(0x4),
//                     &Button::Keyboard(Key::W)   => Some(0x5),
//                     &Button::Keyboard(Key::E)   => Some(0x6),
//                     &Button::Keyboard(Key::R)   => Some(0xd),
//                     &Button::Keyboard(Key::A)   => Some(0x7),
//                     &Button::Keyboard(Key::S)   => Some(0x8),
//                     &Button::Keyboard(Key::D)   => Some(0x9),
//                     &Button::Keyboard(Key::F)   => Some(0xe),
//                     &Button::Keyboard(Key::Z)   => Some(0xa),
//                     &Button::Keyboard(Key::X)   => Some(0x0),
//                     &Button::Keyboard(Key::C)   => Some(0xb),
//                     &Button::Keyboard(Key::V)   => Some(0xf),
//                     _ => None,
//                 };

//                 if let Some(i) = index {
//                     game.keys_pressed[i] = true;
//                 }
//             } else if k.state == ButtonState::Release {
//                 // println!("Button released {:?}", &k.button);
//                 let index = match &k.button {
//                     &Button::Keyboard(Key::D1)  => Some(0x1),
//                     &Button::Keyboard(Key::D2)  => Some(0x2),
//                     &Button::Keyboard(Key::D3)  => Some(0x3),
//                     &Button::Keyboard(Key::D4)  => Some(0xc),
//                     &Button::Keyboard(Key::Q)   => Some(0x4),
//                     &Button::Keyboard(Key::W)   => Some(0x5),
//                     &Button::Keyboard(Key::E)   => Some(0x6),
//                     &Button::Keyboard(Key::R)   => Some(0xd),
//                     &Button::Keyboard(Key::A)   => Some(0x7),
//                     &Button::Keyboard(Key::S)   => Some(0x8),
//                     &Button::Keyboard(Key::D)   => Some(0x9),
//                     &Button::Keyboard(Key::F)   => Some(0xe),
//                     &Button::Keyboard(Key::Z)   => Some(0xa),
//                     &Button::Keyboard(Key::X)   => Some(0x0),
//                     &Button::Keyboard(Key::C)   => Some(0xb),
//                     &Button::Keyboard(Key::V)   => Some(0xf),
//                     _ => None,
//                 };

//                 if let Some(i) = index {
//                     game.keys_pressed[i] = false;
//                 }
//             }
//         }

//     }
// }
    // Dump memory to screen
    // for i in 0..c8.ram.len() {
    //     println!("0x{:X} 0x{:X}", i, c8.ram[i]);
    // }

    // c8.op_annn(0x00);
    // c8.op_dxyn(0x00, 0x00, 0x05);
    // draw(&c8.vram);


    // loop {
    //     // let mut buffer = Vec::new();
    //     // let _ = io::stdin().read(&mut buffer);
    //     // let _input: Option<u8> = std::io::stdin()
    //     //     .bytes() 
    //     //     .next()
    //     //     .and_then(|result| result.ok());
    //     // let output = c8.tick([false; 16]);
    //     // if output.should_draw {
    //     //     print!("{}[2J", 27 as char);
    //     //     draw(output.vram);
    //     // }
    //     thread::sleep(sleep_duration);
    //     // print!("{}[2J", 27 as char);
    // }


// fn draw(vram: &[[u8; 64]; 32]) {
//     for y in 0..vram.len() {
//         for x in 0..vram[y as usize].len() {
//             print!("{}", if vram[y][x] == 1 { "█" } else { " " });
//         }
//         print!("\n");
//     }
//     let _ = std::io::stdout().flush();
// }
