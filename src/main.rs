extern crate rand;
extern crate byteorder;
extern crate time;
extern crate sdl2;

use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;

use cpu::CPU;
use interconnect::Interconnect;

mod fonts;
mod cpu;
mod interconnect;

fn main() {
    let program_file_name = env::args().nth(1).unwrap();
    let program = read_bin(program_file_name);

    let interconnect = Interconnect::new(program);
    let mut cpu: CPU = CPU::new(interconnect);
    cpu.start();
}

fn read_bin<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut file = fs::File::open(path.as_ref()).unwrap();
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf).unwrap();
    file_buf
}

