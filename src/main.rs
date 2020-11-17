use std::{env, thread};
use crate::processor::Processor;
use crate::display::Display;
use crate::input::Input;
use std::borrow::Borrow;
use std::time::Duration;

mod processor;
mod font;
mod display;
mod input;

fn main() {
    let sdl_context = match sdl2::init() {
        Ok(context) => context,
        Err(_) => panic!("Unable to start sdl context")
    };

    let mut display = Display::new(&sdl_context);
    let mut input = Input::new(&sdl_context);

    let args: Vec<String> = env::args().collect();
    let rom_filename = &args[1];

    let mut processor = Processor::new();
    processor.load_rom(String::from(rom_filename));

    while let Ok(keypad) = input.poll() {
        processor.tick(keypad);

        display.draw(processor.video.borrow());

        thread::sleep(Duration::from_millis(1));
    }

}
