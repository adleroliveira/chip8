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

    vm.load_program(&rom);
    vm.boot();
}