// The MIT License (MIT)

// Copyright (c) 2021 AnonymousDapper

#![deny(rust_2018_idioms)]
#![feature(array_map)]

pub mod dis;
pub mod display;

use std::convert::TryInto;
use std::io::{stdin, stdout, Write};
use std::thread;
use std::time::{Duration, Instant};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

#[rustfmt::skip]
static FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80,
];

pub fn handle_ctrlc() {
    print!("\x1b[45;0H{}", termion::cursor::Show);
    std::process::exit(1);
}

fn translate_keypad(key: char) -> Option<u8> {
    match key {
        '1' => Some(1),
        '2' => Some(2),
        '3' => Some(3),
        '4' => Some(0xc),
        'q' => Some(4),
        'w' => Some(5),
        'e' => Some(6),
        'r' => Some(0xd),
        'a' => Some(7),
        's' => Some(8),
        'd' => Some(9),
        'f' => Some(0xe),
        'z' => Some(0xa),
        'x' => Some(0),
        'c' => Some(0xb),
        'v' => Some(0xf),
        _ => None,
    }
}

fn scan_key(code: u8) -> bool {
    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin = stdin();
    let mut rv: bool = false;

    for c in stdin.keys() {
        if let Key::Ctrl('c') = c.as_ref().unwrap() {
            handle_ctrlc();
        }

        if let Key::Char(key) = c.unwrap() {
            if let Some(key_code) = translate_keypad(key) {
                if key_code == code {
                    rv = true;
                }
            }
        }
        stdout.flush().unwrap();
    }

    std::mem::drop(stdout);
    rv
}

fn wait_key() -> u8 {
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut got_key = None;

    while got_key.is_none() {
        let stdin = stdin();
        for c in stdin.keys() {
            if let Key::Ctrl('c') = c.as_ref().unwrap() {
                handle_ctrlc();
            }

            if let Key::Char(key) = c.unwrap() {
                if let Some(key_code) = translate_keypad(key) {
                    got_key = Some(key_code);
                    break;
                }
            }
            stdout.flush().unwrap();
        }

        thread::sleep(Duration::from_millis(5));
    }

    std::mem::drop(stdout);

    got_key.unwrap()
}

#[derive(Debug)]
pub struct InstHelper {
    pub nnn: u16,
    pub nn: u8,
    pub x: usize,
    pub y: usize,
}

#[derive(Debug)]
pub struct Timer60Hz {
    val: u8,
    t: Option<Instant>,
    time_scale: u64,
}

impl Timer60Hz {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            val: 0,
            t: None,
            time_scale: 1,
        }
    }

    pub fn with_time_scale(scale: u64) -> Self {
        Self {
            val: 0,
            t: None,
            time_scale: scale,
        }
    }

    pub fn set_time_scale(&mut self, scale: u64) {
        self.time_scale = scale;
    }

    pub fn set(&mut self, v: u8) {
        self.t = Some(Instant::now());
        self.val = v;
    }

    pub fn get(&mut self) -> u8 {
        if self.t.is_none() {
            0
        } else {
            let now = Instant::now();
            let elapsed = now.duration_since(self.t.unwrap()).as_millis();
            let elapsed_ticks = elapsed / 16 / self.time_scale as u128;
            if elapsed_ticks > self.val.into() {
                self.val = 0;
                self.t = None;
                0
            } else {
                (self.val as u128 - elapsed_ticks).try_into().unwrap()
            }
        }
    }

    pub fn get_no_mod(&self) -> u8 {
        if self.t.is_none() {
            0
        } else {
            let now = Instant::now();
            let elapsed = now.duration_since(self.t.unwrap()).as_millis();
            let elapsed_ticks = elapsed / 16;
            if elapsed_ticks > self.val.into() {
                0
            } else {
                (self.val as u128 - elapsed_ticks).try_into().unwrap()
            }
        }
    }
}

#[derive(Debug)]
pub struct Memory {
    pub ram: [u8; 4096],
    pub stack: Vec<u16>,
    pub scratch: [u8; 32],
    pub display: [u8; 2048],
}

impl Memory {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut tmp = Self {
            ram: [0; 4096],
            stack: Vec::new(),
            scratch: [0; 32],
            display: [0; 2048],
        };

        // Nasty
        for (i, x) in FONT.iter().enumerate() {
            tmp.ram[i] = *x;
        }

        tmp
    }

    pub fn load(&mut self, buf: &[u8]) {
        for (i, c) in buf.iter().enumerate() {
            self.ram[0x200 + i] = *c;
        }
    }

    pub fn reload(&mut self, buf: &[u8]) {
        self.clear();
        self.load(buf);
    }

    fn clear(&mut self) {
        self.ram.map(|_| 0);
        self.scratch.map(|_| 0);
        self.display.map(|_| 0);
        self.stack.clear();
    }

    pub fn read_u16(&self, ip: u16) -> u16 {
        (self.ram[ip as usize] as u16) << 8 | (self.ram[ip as usize + 1] as u16)
    }

    pub fn read_u8(&self, ip: u16) -> u8 {
        self.ram[ip as usize]
    }

    pub fn write_u8(&mut self, ip: u16, v: u8) {
        self.ram[ip as usize] = v;
    }

    pub fn clear_display(&mut self) {
        self.display.map(|_| 0);
    }

    pub fn write_sprite(&mut self, x: u8, y: u8, data: &[u8]) -> bool {
        use display::DISPLAY_WIDTH;

        let mut unset = false;

        let rx = x % 64;
        let ry = y % 32;

        for (yi, chunk) in data.iter().enumerate() {
            let ryi = ry as usize + yi;
            if ryi < 32 {
                let p_data = chunk.reverse_bits();
                for xi in 0..8 {
                    let rxi = (rx + xi) as usize;
                    if rxi < 64 {
                        let pixel = p_data >> xi & 0x1;

                        if pixel & self.display[rxi + ryi * DISPLAY_WIDTH] == 0x1 {
                            unset = true;
                        }

                        self.display[rxi + ryi * DISPLAY_WIDTH] ^= pixel;
                    }
                }
            }
        }

        unset
    }
}

#[derive(Debug)]
pub struct System {
    mem: Memory,
    pc: u16, // Program Counter
    ir: u16, // Index Register
    //dt: u8,  // Delay Timer // TODO: maybe Instant computation for dt instead
    dt: Timer60Hz,
    st: u8, // Sound Timer
    registers: [u8; 16],
    cycle_delay_ms: Duration,
    halted: bool,
}

impl System {
    #[allow(clippy::new_without_default)]
    pub fn new(delay: u64) -> Self {
        Self {
            mem: Memory::new(),
            pc: 0x200,
            ir: 0,
            //dt: 0,
            dt: Timer60Hz::with_time_scale(delay / 2),
            st: 0,
            registers: [0; 16],
            cycle_delay_ms: Duration::from_millis(delay),
            halted: false,
        }
    }

    pub fn read_register(&self, rp: u8) -> u8 {
        if rp > 0xF {
            panic!("read_register: reg pointer > 0xF");
        }

        self.registers[rp as usize]
    }

    pub fn write_register(&mut self, rp: u8, value: u8) {
        if rp > 0xF {
            panic!("write_register: reg pointer > 0xF");
        }

        self.registers[rp as usize] = value;
    }

    fn reset(&mut self) {
        self.pc = 0x200;
        self.ir = 0;
        self.dt = Timer60Hz::new();
        self.st = 0;
        self.registers.map(|_| 0);
    }

    pub fn halt(&mut self) {
        self.halted = true;
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        self.mem.reload(data);
        self.reset();
    }

    pub fn read_decode(&self) -> (u16, (u8, u8, u8, u8), InstHelper) {
        let instruction: u16 = self.mem.read_u16(self.pc);

        let decoded = (
            ((instruction & 0xF000) >> 12) as u8,
            ((instruction & 0x0F00) >> 8) as u8,
            ((instruction & 0x00F0) >> 4) as u8,
            (instruction & 0x000F) as u8,
        );

        let helper = InstHelper {
            nnn: instruction & 0x0FFF,
            nn: (instruction & 0x00FF) as u8,
            x: ((instruction & 0x0F00) >> 8) as usize,
            y: ((instruction & 0x00F0) >> 4) as usize,
        };

        (instruction, decoded, helper)
    }

    pub fn run(&mut self, mut maybe_dis: Option<&mut dis::Disassembler>) {
        while !self.halted {
            if let Some(ref mut dis) = maybe_dis {
                dis.print_state(&self);
                dis.print_dis(&self);
            }

            thread::sleep(self.cycle_delay_ms);

            self.execute();
        }
    }

    fn execute(&mut self) {
        let (raw, decoded, helper) = self.read_decode();

        self.pc += 2;

        match decoded {
            (0, 0, 0xe, 0) => {
                self.mem.clear_display();
                display::write_display(&self.mem.display);
            }
            (0, 0, 0xe, 0xe) => {
                if let Some(v) = self.mem.stack.pop() {
                    self.pc = v;
                } else {
                    panic!("RET: stack underflow");
                }
            }
            (0, _, _, _) => self.halted = true, //std::process::exit(0),
            (1, _, _, _) => {
                self.pc = helper.nnn;
            }
            (2, _, _, _) => {
                self.mem.stack.push(self.pc);
                self.pc = helper.nnn;
            }
            (3, _, _, _) => {
                if self.registers[helper.x] == helper.nn {
                    self.pc += 2;
                }
            }
            (4, _, _, _) => {
                if self.registers[helper.x] != helper.nn {
                    self.pc += 2;
                }
            }
            (5, _, _, 0) => {
                if self.registers[helper.x] == self.registers[helper.y] {
                    self.pc += 2;
                }
            }
            (6, _, _, _) => self.registers[helper.x] = helper.nn,
            (7, _, _, _) => {
                let (val, _) = self.registers[helper.x].overflowing_add(helper.nn);
                self.registers[helper.x] = val;
            }
            (8, _, _, 0) => self.registers[helper.x] = self.registers[helper.y],
            (8, _, _, 1) => self.registers[helper.x] |= self.registers[helper.y],
            (8, _, _, 2) => self.registers[helper.x] &= self.registers[helper.y],
            (8, _, _, 3) => self.registers[helper.x] ^= self.registers[helper.y],
            (8, _, _, 4) => {
                let (result, carry_flag) =
                    self.registers[helper.x].overflowing_add(self.registers[helper.y]);

                self.registers[helper.x] = result;
                self.registers[0xf] = carry_flag as u8;
            }
            (8, _, _, 5) => {
                let (result, carry_flag) =
                    self.registers[helper.x].overflowing_sub(self.registers[helper.y]);

                self.registers[helper.x] = result;
                self.registers[0xf] = !carry_flag as u8;
            }
            (8, _, _, 6) => {
                self.registers[0xf] = self.registers[helper.y] & 0x1;

                self.registers[helper.x] = self.registers[helper.y] >> 1;
            }
            (8, _, _, 7) => {
                let (result, carry_flag) =
                    self.registers[helper.y].overflowing_sub(self.registers[helper.x]);

                self.registers[helper.x] = result;
                self.registers[0xf] = !carry_flag as u8;
            }
            (8, _, _, 0xe) => {
                self.registers[0xf] = (self.registers[helper.y] & 0x80) >> 7;

                self.registers[helper.x] = self.registers[helper.y] << 1;
            }
            (9, _, _, 0) => {
                if self.registers[helper.x] != self.registers[helper.y] {
                    self.pc += 2;
                }
            }
            (0xa, _, _, _) => self.ir = helper.nnn,
            (0xb, _, _, _) => self.pc = helper.nnn + self.registers[0] as u16,
            (0xc, _, _, _) => {
                self.registers[helper.x] = 255; // TODO: finish RND
            }
            (0xd, _, _, n) => {
                let mut buf = Vec::new();
                for i in 0..n as u16 {
                    buf.push(self.mem.read_u8(self.ir + i));
                }

                let result =
                    self.mem
                        .write_sprite(self.registers[helper.x], self.registers[helper.y], &buf);

                self.registers[0xf] = result as u8;

                display::write_display(&self.mem.display);
            }
            // TODO: finish HID instructions
            (0xe, _, 9, 0xe) => {
                if scan_key(self.registers[helper.x]) {
                    self.pc += 2;
                }
            }
            (0xe, _, 0xa, 1) => {
                if !scan_key(self.registers[helper.x]) {
                    self.pc += 2;
                }
            }
            (0xf, _, 0, 7) => self.registers[helper.x] = self.dt.get(),
            (0xf, _, 0, 0xa) => {
                self.registers[helper.x] = wait_key();
            }
            (0xf, _, 1, 5) => self.dt.set(self.registers[helper.x]),
            (0xf, _, 1, 8) => self.st = self.registers[helper.x],
            (0xf, _, 1, 0xe) => self.ir += self.registers[helper.x] as u16,
            (0xf, _, 2, 9) => self.ir = (self.registers[helper.x] as u16 & 0xF) * 5,
            (0xf, _, 3, 3) => {
                let x = self.registers[helper.x];
                self.mem.write_u8(self.ir + 2, x % 10);
                self.mem.write_u8(self.ir + 1, x / 10 % 10);
                self.mem.write_u8(self.ir, x / 10 / 10);
            }
            (0xf, _, 5, 5) => {
                for i in 0..=helper.x {
                    self.mem.write_u8(self.ir + i as u16, self.registers[i]);
                }

                self.ir += helper.x as u16 + 1;
            }
            (0xf, _, 6, 5) => {
                for i in 0..=helper.x {
                    //self.mem.write_u8(self.ir + i as u16, self.registers[i]);
                    self.registers[i] = self.mem.read_u8(self.ir + i as u16);
                }

                self.ir += helper.x as u16 + 1;
            }
            (0xf, 0xf, 0xf, 0xf) => self.halt(),

            (_, _, _, _) => panic!("Unhandled op: {:#06x}", raw),
        }
    }
}
