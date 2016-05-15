use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;

const MEM_SIZE: usize = 4096;
const NUM_GPR: usize = 16;
const STACK_SIZE: usize = 16;

struct Interpreter {
    ram: [u32; MEM_SIZE],

    reg_gpr: [u8; NUM_GPR],
    reg_i: u16,

    reg_dt: u8,
    reg_st: u8,

    reg_pc: u16,
    reg_sp: u8,

    stack: [u16; STACK_SIZE],
}

impl Interpreter {
/*    fn new() -> Interpreter {
        Interpreter {
            // TODO
        }
    }*/
}

fn main() {
    let program_file_name = env::args().nth(1).unwrap();
    let program = read_bin(program_file_name);
    
    // store fonts in ram
    // load program into the ram
    // start interpreter
}

fn read_bin<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut file = fs::File::open(path.as_ref()).unwrap();
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf).unwrap();
    file_buf
}

fn parse_instruction(instr: u32) {
    let opcode = (instr >> 12) as u8;

    match opcode {
        0x0 => {},
        0x1 => {println!("Opcode 0x1");},
        0x2 => {},
        0x3 => {},
        0x4 => {},
        0x5 => {},
        0x6 => {},
        0x7 => {},
        0x8 => {},
        0x9 => {},
        0xA => {},
        0xB => {},
        0xC => {},
        0xD => {},
        0xE => {},
        0xF => {},
        _ => {println!("Unknown Opcode");},
    }
}


