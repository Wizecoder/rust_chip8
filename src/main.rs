use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;

fn main() {
    let start_of_ram = 0;
    let end_of_ram = 4095;
    let start_of_program = 512;

    let regs: [u8; 16];

    let instr = 0x1010;

    let rom_file_name = env::args().nth(1).unwrap();

    let rom = read_bin(rom_file_name);
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


