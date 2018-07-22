// use hardware::VideoDriver;
// use chip8::CHIP8_WIDTH;
// use chip8::CHIP8_HEIGHT;
// use std::io;
// use std::io::Write;

// extern crate glutin_window;
// extern crate piston;
// extern crate opengl_graphics;
// extern crate graphics;

// use glutin_window::GlutinWindow;
// use piston::window::WindowSettings;
// use piston::event_loop::{Events, EventSettings, EventLoop};
// // use piston::input::*;
// use opengl_graphics::{OpenGL, GlGraphics};


// const SQUARE_WIDTH: usize = 10;

// pub struct Video {
//     gl: GlGraphics,
// }
// impl VideoDriver for Video {
//     fn draw(&self, _frame_buffer: &[[u8; CHIP8_WIDTH]; CHIP8_HEIGHT], _width: usize, _height: usize) {
//         println!("DRAW");
//         // for y in 0..height {
//         //     for x in 0..width {
//         //         print!("{}", if frame_buffer[y][x] == 1 { "█" } else { " " });
//         //     }
//         //     let _ = io::stdout().flush();
//         //     println!("");
//         // }
//         // let _ = io::stdout().flush();
//     }
// }

// impl Video {
//     pub fn new() -> Video {
//         let opengl = OpenGL::V3_2;
//         let width = CHIP8_WIDTH * SQUARE_WIDTH;
//         let height = CHIP8_HEIGHT * SQUARE_WIDTH;

//         let mut window: GlutinWindow = WindowSettings::new("Chip8", [width as u32, height as u32])
//             .opengl(opengl)
//             .exit_on_esc(true)
//             .build()
//             .unwrap();

//         let video = Video {
//             gl: GlGraphics::new(opengl),
//         };

//         let mut events = Events::new(EventSettings::new()).ups(60);

//         while let Some(e) = events.next(&mut window) {
//             println!("{:?}", e);
//         }

//         video
//     }
// }