extern crate byteorder;

use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;
use byteorder::{BigEndian, ByteOrder};

const MEM_SIZE: usize = 4096;
const NUM_GPR: usize = 16;
const STACK_SIZE: usize = 16;

const PROGRAM_START: usize = 512;

#[derive(Default)]
struct Interpreter {
    ram: Memory,

    reg_gpr: [u8; NUM_GPR],
    reg_i: u16,

    reg_dt: u8,
    reg_st: u8,

    reg_pc: u16,
    reg_sp: u8,

    stack: [u16; STACK_SIZE],
}

impl Interpreter {
    fn new() -> Interpreter {
        Interpreter::default()
    }

    fn start(&mut self) {
        self.reg_pc = 512;
        loop {
            let instr = self.read_word(self.reg_pc);
            self.parse_instruction(instr);
            break;
        }
    }

    #[inline(always)]
    fn read_word(&self, addr: u16) -> u16 {
        BigEndian::read_u16(&self.ram.storage[addr as usize..])
    }

    fn parse_instruction(&mut self, instr: u16) {
        let opcode = (instr >> 12) as u8;

        match opcode {
            0x0 => {
                // 0nnn - SYS addr
                // Jump to a machine code routine at nnn

                // 00E0 - CLS
                // Clear the display

                // 00EE - RET
                // Return from a subroutine
                println!("Opcode 0x0");
            },
            0x1 => {
                // 1nnn - JP addr
                // Jump to location nnn
                let addr = ((instr << 4) >> 4) as u16;
                self.reg_pc = addr;
                println!("Opcode 0x1");
            },
            0x2 => {
                // 2nnn - CALL addr
                // Call subroutine at nnn

                // NNN
                let addr = ((instr << 4) >> 4) as u16;
                self.stack[self.reg_sp as usize] = self.reg_pc;
                self.reg_pc = addr;
                println!("Opcode 0x2");
            },
            0x3 => {
                // 3xkk - SE Vx, byte
                // Skip next instruction if Vx = kk
                println!("Opcode 0x3");
            },
            0x4 => {
                // 4xkk - SNE Vx, byte
                // Skip next instruction if Vx != kk
                println!("Opcode 0x4");
            },
            0x5 => {
                // 5xy0 - SE Vx, Vy
                // Skip next instrution if Vx = Vy
                println!("Opcode 0x5");
            },
            0x6 => {
                // 6xkk - LD Vx, byte
                // Set Vx = kk
                println!("Opcode 0x6");
            },
            0x7 => {
                // 7xkk - ADD Vx, byte
                // Set Vx = Vx + kk
                println!("Opcode 0x7");
            },
            0x8 => {
                // 8xy0 - LD Vx, Vy
                // Set Vx = Vy

                // 8xy1 - OR Vx, Vy
                // Set Vx = Vx OR Vy

                // 8xy2 - AND Vx, Vy
                // Set Vx = Vx AND Vy

                // 8xy3 - XOR Vx, Vy
                // Set Vx = Vx XOR Vy
                //
                // 8xy4 - ADD Vx, Vy
                // Set Vx = Vx + Vy, set VF = carry
                //
                // 8xy5 - SUB Vx, Vy
                // Set Vx = Vx - Vy, set VF = NOT borrow
                //
                // 8xy6 - SHR Vx, Vy
                // Set Vx = Vy SHIFT_RIGHT 1, set VF to least sig bit
                //
                // 8xy7 - SUBN Vx, Vy
                // Set Vx = Vy - Vx, set VF = NOT borrow
                //
                // 8xyE - SHL Vx, Vy
                // Set Vx = Vy SIFT_LEFT 1, set VF to most sig bit
                println!("Opcode 0x8");
            },
            0x9 => {
                // 9xy0 - SNE Vx, Vy
                // Skip next instruction if Vx != Vy
                println!("Opcode 0x9");
            },
            0xA => {
                // Annn - LD 1, addr
                // Set I = nnn
                println!("Opcode 0xA");
            },
            0xB => {
                // Bnnn - JP V0, addr
                // Jump to location nnn + V0
                println!("Opcode 0xB");
            },
            0xC => {
                // Cxkk - RND Vx, byte
                // Set Vx = random byte AND kk
                println!("Opcode 0xC");
            },
            0xD => {
                // Dxyn - DRW Vx, Vy, nibble
                // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
                println!("Opcode 0xD");
            },
            0xE => {
                // Ex9E - SKP Vx
                // Skip next instruction if key with the value of Vx is pressed
                //
                // ExA1 - SKNP Vx
                // Skip next instruction if key with the value of Vx is not pressed
                println!("Opcode 0xE");
            },
            0xF => {
                // Fx07 - LD Vx, DT
                // Set Vx = delay timer value
                //
                // Fx0A  - LD Vx, K
                // Wait for a key press, store the value of the key in Vx
                //
                // Fx15 - LD DT, Vx
                // Set delay timer = Vx
                //
                // Fx18 - LD ST, Vx
                // Set sound timer = Vx
                //
                // Fx1E - ADD I, Vx
                // Set I = I + Vx
                //
                // Fx29 - LD F, Vx
                // Set I = location of sprite for digit Vx
                //
                // Fx33 - LD B, Vx
                // Store BCD representation of Vx in Memory Locations I, I+1, and I+2
                //
                // Fx55 - LD [I], Vx
                // Store registers V0 through Vx in memory starting at location I
                //
                // Fx65 - LD Vx, [I]
                // Read registers V0 through Vx from memory starting at locaiton I
                println!("Opcode 0xF");
            },
            _ => {
                println!("Unknown Opcode");
            },
        }
    }
}

struct Memory {
    storage: [u8; MEM_SIZE],
}

impl Memory {
    fn new() -> Memory {
        Memory {
            storage: [0; MEM_SIZE],
        }
    }
}

impl Default for Memory {
    fn default() -> Memory { Memory::new() }
}


fn main() {
    let program_file_name = env::args().nth(1).unwrap();
    let program = read_bin(program_file_name);

    let mut interp: Interpreter = Interpreter::new();

    let mut ram: [u8; MEM_SIZE] = [0; MEM_SIZE];
    let mut ram_index = 0;

    // Save fonts
    let fonts = get_fonts();
    for val in fonts {
        ram[ram_index] = val;
        ram_index = ram_index + 1;
    }

    ram_index = 512;

    for val in program {
        ram[ram_index] = val;
        ram_index = ram_index + 1;
    }

    let mut new_ram: Memory = Default::default();
    new_ram.storage = ram;

    interp.ram = new_ram;

    interp.start();
}

fn read_bin<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut file = fs::File::open(path.as_ref()).unwrap();
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf).unwrap();
    file_buf
}


// Define the binary data for the fonts
fn get_fonts() -> Vec<u8> {
    return vec![
        // 0
        0xF0,
        0x90,
        0x90,
        0x90,
        0xF0,

        // 1
        0x20,
        0x60,
        0x20,
        0x20,
        0x70,

        // 2
        0xF0,
        0x10,
        0xF0,
        0x80,
        0xF0,

        // 3
        0xF0,
        0x10,
        0xF0,
        0x10,
        0xF0,

        // 4
        0x90,
        0x90,
        0xF0,
        0x10,
        0x10,

        // 5
        0xF0,
        0x80,
        0xF0,
        0x10,
        0xF0,

        // 6
        0xF0,
        0x80,
        0xF0,
        0x90,
        0xF0,
        
        // 7
        0xF0,
        0x10,
        0x20,
        0x40,
        0x40,
        
        // 8
        0xF0,
        0x90,
        0xF0,
        0x90,
        0xF0,
        
        // 9
        0xF0,
        0x90,
        0xF0,
        0x10,
        0xF0,

        // A
        0xF0,
        0x90,
        0xF0,
        0x90,
        0x90,
        
        // B
        0xE0,
        0x90,
        0xE0,
        0x90,
        0xE0,

        // C
        0xF0,
        0x80,
        0x80,
        0x80,
        0xF0,

        // D
        0xE0,
        0x90,
        0x90,
        0x90,
        0xE0,
        
        // E
        0xF0,
        0x80,
        0xF0,
        0x80,
        0xF0,
        
        // F
        0xF0,
        0x80,
        0xF0,
        0x80,
        0x80,
    ];
}
