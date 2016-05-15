extern crate byteorder;
extern crate time;
extern crate sdl2;

use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;
use byteorder::{BigEndian, ByteOrder};
use time::PreciseTime;

const MEM_SIZE: usize = 4096;
const NUM_GPR: usize = 16;
const STACK_SIZE: usize = 16;

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

        let ticks = 60;
        let ns = 1000000000 / ticks;

        let mut delta: i64 = 0;
        let mut last_time = PreciseTime::now();
        loop {
            let instr = self.read_word(self.reg_pc);
            self.reg_pc = self.reg_pc + 2;
            if self.parse_instruction(instr) {
                break;
            }
            let now = PreciseTime::now();
            delta += last_time.to(now).num_nanoseconds().unwrap();
            last_time = now;
            if delta > ns {
                let steps = (delta / ns) as u8;
                delta = delta % ns;
                if self.reg_dt < steps { self.reg_dt = 0 } else { self.reg_dt = self.reg_dt - steps }
                if self.reg_st < steps { self.reg_st = 0 } else { self.reg_st = self.reg_st - steps }
            }
        }
    }

    #[inline(always)]
    fn read_word(&self, addr: u16) -> u16 {
        BigEndian::read_u16(&self.ram.storage[addr as usize..])
    }

    fn parse_instruction(&mut self, instr: u16) -> bool {
        println!("{0:x}", instr);
        if instr == 0x0000 {
            return true;
        }
        let opcode = (instr >> 12) as u8;

        match opcode {
            0x0 => {
                let filter = ((instr << 4) >> 4) as u16;

                // 0nnn - SYS addr
                // Jump to a machine code routine at nnn
                if filter != 0x0EE && filter != 0x0E0 {
                    println!("Jumping to machine code {}", filter);
                }

                // 00E0 - CLS
                // Clear the display
                if filter == 0x0E0 {
                    println!("Clearing the Display");
                }

                // 00EE - RET
                // Return from a subroutine
                if filter == 0x0EE {
                    self.reg_pc = self.stack[self.reg_sp as usize];
                    self.reg_sp = self.reg_sp - 1;
                    println!("Returning to {0:x}", self.reg_pc);
                }
            },
            0x1 => {
                // 1nnn - JP addr
                // Jump to location nnn
                let addr = ((instr << 4) >> 4) as u16;
                self.reg_pc = addr;
                println!("Jumping to location {0:x}", self.reg_pc);
            },
            0x2 => {
                // 2nnn - CALL addr
                // Call subroutine at nnn
                let addr = ((instr << 4) >> 4) as u16;
                self.reg_sp = self.reg_sp + 1;
                self.stack[self.reg_sp as usize] = self.reg_pc;
                self.reg_pc = addr;
                println!("Calling Address {0:x}", addr);
            },
            0x3 => {
                // 3xkk - SE Vx, byte
                // Skip next instruction if Vx = kk
                let reg = ((instr << 4) >> 12) as usize;
                let val = ((instr << 8) >> 8) as u8;
                if self.reg_gpr[reg] == val {
                    self.reg_pc = self.reg_pc + 2;
                    println!("Skipping next instruction because reg {} == {}", reg, val);
                }
            },
            0x4 => {
                // 4xkk - SNE Vx, byte
                // Skip next instruction if Vx != kk
                let reg = ((instr << 4) >> 12) as usize;
                let val = ((instr << 8) >> 8) as u8;
                if self.reg_gpr[reg] != val {
                    self.reg_pc = self.reg_pc + 2;
                    println!("Skipping next instruction because reg {} != {}", reg, val);
                }
            },
            0x5 => {
                // 5xy0 - SE Vx, Vy
                // Skip next instrution if Vx = Vy
                let reg_x = ((instr << 4) >> 12) as usize;
                let reg_y = ((instr << 8) >> 12) as usize;
                if self.reg_gpr[reg_x] != self.reg_gpr[reg_y] {
                    self.reg_pc = self.reg_pc + 2;
                    println!("Skipping next instruction because reg {} == reg {}", reg_x, reg_y);
                }
            },
            0x6 => {
                // 6xkk - LD Vx, byte
                // Set Vx = kk
                let reg = ((instr << 4) >> 12) as usize;
                let val = ((instr << 8) >> 8) as u8;
                self.reg_gpr[reg] = val;
                println!("Setting register {} = {}", reg, val);
            },
            0x7 => {
                // 7xkk - ADD Vx, byte
                // Set Vx = Vx + kk
                let reg = ((instr << 4) >> 12) as usize;
                let val = ((instr << 8) >> 8) as u8;
                self.reg_gpr[reg] = self.reg_gpr[reg].wrapping_add(val);
                println!("Adding {} to register {} = {}", val, reg, self.reg_gpr[reg]);
            },
            0x8 => {
                let reg_x = ((instr << 4) >> 12) as usize;
                let reg_y = ((instr << 8) >> 12) as usize;
                let last_val = ((instr << 12) >> 12) as u8;

                // 8xy0 - LD Vx, Vy
                // Set Vx = Vy
                if last_val == 0x0 {
                    self.reg_gpr[reg_x] = self.reg_gpr[reg_y];
                    println!("Setting register {} = register {} = {}", reg_x, reg_y, self.reg_gpr[reg_x]);
                }

                // 8xy1 - OR Vx, Vy
                // Set Vx = Vx OR Vy
                if last_val == 0x1 {
                    let new_val = self.reg_gpr[reg_x] | self.reg_gpr[reg_y];
                    self.reg_gpr[reg_x] = new_val;
                    println!("ORing register {} with register {} = {}", reg_x, reg_y, new_val);
                }

                // 8xy2 - AND Vx, Vy
                // Set Vx = Vx AND Vy
                if last_val == 0x2 {
                    let new_val = self.reg_gpr[reg_x] & self.reg_gpr[reg_y];
                    self.reg_gpr[reg_x] = new_val;
                    println!("ANDing register {} with register {} = {}", reg_x, reg_y, new_val);
                }

                // 8xy3 - XOR Vx, Vy
                // Set Vx = Vx XOR Vy
                if last_val == 0x3 {
                    let new_val = self.reg_gpr[reg_x] ^ self.reg_gpr[reg_y];
                    self.reg_gpr[reg_x] = new_val;
                    println!("XORing register {} with register {} = {}", reg_x, reg_y, new_val);
                }

                // 8xy4 - ADD Vx, Vy
                // Set Vx = Vx + Vy, set VF = carry
                if last_val == 0x4 {
                    let original_val = self.reg_gpr[reg_x];
                    let new_val = original_val.wrapping_add(self.reg_gpr[reg_y]);
                    let overflowed = new_val < original_val;
                    self.reg_gpr[reg_x] = new_val;
                    self.reg_gpr[0xF] = if overflowed { 0x1 } else { 0x0 };
                    println!("Adding register {} to register {} = {}", reg_y, reg_x, new_val);
                }

                // 8xy5 - SUB Vx, Vy
                // Set Vx = Vx - Vy, set VF = NOT borrow
                if last_val == 0x5 {
                    let x_val = self.reg_gpr[reg_x];
                    let y_val = self.reg_gpr[reg_y];
                    let not_borrowed = x_val > y_val;
                    let new_val = x_val.wrapping_sub(y_val);
                    self.reg_gpr[reg_x] = new_val;
                    self.reg_gpr[0xF] = if not_borrowed { 0x1 } else { 0x0 };
                    println!("Subtracting register {} from register {} = {}", reg_y, reg_y, new_val);
                }

                // 8xy6 - SHR Vx, Vy
                // Set Vx = Vy SHIFT_RIGHT 1, set VF to least sig bit
                if last_val == 0x6 {
                    println!("Shift right unsupported");
                }

                // 8xy7 - SUBN Vx, Vy
                // Set Vx = Vy - Vx, set VF = NOT borrow
                if last_val == 0x7 {
                    let x_val = self.reg_gpr[reg_x];
                    let y_val = self.reg_gpr[reg_y];
                    let not_borrowed = y_val > x_val;
                    let new_val = y_val.wrapping_sub(x_val);
                    self.reg_gpr[reg_x] = new_val;
                    self.reg_gpr[0xF] = if not_borrowed { 0x1 } else { 0x0 };
                    println!("Setting reg {} = reg {} - reg {} = {}", reg_x, reg_y, reg_x, new_val);
                }

                // 8xyE - SHL Vx, Vy
                // Set Vx = Vy SIFT_LEFT 1, set VF to most sig bit
                if last_val == 0xE {
                    println!("Shift left unsupported");
                }
            },
            0x9 => {
                // 9xy0 - SNE Vx, Vy
                // Skip next instruction if Vx != Vy
                let reg_x = ((instr << 4) >> 12) as usize;
                let reg_y = ((instr << 8) >> 12) as usize;
                if self.reg_gpr[reg_x] != self.reg_gpr[reg_y] {
                    self.reg_pc = self.reg_pc + 2;
                    println!("Skipping next instruction because reg {} == reg {}", reg_x, reg_y);
                }
            },
            0xA => {
                // Annn - LD 1, addr
                // Set I = nnn
                let addr = ((instr << 4) >> 4) as u16;
                self.reg_i = addr;
                println!("Setting register I to {0:x}", addr);
            },
            0xB => {
                // Bnnn - JP V0, addr
                // Jump to location nnn + V0
                let addr = ((instr << 4) >> 4) as u16;
                let reg_val = self.reg_gpr[0x0] as u16;
                let jmp_addr = reg_val + addr;
                self.reg_pc = jmp_addr;
                println!("Jumping to address {0:x}", jmp_addr);
            },
            0xC => {
                // Cxkk - RND Vx, byte
                // Set Vx = random byte AND kk
                println!("Randomness not yet implemented");
            },
            0xD => {
                // Dxyn - DRW Vx, Vy, nibble
                // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
                let reg_x = ((instr << 4) >> 12) as usize;
                let reg_y = ((instr << 8) >> 12) as usize;
                let x_val = self.reg_gpr[reg_x];
                let y_val = self.reg_gpr[reg_y];
                let n = ((instr << 12) >> 12) as u8;
                println!("[not yet implemented] Displaying {} bytes at {}, {}", n, x_val, y_val);
            },
            0xE => {
                let filter = ((instr << 8) >> 8) as u16;
                let reg = ((instr << 4) >> 12) as usize;
                let reg_val = self.reg_gpr[reg];
                // Ex9E - SKP Vx
                // Skip next instruction if key with the value of Vx is pressed
                if filter == 0x9E {
                    println!("[not yet implemented] Skipping if key {} is pressed", reg_val);
                }

                // ExA1 - SKNP Vx
                // Skip next instruction if key with the value of Vx is not pressed
                if filter == 0xA1 {
                    println!("[not yet implemented] Skipping if key {} is not pressed", reg_val);
                }
            },
            0xF => {
                let filter = ((instr << 8) >> 8) as u16;
                let reg = ((instr << 4) >> 12) as usize;
                // Fx07 - LD Vx, DT
                // Set Vx = delay timer value
                if filter == 0x07 {
                    self.reg_gpr[reg] = self.reg_dt;
                    println!("Setting reg {} to delay timer val = {}", reg, self.reg_dt);
                }
                
                // Fx0A  - LD Vx, K
                // Wait for a key press, store the value of the key in Vx
                if filter == 0x0A {
                    loop {}
                    println!("[not yet implemented] waiting for key press");
                }

                // Fx15 - LD DT, Vx
                // Set delay timer = Vx
                if filter == 0x15 {
                    self.reg_dt = self.reg_gpr[reg];
                    println!("Setting delay timer to {}", self.reg_dt);
                }

                // Fx18 - LD ST, Vx
                // Set sound timer = Vx
                if filter == 0x18 {
                    self.reg_st = self.reg_gpr[reg];
                    println!("Setting sound timer to {}", self.reg_st);
                }

                // Fx1E - ADD I, Vx
                // Set I = I + Vx
                if filter == 0x1E {
                    self.reg_i = self.reg_i + (self.reg_gpr[reg] as u16);
                    println!("Setting reg I to {}", self.reg_i);
                }
                
                // Fx29 - LD F, Vx
                // Set I = location of sprite for digit Vx
                if filter == 0x29 {
                    let digit = self.reg_gpr[reg];
                    self.reg_i = 0x5 * digit as u16;
                    println!("Setting I to location of digit {} = {}", digit, self.reg_i);
                }
                
                // Fx33 - LD B, Vx
                // Store BCD representation of Vx in Memory Locations I, I+1, and I+2
                if filter == 0x33 {
                    println!("[not yet implemented] Storing BCD rep");
                }
                
                // Fx55 - LD [I], Vx
                // Store registers V0 through Vx in memory starting at location I
                if filter == 0x55 {
                    let mem_index = self.reg_i as usize;
                    for n in 0..reg {
                        self.ram.storage[mem_index + n] = self.reg_gpr[n];
                    }
                    println!("Storing registers 0 to {} starting at I", reg);
                }
                
                // Fx65 - LD Vx, [I]
                // Read registers V0 through Vx from memory starting at locaiton I
                if filter == 0x65 {
                    let mem_index = self.reg_i as usize;
                    for n in 0..reg {
                        self.reg_gpr[n] = self.ram.storage[mem_index + n];
                    }
                    println!("Loading registers 0 to {} from I", reg);
                }
            },
            _ => {
                println!("Unknown Opcode");
            },
        }
        return false;
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
    let ctx = sdl2::init().unwrap();
    let video_ctx = ctx.video().unwrap();
    let mut timer = ctx.timer().unwrap();

    let mut window = match video_ctx.window("chip8", 640, 480).position_centered().opengl().build() {
        Ok(window) => window,
        Err(err)   => panic!("failed to create window: {}", err)
    };

    window.show();
    timer.delay(3000);

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
