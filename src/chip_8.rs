use std::path::Path;
use std::fs::File;
use std::io::Read;
use crate::font::FONTSET;

const ROM_START_ADDRESS: usize = 0x200;
const FONTSET_START_ADDRESS: usize = 0x50;

const REGISTERS_SIZE: usize = 16;
const MEMORY_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const KEYPAD_SIZE: usize = 16;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const SCREEN_RESOLUTION: usize = SCREEN_WIDTH * SCREEN_HEIGHT;


pub struct Chip8 {
    registers: [u8; REGISTERS_SIZE],
    memory: [u8; MEMORY_SIZE],
    index: u16,
    pc: u16,
    stack: [u16; STACK_SIZE],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; KEYPAD_SIZE],
    video: [u32; SCREEN_RESOLUTION],
    opcode: u16
}

impl Chip8 {
    fn load_rom(&mut self, filename: String) {
        let path = Path::new(&filename);

        let mut file = match File::open(&path) {
            Err(why) => panic!("Unable to open rom {}: {}", path.display(), why),
            Ok(file) => file,
        };

        let mut rom_vector: Vec<u8> = Vec::new();

        match file.read_to_end(&mut rom_vector) {
            Ok(_) => {},
            Err(why) => panic!("Unable to read rom: {}", why)
        }

        for i in 0..rom_vector.len() {
            self.memory[ROM_START_ADDRESS + i] = *rom_vector.get(i).expect("Unable to read from ROM vector");
        }
    }

    fn new() -> Chip8 {
        let mut memory = [0; MEMORY_SIZE];

        for i in 0..FONTSET.len() {
            memory[FONTSET_START_ADDRESS + i] = FONTSET[i];
        }

        Chip8{
            registers: [0; REGISTERS_SIZE],
            memory,
            index: 0,
            pc: ROM_START_ADDRESS as u16,
            stack: [0; STACK_SIZE],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; KEYPAD_SIZE],
            video: [0; SCREEN_RESOLUTION],
            opcode: 0
        }
    }
}