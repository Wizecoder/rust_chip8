use super::fonts::get_fonts;
use byteorder::{BigEndian, ByteOrder};
use rand;
use rand::Rng;

use sdl2;
use sdl2::pixels::Color;
use sdl2::audio::AudioCallback;
use sdl2::audio::AudioSpecDesired;

const RAM_SIZE: usize = 4096;

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        //Generate a square wave
        for x in out.iter_mut() {
            *x = match self.phase {
                0.0...0.5 => self.volume,
                _ => -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}


pub struct Interconnect {
    // renderer
    renderer: sdl2::render::Renderer<'static>,

    // audio
    audio_device: sdl2::audio::AudioDevice<SquareWave>,

    // events
    event_pump: sdl2::EventPump,
    
    ram: [u8; RAM_SIZE],

    pub halt: bool,

    display_state: [[bool; 32]; 64],

    key_state: [bool; 16],
}

impl Interconnect {
    pub fn new(program: Vec<u8>) -> Interconnect {
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

        let sdl_context = sdl2::init().unwrap();
        let video = sdl_context.video().unwrap();

        let window = video.window("Chip 8", 640, 320)
            .position_centered().opengl()
            .build().unwrap();

        let mut renderer = window.renderer()
            .accelerated()
            .build().unwrap();

        renderer.set_draw_color(Color::RGB(0, 0, 0));
        renderer.clear();
        renderer.present();

        // TODO: Setup audio device
        let audio_system = sdl_context.audio().unwrap();
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };

        let device = audio_system.open_playback(None, &desired_spec, |spec| {
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        }).unwrap();

        let event_pump = sdl_context.event_pump().unwrap();
       
        Interconnect {
            ram: ram,
            renderer: renderer,
            event_pump: event_pump,
            halt: false,
            display_state: [[false; 32]; 64],
            key_state: [false; 16],
            audio_device: device,
        }
    }

    pub fn handle_events(&mut self) {
        for event in self.event_pump.poll_iter() {
            use sdl2::event::Event::*;
            use sdl2::keyboard::Keycode::*;

            match event {
                Quit { .. } => self.halt = true,

                KeyDown { keycode, .. } => match keycode {
                    Some(Escape) => self.halt = true,
                    Some(Num0) => self.key_state[0x0] = true,
                    Some(Num1) => self.key_state[0x1] = true,
                    Some(Num2) => self.key_state[0x2] = true,
                    Some(Num3) => self.key_state[0x3] = true,
                    Some(Num4) => self.key_state[0x4] = true,
                    Some(Num5) => self.key_state[0x5] = true,
                    Some(Num6) => self.key_state[0x6] = true,
                    Some(Num7) => self.key_state[0x7] = true,
                    Some(Num8) => self.key_state[0x8] = true,
                    Some(Num9) => self.key_state[0x9] = true,
                    Some(A) => self.key_state[0xA] = true,
                    Some(B) => self.key_state[0xB] = true,
                    Some(C) => self.key_state[0xC] = true,
                    Some(D) => self.key_state[0xD] = true,
                    Some(E) => self.key_state[0xE] = true,
                    Some(F) => self.key_state[0xF] = true,
                    _ => {}
                },

                KeyUp { keycode, .. } => match keycode {
                    Some(Num0) => self.key_state[0x0] = false,
                    Some(Num1) => self.key_state[0x1] = false,
                    Some(Num2) => self.key_state[0x2] = false,
                    Some(Num3) => self.key_state[0x3] = false,
                    Some(Num4) => self.key_state[0x4] = false,
                    Some(Num5) => self.key_state[0x5] = false,
                    Some(Num6) => self.key_state[0x6] = false,
                    Some(Num7) => self.key_state[0x7] = false,
                    Some(Num8) => self.key_state[0x8] = false,
                    Some(Num9) => self.key_state[0x9] = false,
                    Some(A) => self.key_state[0xA] = false,
                    Some(B) => self.key_state[0xB] = false,
                    Some(C) => self.key_state[0xC] = false,
                    Some(D) => self.key_state[0xD] = false,
                    Some(E) => self.key_state[0xE] = false,
                    Some(F) => self.key_state[0xF] = false,
                    _ => {}
                },

                _ => {}
            }
        }
    }

    pub fn clear_display(&mut self) {
        self.renderer.set_draw_color(Color::RGB(0, 0, 0));
        self.renderer.clear();
        self.renderer.present();
    }

    fn keycode_from_key(&self, key: u8) -> sdl2::keyboard::Keycode {
        use sdl2::keyboard::Keycode::*;
        return match key {
            0x0 => Num0,
            0x1 => Num1,
            0x2 => Num2,
            0x3 => Num3,
            0x4 => Num4,
            0x5 => Num5,
            0x6 => Num6,
            0x7 => Num7,
            0x8 => Num8,
            0x9 => Num9,
            0xA => A,
            0xB => B,
            0xC => C,
            0xD => D,
            0xE => E,
            0xF => F,
            _ => Escape,
        }
    }

    pub fn wait_for_keypress(&mut self, key: u8) {
        let keycode_to_match = self.keycode_from_key(key);
        loop {
            use sdl2::event::Event::*;
            use sdl2::keyboard::Keycode::*;
            let event = self.event_pump.wait_event();
            match event {
                Quit { .. } => {
                    self.halt = true;
                    break;
                },

                KeyDown { keycode, .. } => {
                    match keycode {
                        Some(Escape) => {
                            self.halt = true;
                            break;
                        },
                        _ => {
                            if keycode == Some(keycode_to_match) {
                                break;
                            }
                        }
                    }
                },

                KeyUp { keycode, .. } => match keycode {
                    _ => {}
                },

                _ => {}
            }
        }
    }

    pub fn start_beep(&mut self) {
        self.audio_device.resume();
    }

    pub fn stop_beep(&mut self) {
        self.audio_device.pause();
    }

    pub fn wait_for_step(&mut self) -> bool {
        loop {
            use sdl2::event::Event::*;
            use sdl2::keyboard::Keycode::*;
            let event = self.event_pump.wait_event();
            match event {
                KeyDown { keycode, .. } => {
                    match keycode {
                        Some(S) => {
                            return false;
                        },
                        Some(P) => {
                            return true;
                        },
                        _ => { }
                    }
                },

                _ => {}
            }
        }
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

    fn write_byte_to_display(&mut self, addr: usize, x_loc: usize, y_loc: usize) -> bool {
        let byte = self.ram[addr] as u8;
        println!("Writing byte {0:b} at location {1} {2}", byte, x_loc, y_loc);
        let mut overrode = false;
        for i in 0..8 {
            let bit = (byte << i) >> 7;
            let x_pos = (x_loc + i) % 64;
            let cur_val = self.display_state[x_pos][y_loc];
            if bit == 1 && cur_val == true {
                self.display_state[x_pos][y_loc] = false;
                overrode = true;
            } else {
                let val = bit == 1 || cur_val;
                self.display_state[x_pos][y_loc] = val;
            }
        }

        return overrode;
    }

    pub fn display_bytes(&mut self, num_bytes: u8, i_addr: usize, x_loc: usize, y_loc: usize) -> bool {
        let mut overrode = false;
        for i in 0..num_bytes as usize {
            overrode = self.write_byte_to_display(i_addr + i, x_loc, (y_loc + i) % 32) || overrode;
        }
        self.render_display_state();
        return overrode;
    }

    fn render_display_state(&mut self) {
        self.renderer.set_draw_color(Color::RGB(0, 0, 0));
        self.renderer.clear();
        self.renderer.set_draw_color(Color::RGB(255, 255, 255));
        let mut rects: Vec<sdl2::rect::Rect> = Vec::new();
        for x in 0..64 {
            for y in 0..32 {
                if self.display_state[x][y] {
                    rects.push(sdl2::rect::Rect::new((x as i32) * 10, (y as i32) * 10, 10, 10));
                }
            }
        }
        self.renderer.fill_rects(&rects[..]);
        self.renderer.present();
    }

    pub fn is_key_pressed(&self, key: u8) -> bool{
        return self.key_state[key as usize];
    }
}
