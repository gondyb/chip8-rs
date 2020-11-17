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

const CARRY_REGISTER: usize = 0xF;

const FONT_CHARACTER_BYTES: u8 = 5;


pub struct Processor {
    registers: [u8; REGISTERS_SIZE],
    memory: [u8; MEMORY_SIZE],
    index: u16,
    pc: u16,
    stack: [u16; STACK_SIZE],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool; KEYPAD_SIZE],
    pub(crate) video: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    opcode: u16
}

impl Processor {
    pub fn new() -> Processor {
        let mut memory = [0; MEMORY_SIZE];

        for i in 0..FONTSET.len() {
            memory[FONTSET_START_ADDRESS as usize + i] = FONTSET[i];
        }

        Processor {
            registers: [0; REGISTERS_SIZE],
            memory,
            index: 0,
            pc: ROM_START_ADDRESS,
            stack: [0; STACK_SIZE],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; KEYPAD_SIZE],
            video: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            opcode: 0
        }
    }

    pub fn load_rom(&mut self, filename: String) {
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

    pub fn tick(&mut self, keypad: [bool; KEYPAD_SIZE]) {
        self.keypad = keypad;

        if self.delay_timer > 0 {
            self.delay_timer -= 1
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1
        }

        self.opcode = self.get_opcode();

        self.pc += 2;

        self.run_opcode();
    }

    fn get_opcode(&self) -> u16 {
        (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16)
    }

    fn run_opcode(&mut self) {
        let params = (
            (self.opcode & 0xF000) >> 12 as u8,
            (self.opcode & 0x0F00) >> 8 as u8,
            (self.opcode & 0x00F0) >> 4 as u8,
            (self.opcode & 0x000F) as u8
        );

        let nnn = (self.opcode & 0x0FFF) as u16;
        let kk = (self.opcode & 0x00FF) as u8;
        let x = params.1 as usize;
        let y = params.2 as usize;
        let n = params.3 as usize;

        match params {
            (0x00, 0x00, 0x0e, 0x00) => self.op_00e0(),
            (0x00, 0x00, 0x0e, 0x0e) => self.op_00ee(),
            (0x01, _, _, _) => self.op_1nnn(nnn),
            (0x02, _, _, _) => self.op_2nnn(nnn),
            (0x03, _, _, _) => self.op_3xkk(x, kk),
            (0x04, _, _, _) => self.op_4xkk(x, kk),
            (0x05, _, _, 0x00) => self.op_5xy0(x, y),
            (0x06, _, _, _) => self.op_6xkk(x, kk),
            (0x07, _, _, _) => self.op_7xkk(x, kk),
            (0x08, _, _, 0x00) => self.op_8xy0(x, y),
            (0x08, _, _, 0x01) => self.op_8xy1(x, y),
            (0x08, _, _, 0x02) => self.op_8xy2(x, y),
            (0x08, _, _, 0x03) => self.op_8xy3(x, y),
            (0x08, _, _, 0x04) => self.op_8xy4(x, y),
            (0x08, _, _, 0x05) => self.op_8xy5(x, y),
            (0x08, _, _, 0x06) => self.op_8xy6(x),
            (0x08, _, _, 0x07) => self.op_8xy7(x, y),
            (0x08, _, _, 0x0e) => self.op_8xye(x),
            (0x09, _, _, 0x00) => self.op_9xy0(x, y),
            (0x0a, _, _, _) => self.op_annn(nnn),
            (0x0b, _, _, _) => self.op_bnnn(nnn),
            (0x0c, _, _, _) => self.op_cxkk(x, kk),
            (0x0d, _, _, _) => self.op_dxyn(x, y, n),
            (0x0e, _, 0x09, 0x0e) => self.op_ex9e(x),
            (0x0e, _, 0x0a, 0x01) => self.op_exa1(x),
            (0x0f, _, 0x00, 0x07) => self.op_fx07(x),
            (0x0f, _, 0x00, 0x0a) => self.op_fx0a(x),
            (0x0f, _, 0x01, 0x05) => self.op_fx15(x),
            (0x0f, _, 0x01, 0x08) => self.op_fx18(x),
            (0x0f, _, 0x01, 0x0e) => self.op_fx1e(x),
            (0x0f, _, 0x02, 0x09) => self.op_fx29(x),
            (0x0f, _, 0x03, 0x03) => self.op_fx33(x),
            (0x0f, _, 0x05, 0x05) => self.op_fx55(x),
            (0x0f, _, 0x06, 0x05) => self.op_fx65(x),
            _ => panic!("Unknown instruction")
        }
    }

    // Clear the display.
    fn op_00e0(&mut self) {
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                self.video[y][x] = 0;
            }
        }
    }

    // Return from a subroutine.
    fn op_00ee(&mut self) {
        self.sp = self.sp - 1;
        self.pc = self.stack[self.sp as usize];
    }

    // Jump to location nnn.
    // The interpreter sets the program counter to nnn.
    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    // Call subroutine at nnn.
    fn op_2nnn(&mut self, nnn: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.sp = self.sp + 1;
        self.pc = nnn;
    }

    // Skip next instruction if Vx = kk.
    fn op_3xkk(&mut self, x: usize, kk: u8) {
        if self.registers[x] == kk {
            self.pc = self.pc + 2;
        }
    }

    // Skip next instruction if Vx != kk.
    fn op_4xkk(&mut self, x: usize, kk: u8) {
        if self.registers[x] != kk {
            self.pc = self.pc + 2;
        }
    }

    // Skip next instruction if Vx = Vy.
    fn op_5xy0(&mut self, x: usize, y: usize) {
        if self.registers[x] == self.registers[y] {
            self.pc = self.pc + 2;
        }
    }

    // Set Vx = kk.
    fn op_6xkk(&mut self, x: usize, kk: u8) {
        self.registers[x] = kk;
    }

    // Set Vx = Vx + kk.
    fn op_7xkk(&mut self, x: usize, kk: u8) {
        let vx = self.registers[x] as u16;
        let val = kk as u16;
        let result = vx + val;
        self.registers[x] = result as u8;
    }

    // Set Vx = Vy.
    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[y];
    }

    // Set Vx = Vx OR Vy.
    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[x] | self.registers[y];
    }

    // Set Vx = Vx AND Vy.
    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[x] & self.registers[y];
    }

    // Set Vx = Vx XOR Vy.
    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[x] ^ self.registers[y];
    }

    // Set Vx = Vx + Vy, set VF = carry.
    // The values of Vx and Vy are added together.
    // If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0.
    // Only the lowest 8 bits of the result are kept, and stored in Vx.
    fn op_8xy4(&mut self, x: usize, y: usize) {
        let vx = self.registers[x] as u16;
        let vy = self.registers[y] as u16;
        let result = vx + vy;
        self.registers[x] = result as u8;
        self.registers[CARRY_REGISTER] = if result > 0xFF { 1 } else { 0 };
    }

    // Set Vx = Vx - Vy, set VF = NOT borrow.
    // If Vx > Vy, then VF is set to 1, otherwise 0.
    // Then Vy is subtracted from Vx, and the results stored in Vx.
    fn op_8xy5(&mut self, x: usize, y: usize) {
        self.registers[CARRY_REGISTER] = if self.registers[x] > self.registers[y] { 1 } else { 0 };
        self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
    }

    // Set Vx = Vx SHR 1.
    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0.
    // Then Vx is divided by 2.
    fn op_8xy6(&mut self, x: usize) {
        self.registers[CARRY_REGISTER] = self.registers[x] & 0x1;

        self.registers[x] = self.registers[x] >> 1;
    }

    // Set Vx = Vy - Vx, set VF = NOT borrow.
    // If Vy > Vx, then VF is set to 1, otherwise 0.
    // Then Vx is subtracted from Vy, and the results stored in Vx.
    fn op_8xy7(&mut self, x: usize, y: usize) {
        self.registers[CARRY_REGISTER] = if self.registers[y] > self.registers[x] { 1 } else { 0 };
        self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
    }

    // Set Vx = Vx SHL 1.
    // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0.
    // Then Vx is multiplied by 2.
    fn op_8xye(&mut self, x: usize) {
        self.registers[CARRY_REGISTER] = (self.registers[x] & 0b10000000) >> 7;
        self.registers[x] <<= 1;
    }

    // Skip next instruction if Vx != Vy.
    fn op_9xy0(&mut self, x: usize, y: usize) {
        if self.registers[x] != self.registers[y] {
            self.pc = self.pc + 2;
        }
    }

    // Set I = nnn.
    fn op_annn(&mut self, nnn: u16) {
        self.index = nnn
    }

    // Jump to location nnn + V0.
    fn op_bnnn(&mut self, nnn: u16) {
        self.pc = self.registers[0] as u16 + nnn;
    }

    // Set Vx = random byte AND kk.
    fn op_cxkk(&mut self, x: usize, kk: u8) {
        let mut rng = rand::thread_rng();
        let rnum: u8 = rng.gen();

        self.registers[x] = rnum & kk;
    }

    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    // The interpreter reads n bytes from memory, starting at the address
    // stored in I. These bytes are then displayed as sprites on screen at
    // coordinates (Vx, Vy). Sprites are XORed onto the existing screen.
    // If this causes any pixels to be erased, VF is set to 1, otherwise
    // it is set to 0. If the sprite is positioned so part of it is outside
    // the coordinates of the display, it wraps around to the opposite side
    // of the screen.
    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) {
        self.registers[CARRY_REGISTER] = 0;
        for byte in 0..n {
            let y = (self.registers[y] as usize + byte) % SCREEN_HEIGHT;
            for bit in 0..8 {
                let x = (self.registers[x] as usize + bit) % SCREEN_WIDTH;
                let color = (self.memory[self.index as usize + byte] >> (7 - bit)) & 1;
                self.registers[CARRY_REGISTER] |= color & self.video[y][x];
                self.video[y][x] ^= color;
            }
        }
    }

    // Skip next instruction if key with the value of Vx is pressed.
    fn op_ex9e(&mut self, x: usize) {
        let key = self.registers[x] as usize;

        if self.keypad[key] == true {
            self.pc = self.pc + 2;
        }
    }

    // Skip next instruction if key with the value of Vx is not pressed.
    fn op_exa1(&mut self, x: usize) {
        let key = self.registers[x] as usize;

        if self.keypad[key] == false {
            self.pc = self.pc + 2;
        }
    }

    // Set Vx = delay timer value.
    fn op_fx07(&mut self, x: usize) {
        self.registers[x] = self.delay_timer;
    }

    // Wait for a key press, store the value of the key in Vx.
    fn op_fx0a(&mut self, x: usize) {
        let mut found = false;

        for i in 0..16 as usize {
            if self.keypad[i] == true {
                self.registers[x] = i as u8;
                found = true;
                break;
            }
        }

        if !found {
            self.pc = self.pc - 2;
        }
    }

    // Set delay timer = Vx.
    fn op_fx15(&mut self, x: usize) {
        self.delay_timer = self.registers[x];
    }

    // Set sound timer = Vx.
    fn op_fx18(&mut self, x: usize) {
        self.sound_timer = self.registers[x];
    }

    // Set I = I + Vx.
    fn op_fx1e(&mut self, x: usize) {
        self.index = self.index + self.registers[x] as u16;
    }

    // Set I = location of sprite for digit Vx.
    fn op_fx29(&mut self, x: usize) {
        let digit = self.registers[x];

        self.index = (FONTSET_START_ADDRESS + FONT_CHARACTER_BYTES * digit) as u16;
    }

    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
    // The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at
    // location in I, the tens digit at location I+1, and the ones digit at location I+2.
    fn op_fx33(&mut self, x: usize) {
        let vx = self.registers[x];
        self.memory[self.index as usize] = (vx / 100) as u8;
        self.memory[self.index as usize + 1] = ((vx % 100) / 10) as u8;
        self.memory[self.index as usize + 2] = (vx % 10) as u8;
    }

    // Store registers V0 through Vx in memory starting at location I.
    fn op_fx55(&mut self, x: usize) {
        for i in 0..x + 1 {
            self.memory[self.index as usize + i] = self.registers[i];
        }
    }

    // Read registers V0 through Vx from memory starting at location I.
    fn op_fx65(&mut self, x: usize) {
        for i in 0..x + 1 {
            self.registers[i] = self.memory[self.index as usize + i];
        }
    }

}