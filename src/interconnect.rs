use super::fonts::get_fonts;
use byteorder::{BigEndian, ByteOrder};
use rand;
use rand::Rng;

const RAM_SIZE: usize = 4096;

pub struct Interconnect {
    // renderer
    // audio
    // events
    
    ram: [u8; RAM_SIZE],
}

impl Interconnect {
    pub fn new(program: Vec<u8>) -> Interconnect {
        // TODO (DONE): Load rom and fonts into memory
        let mut ram: [u8; RAM_SIZE] = [0; RAM_SIZE];

        let mut ram_index = 0;

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

        // TODO: Setup renderer
        //
        // TODO: Create window
        //
        // TODO: Setup audio device
        //
        // TODO: Setup event management
        Interconnect {
            ram: ram,
        }
    }

    pub fn clear_display(&self) {
        // TODO implement this
    }

    pub fn wait_for_keypress(&self, key: u8) {
        // TODO implement this
        loop {}
    }

    pub fn get_random_value(&self) -> u8 {
        let mut rng = rand::thread_rng();
        return rng.gen::<u8>();
    }

    #[inline(always)]
    pub fn read_word(&self, addr: u16) -> u16 {
        BigEndian::read_u16(&self.ram[addr as usize..])
    }

    pub fn write_to_addr(&mut self, addr: usize, val: u8) {
        self.ram[addr] = val;
    }

    pub fn get_from_addr(&self, addr: usize) -> u8 {
        return self.ram[addr];
    }

    pub fn display_bytes(&mut self, num_bytes: u8, i_addr: u16, x_loc: u8, y_loc: u8) {
        // TODO implement this
    }

    pub fn is_key_pressed(&self, key: u8) -> bool{
        // TODO implement this
        return false;
    }
}
