use super::interconnect;

use time::PreciseTime;
use std::time::Duration;
use std::thread;

const NUM_GPR: usize = 16;
const STACK_SIZE: usize = 16;

pub struct CPU {
    interconnect: interconnect::Interconnect,

    reg_gpr: [u8; NUM_GPR],
    reg_i: u16,

    reg_dt: u8,
    reg_st: u8,

    // Program Counter
    reg_pc: u16,

    // Stack Pointer
    reg_sp: u8,

    stack: [u16; STACK_SIZE],
}

impl CPU {
    pub fn new(interconnect: interconnect::Interconnect) -> CPU {
        CPU {
            interconnect: interconnect,

            reg_gpr: [0; NUM_GPR],
            reg_i: 0,

            reg_dt: 0,
            reg_st: 0,

            reg_pc: 0,

            reg_sp: 0,

            stack: [0; STACK_SIZE],
        }
    }

    pub fn start(&mut self) {
        self.reg_pc = 512;

        loop {
            if self.interconnect.halt {
                break;
            }

            let instr = self.interconnect.read_word(self.reg_pc);
            let debug = false;
            if (debug) {
                println!("Instr: {0:x}", instr);
                if (self.interconnect.wait_for_step()) {
                    println!("Regs: {:?}", self.reg_gpr);
                    println!("PC: {0:x}", self.reg_pc);
                    println!("SP: {}", self.reg_sp);
                    println!("PC: {0:x}", self.reg_i);
                    
                    println!("DT: {}", self.reg_dt);
                    println!("ST: {}", self.reg_st);
                }
            }
            self.reg_pc = self.reg_pc + 2;
            if self.parse_instruction(instr) {
                break;
            }

            self.interconnect.handle_events();

            let dt_enabled = self.reg_dt > 0;
            let st_enabled = self.reg_st > 0;

            if (!st_enabled) {
                self.interconnect.stop_beep();
            }
            if dt_enabled || st_enabled {
                thread::sleep(Duration::from_millis(16));
                if dt_enabled {
                    self.reg_dt -= 1;
                }
                if st_enabled {
                    self.reg_st -= 1;
                    self.interconnect.start_beep();
                }
            } else {
                thread::sleep(Duration::from_millis(2));
            }

        }
    }

    fn parse_instruction(&mut self, instr: u16) -> bool {
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
                    self.reg_pc = filter;
                }

                // 00E0 - CLS
                // Clear the display
                if filter == 0x0E0 {
                    self.interconnect.clear_display();
                }

                // 00EE - RET
                // Return from a subroutine
                if filter == 0x0EE {
                    self.reg_pc = self.stack[self.reg_sp as usize];
                    self.reg_sp = self.reg_sp - 1;
                }
            },
            0x1 => {
                // 1nnn - JP addr
                // Jump to location nnn
                let addr = ((instr << 4) >> 4) as u16;
                self.reg_pc = addr;
            },
            0x2 => {
                // 2nnn - CALL addr
                // Call subroutine at nnn
                let addr = ((instr << 4) >> 4) as u16;
                self.reg_sp = self.reg_sp + 1;
                self.stack[self.reg_sp as usize] = self.reg_pc;
                self.reg_pc = addr;
            },
            0x3 => {
                // 3xkk - SE Vx, byte
                // Skip next instruction if Vx = kk
                let reg = ((instr << 4) >> 12) as usize;
                let val = ((instr << 8) >> 8) as u8;
                if self.reg_gpr[reg] == val {
                    self.reg_pc = self.reg_pc + 2;
                }
            },
            0x4 => {
                // 4xkk - SNE Vx, byte
                // Skip next instruction if Vx != kk
                let reg = ((instr << 4) >> 12) as usize;
                let val = ((instr << 8) >> 8) as u8;
                if self.reg_gpr[reg] != val {
                    self.reg_pc = self.reg_pc + 2;
                }
            },
            0x5 => {
                // 5xy0 - SE Vx, Vy
                // Skip next instrution if Vx = Vy
                let reg_x = ((instr << 4) >> 12) as usize;
                let reg_y = ((instr << 8) >> 12) as usize;
                if self.reg_gpr[reg_x] != self.reg_gpr[reg_y] {
                    self.reg_pc = self.reg_pc + 2;
                }
            },
            0x6 => {
                // 6xkk - LD Vx, byte
                // Set Vx = kk
                let reg = ((instr << 4) >> 12) as usize;
                let val = ((instr << 8) >> 8) as u8;
                self.reg_gpr[reg] = val;
            },
            0x7 => {
                // 7xkk - ADD Vx, byte
                // Set Vx = Vx + kk
                let reg = ((instr << 4) >> 12) as usize;
                let val = ((instr << 8) >> 8) as u8;
                self.reg_gpr[reg] = self.reg_gpr[reg].wrapping_add(val);
            },
            0x8 => {
                let reg_x = ((instr << 4) >> 12) as usize;
                let reg_y = ((instr << 8) >> 12) as usize;
                let last_val = ((instr << 12) >> 12) as u8;

                // 8xy0 - LD Vx, Vy
                // Set Vx = Vy
                if last_val == 0x0 {
                    self.reg_gpr[reg_x] = self.reg_gpr[reg_y];
                }

                // 8xy1 - OR Vx, Vy
                // Set Vx = Vx OR Vy
                if last_val == 0x1 {
                    let new_val = self.reg_gpr[reg_x] | self.reg_gpr[reg_y];
                    self.reg_gpr[reg_x] = new_val;
                }

                // 8xy2 - AND Vx, Vy
                // Set Vx = Vx AND Vy
                if last_val == 0x2 {
                    let new_val = self.reg_gpr[reg_x] & self.reg_gpr[reg_y];
                    self.reg_gpr[reg_x] = new_val;
                }

                // 8xy3 - XOR Vx, Vy
                // Set Vx = Vx XOR Vy
                if last_val == 0x3 {
                    let new_val = self.reg_gpr[reg_x] ^ self.reg_gpr[reg_y];
                    self.reg_gpr[reg_x] = new_val;
                }

                // 8xy4 - ADD Vx, Vy
                // Set Vx = Vx + Vy, set VF = carry
                if last_val == 0x4 {
                    let original_val = self.reg_gpr[reg_x];
                    let new_val = original_val.wrapping_add(self.reg_gpr[reg_y]);
                    let overflowed = new_val < original_val;
                    self.reg_gpr[reg_x] = new_val;
                    self.reg_gpr[0xF] = if overflowed { 0x1 } else { 0x0 };
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
                }

                // 8xy6 - SHR Vx, Vy
                // Set Vx = Vy SHIFT_RIGHT 1, set VF to least sig bit
                if last_val == 0x6 {
                    let y_val = self.reg_gpr[reg_y];
                    let least_sig_bit = (y_val << 7) >> 7;
                    let new_val = y_val >> 1;
                    self.reg_gpr[reg_x] = new_val;
                    self.reg_gpr[0xF] = least_sig_bit;
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
                }

                // 8xyE - SHL Vx, Vy
                // Set Vx = Vy SIFT_LEFT 1, set VF to most sig bit
                if last_val == 0xE {
                    let y_val = self.reg_gpr[reg_y];
                    let most_sig_bit = y_val >> 7;
                    let new_val = y_val << 1;
                    self.reg_gpr[reg_x] = new_val;
                    self.reg_gpr[0xF] = most_sig_bit;
                }
            },
            0x9 => {
                // 9xy0 - SNE Vx, Vy
                // Skip next instruction if Vx != Vy
                let reg_x = ((instr << 4) >> 12) as usize;
                let reg_y = ((instr << 8) >> 12) as usize;
                if self.reg_gpr[reg_x] != self.reg_gpr[reg_y] {
                    self.reg_pc = self.reg_pc + 2;
                }
            },
            0xA => {
                // Annn - LD 1, addr
                // Set I = nnn
                let addr = ((instr << 4) >> 4) as u16;
                self.reg_i = addr;
            },
            0xB => {
                // Bnnn - JP V0, addr
                // Jump to location nnn + V0
                let addr = ((instr << 4) >> 4) as u16;
                let reg_val = self.reg_gpr[0x0] as u16;
                let jmp_addr = reg_val + addr;
                self.reg_pc = jmp_addr;
            },
            0xC => {
                // Cxkk - RND Vx, byte
                // Set Vx = random byte AND kk
                let reg = ((instr << 4) >> 12) as usize;
                let val = ((instr << 8) >> 8) as u8;
                let rand_val = self.interconnect.get_random_value();
                let anded_val = rand_val & val;
                self.reg_gpr[reg] = anded_val;
            },
            0xD => {
                // Dxyn - DRW Vx, Vy, nibble
                // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
                let reg_x = ((instr << 4) >> 12) as usize;
                let reg_y = ((instr << 8) >> 12) as usize;
                let x_val = self.reg_gpr[reg_x];
                let y_val = self.reg_gpr[reg_y];
                let n = ((instr << 12) >> 12) as u8;
                let overrode = self.interconnect.display_bytes(n, self.reg_i as usize, x_val as usize, y_val as usize);
                self.reg_gpr[0xF] = if overrode { 1 } else { 0 };
            },
            0xE => {
                let filter = ((instr << 8) >> 8) as u16;
                let reg = ((instr << 4) >> 12) as usize;
                let reg_val = self.reg_gpr[reg];

                // Ex9E - SKP Vx
                // Skip next instruction if key with the value of Vx is pressed
                if filter == 0x9E {
                    if self.interconnect.is_key_pressed(reg_val) {
                        self.reg_pc = self.reg_pc + 2;
                    }
                }

                // ExA1 - SKNP Vx
                // Skip next instruction if key with the value of Vx is not pressed
                if filter == 0xA1 {
                    if !self.interconnect.is_key_pressed(reg_val) {
                        self.reg_pc = self.reg_pc + 2;
                    }
                }
            },
            0xF => {
                let filter = ((instr << 8) >> 8) as u16;
                let reg = ((instr << 4) >> 12) as usize;
                // Fx07 - LD Vx, DT
                // Set Vx = delay timer value
                if filter == 0x07 {
                    self.reg_gpr[reg] = self.reg_dt;
                }
                
                // Fx0A  - LD Vx, K
                // Wait for a key press, store the value of the key in Vx
                if filter == 0x0A {
                    let key = self.reg_gpr[reg];
                    self.interconnect.wait_for_keypress(key);
                }

                // Fx15 - LD DT, Vx
                // Set delay timer = Vx
                if filter == 0x15 {
                    self.reg_dt = self.reg_gpr[reg];
                }

                // Fx18 - LD ST, Vx
                // Set sound timer = Vx
                if filter == 0x18 {
                    self.reg_st = self.reg_gpr[reg];
                }

                // Fx1E - ADD I, Vx
                // Set I = I + Vx
                if filter == 0x1E {
                    self.reg_i = self.reg_i + (self.reg_gpr[reg] as u16);
                }
                
                // Fx29 - LD F, Vx
                // Set I = location of sprite for digit Vx
                if filter == 0x29 {
                    let digit = self.reg_gpr[reg];
                    self.reg_i = 0x5 * digit as u16;
                }
                
                // Fx33 - LD B, Vx
                // Store BCD representation of Vx in Memory Locations I, I+1, and I+2
                if filter == 0x33 {
                    let val = self.reg_gpr[reg];
                    let dig1 = val / 100;
                    let dig2 = (val % 100) / 10;
                    let dig3 = val % 10;
                    let i = self.reg_i as usize;
                    self.interconnect.write_to_addr(i, dig1);
                    self.interconnect.write_to_addr(i + 1, dig2);
                    self.interconnect.write_to_addr(i + 2, dig3);
                }
                
                // Fx55 - LD [I], Vx
                // Store registers V0 through Vx in memory starting at location I
                if filter == 0x55 {
                    let mem_index = self.reg_i as usize;
                    for n in 0..reg+1 {
                        self.interconnect.write_to_addr(mem_index + n, self.reg_gpr[n]);
                    }
                }
                
                // Fx65 - LD Vx, [I]
                // Read registers V0 through Vx from memory starting at locaiton I
                if filter == 0x65 {
                    let mem_index = self.reg_i as usize;
                    for n in 0..reg+1 {
                        self.reg_gpr[n] = self.interconnect.get_from_addr(mem_index + n);
                    }
                }
            },
            _ => {
                println!("Unknown Opcode");
            },
        }
        return false;
    }
}
