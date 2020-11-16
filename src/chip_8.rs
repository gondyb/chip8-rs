use rand::Rng;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use crate::font::FONTSET;

const ROM_START_ADDRESS: u16 = 0x200;
const FONTSET_START_ADDRESS: u8 = 0x50;

const REGISTERS_SIZE: usize = 16;
const MEMORY_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const KEYPAD_SIZE: usize = 16;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const SCREEN_RESOLUTION: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

const CARRY_REGISTER: u8 = 0xF;

const FONT_CHARACTER_BYTES: u8 = 5;


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
    fn new() -> Chip8 {
        let mut memory = [0; MEMORY_SIZE];

        for i in 0..FONTSET.len() {
            memory[FONTSET_START_ADDRESS as usize + i] = FONTSET[i];
        }

        Chip8{
            registers: [0; REGISTERS_SIZE],
            memory,
            index: 0,
            pc: ROM_START_ADDRESS,
            stack: [0; STACK_SIZE],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; KEYPAD_SIZE],
            video: [0; SCREEN_RESOLUTION],
            opcode: 0
        }
    }

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
            self.memory[ROM_START_ADDRESS as usize + i] = *rom_vector.get(i).expect("Unable to read from ROM vector");
        }
    }

    fn op_00e0(&mut self) {
        for mut i in self.video.iter() {
            i = &0;
        }
    }

    fn op_00ee(&mut self) {
        self.sp = self.sp - 1;
        self.pc = self.stack[self.sp as usize];
    }

    fn op_1nnn(&mut self) {
        self.pc = self.opcode & 0x0FFF;
    }

    fn op_2nnn(&mut self) {
        let new_instruction = self.opcode & 0x0FF;

        self.stack[self.sp as usize] = self.pc;
        self.sp = self.sp + 1;
        self.pc = new_instruction;
    }

    fn op_3xkk(&mut self) {
        let register_number = self.opcode & 0xF00 >> 8;
        let number = self.opcode & 0x0FF;
        if self.registers[register_number as usize] == number as u8 {
            self.pc = self.pc + 2;
        }
    }

    fn op_4xkk(&mut self) {
        let register_number = (self.opcode & 0xF00u16 >> 8);
        let number = (self.opcode & 0x0FFu16);
        if self.registers[register_number as usize] != number as u8 {
            self.pc = self.pc + 2;
        }
    }

    fn op_5xy0(&mut self) {
        let r1 = (self.opcode & 0xF00u16) >> 8;
        let r2 = (self.opcode & 0x0F0u16) >> 4;
        if self.registers[r1 as usize] == self.registers[r2 as usize] {
            self.pc = self.pc + 2;
        }
    }

    fn op_6xkk(&mut self) {
        let register = (self.opcode & 0xF00u16) >> 8;
        let value = self.opcode & 0x0FFu16;
        self.registers[register as usize] = value as u8;
    }

    fn op_7xkk(&mut self) {
        let register = (self.opcode & 0xF00u16) >> 8;
        let value = self.opcode & 0x0FFu16;
        self.registers[register as usize] = self.registers[register as usize] + value as u8;
    }

    fn op_8xy0(&mut self) {
        let r1 = (self.opcode & 0xF00u16) >> 8;
        let r2 = (self.opcode & 0x0F0u16) >> 4;
        self.registers[r1 as usize] = self.registers[r2 as usize];
    }

    fn op_8xy1(&mut self) {
        let r1 = (self.opcode & 0xF00u16) >> 8;
        let r2 = (self.opcode & 0x0F0u16) >> 4;
        self.registers[r1 as usize] = self.registers[r1 as usize] | self.registers[r2 as usize];
    }

    fn op_8xy2(&mut self) {
        let r1 = (self.opcode & 0xF00u16) >> 8;
        let r2 = (self.opcode & 0x0F0u16) >> 4;
        self.registers[r1 as usize] = self.registers[r1 as usize] & self.registers[r2 as usize];
    }

    fn op_8xy3(&mut self) {
        let r1 = (self.opcode & 0xF00u16) >> 8;
        let r2 = (self.opcode & 0x0F0u16) >> 4;
        self.registers[r1 as usize] = self.registers[r1 as usize] ^ self.registers[r2 as usize];
    }

    fn op_8xy4(&mut self) {
        let r1 = (self.opcode & 0xF00u16) >> 8;
        let r2 = (self.opcode & 0x0F0u16) >> 4;
        let sum = self.registers[r1 as usize] + self.registers[r2 as usize];

        self.registers[CARRY_REGISTER as usize] = u8::from(sum > 0xFF);

        self.registers[r1 as usize] = sum & 0xFF;
    }

    fn op_8xy5(&mut self) {
        let r1 = (self.opcode & 0xF00u16) >> 8;
        let r2 = (self.opcode & 0x0F0u16) >> 4;
        let sub = self.registers[r1 as usize] - self.registers[r2 as usize];

        self.registers[CARRY_REGISTER as usize] = u8::from(sub > 0);

        self.registers[r1 as usize] = sub;
    }

    fn op_8xy6(&mut self) {
        let r1 = (self.opcode & 0xF00u16) >> 8;

        self.registers[CARRY_REGISTER as usize] = self.registers[r1 as usize] & 0x1;

        self.registers[r1 as usize] = self.registers[r1 as usize] >> 1;
    }

    fn op_8xy7(&mut self) {
        let r1 = (self.opcode & 0xF00u16) >> 8;
        let r2 = (self.opcode & 0x0F0u16) >> 4;
        let sub = self.registers[r2 as usize] - self.registers[r1 as usize];

        self.registers[CARRY_REGISTER as usize] = u8::from(sub > 0);

        self.registers[r1 as usize] = sub;
    }

    fn op_8xye(&mut self) {
        let r1 = (self.opcode & 0xF00u16) >> 8;

        self.registers[CARRY_REGISTER as usize] = self.registers[r1 as usize] & 0x80 >> 7;

        self.registers[r1 as usize] = self.registers[r1 as usize] << 1;
    }


    fn op_9xy0(&mut self) {
        let r1 = (self.opcode & 0xF00u16) >> 8;
        let r2 = (self.opcode & 0x0F0u16) >> 4;

        if self.registers[r1 as usize] != self.registers[r2 as usize] {
            self.pc = self.pc + 2;
        }
    }

    fn op_annn(&mut self) {
        self.index = self.opcode & 0x0FFF;
    }

    fn op_bnnn(&mut self) {
        let address = self.opcode & 0x0FFF;
        self.pc = self.registers[0] as u16 + address;
    }

    fn op_cxkk(&mut self) {
        let mut rng = rand::thread_rng();
        let rnum: u8 = rng.gen();

        let register = self.opcode & 0xF00u16 >> 8;
        let byte = self.opcode & 0x0FFu16;
        self.registers[register as usize] = rnum & byte as u8;
    }

    fn op_dxyn(&mut self) {
        let sprite_length = (self.opcode & 0xF) as u8;
        let vx = self.opcode & 0xF00 >> 8;
        let vy = self.opcode & 0x0F0 >> 4;

        let x_pos = self.registers[vx as usize] % SCREEN_WIDTH as u8;
        let y_pos = self.registers[vy as usize] % SCREEN_HEIGHT as u8;

        self.registers[CARRY_REGISTER as usize] = 0;

        for row in 0..sprite_length {
            let sprite_byte = self.memory[(self.index + row as u16) as usize];

            for col in 0..8 {
                let sprite_pixel = sprite_byte & (0x80 >> col);
                let mut screen_pixel = self.video[((y_pos + row) * SCREEN_WIDTH as u8 + (x_pos + col)) as usize];

                if sprite_pixel > 0 {
                    if screen_pixel == 0xFFFFFFFF {
                        self.registers[CARRY_REGISTER as usize] = 1;
                    }

                    screen_pixel = screen_pixel ^ 0xFFFFFFFF;
                }
            }
        }
    }

    fn op_ex9e(&mut self) {
        let vx = (self.opcode & 0xF00) >> 8;
        let key = self.registers[vx as usize];

        if self.keypad[key as usize] > 0{
            self.pc = self.pc + 2;
        }
    }

    fn op_exa1(&mut self) {
        let vx = (self.opcode & 0xF00) >> 8;
        let key = self.registers[vx as usize];

        if self.keypad[key as usize] == 0 {
            self.pc = self.pc + 2;
        }
    }

    fn op_fx07(&mut self) {
        let vx = (self.opcode & 0xF00) >> 8;
        self.registers[vx as usize] = self.delay_timer;
    }

    fn op_fx0a(&mut self) {
        let vx = (self.opcode & 0xF00) >> 8;

        let mut found = false;

        for i in 0..16 {
            if self.keypad[i as usize] > 0 {
                self.registers[vx as usize] = i;
                found = true;
                break;
            }
        }

        if !found {
            self.pc = self.pc - 2;
        }
    }

    fn op_fx15(&mut self) {
        let vx = (self.opcode & 0xF00) >> 8;

        self.delay_timer = self.registers[vx as usize];
    }

    fn op_fx18(&mut self) {
        let vx = (self.opcode & 0xF00) >> 8;

        self.sound_timer = self.registers[vx as usize];
    }

    fn op_fx1e(&mut self) {
        let vx = (self.opcode & 0xF00) >> 8;

        self.index = self.index + self.registers[vx as usize] as u16;
    }

    fn op_fx29(&mut self) {
        let vx = (self.opcode & 0xF00) >> 8;
        let digit = self.registers[vx as usize];

        self.index = (FONTSET_START_ADDRESS + FONT_CHARACTER_BYTES * digit) as u16;
    }

    fn op_fx33(&mut self) {
        let vx = (self.opcode & 0xF00) >> 8;

        self.memory[self.index as usize] = (vx / 100) as u8;
        self.memory[self.index as usize + 1] = ((vx % 100) / 10) as u8;
        self.memory[self.index as usize + 2] = (vx % 10) as u8;
    }

    fn op_fx55(&mut self) {
        let vx = (self.opcode & 0xF00) >> 8;

        for i in 0..vx {
            self.memory[(self.index + 1) as usize] = self.registers[i as usize];
        }
    }

    fn op_fx65(&mut self) {
        let vx = (self.opcode & 0xF00) >> 8;

        for i in 0..vx {
            self.registers[i as usize] = self.memory[(self.index + i) as usize];
        }
    }

}