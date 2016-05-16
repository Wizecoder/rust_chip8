use super::fonts::get_fonts;
use byteorder::{BigEndian, ByteOrder};

const RAM_SIZE: usize = 4096;

pub struct Interconnect {
    ram: [u8; RAM_SIZE],

    // renderer
    //
    // sound util
    //
    // timer
}

impl Interconnect {
    pub fn new() -> Interconnect {
        Interconnect {
            ram: [0; RAM_SIZE],
        }
    }

    pub fn setup(&mut self) {
        self.load_fonts_into_ram();
    }

    pub fn clear_display(&self) {
    }

    pub fn wait_for_keypress(&self, key: u8) {
        loop {}
    }

    #[inline(always)]
    pub fn read_word_from_ram(&self, addr: u16) -> u16 {
        BigEndian::read_u16(&self.ram[addr as usize..])
    }

    pub fn write_to_addr(&mut self, addr: usize, val: u8) {
        self.ram[addr] = val;
    }

    pub fn get_from_addr(&self, addr: usize) -> u8 {
        return self.ram[addr];
    }

    fn load_fonts_into_ram(&mut self) {
        let fonts = get_fonts();
        let mut ram_index = 0;
        for val in fonts {
            self.ram[ram_index] = val;
            ram_index = ram_index + 1;
        }
    }

    pub fn load_program_into_ram(&mut self, program: Vec<u8>) {
        let mut ram_index = 512;
        for val in program {
            self.ram[ram_index] = val;
            ram_index = ram_index + 1;
        }
    }
}
