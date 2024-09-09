use std::usize;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const NUM_KEYS: usize = 16;
const STACK_SIZE: usize = 16;

const START_ADDR: u16 = 0x200;

pub struct Emu {
    // program counter, incrementing through the game
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    // V registers -- for game ops
    v_reg: [u8; NUM_REGS],
    // I register -- for RAM indexing
    i_reg: u16,

    // program stack and stack pointer
    sp: u16,
    stack: [u16; STACK_SIZE],

    // keys pressed state
    keys: [bool; NUM_KEYS],

    // delay and sound timers
    dt: u8,
    st: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = start + data.len();
        self.ram[start..end].copy_from_slice(data)
    }

    pub fn get_display(&self) -> &[bool] {
        // println!("{:?}", self.screen);
        &self.screen
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            if self.st == 1 {
                // TODO: beep
            }
            self.st -= 1;
        }
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    // ----------- internals ----------- //

    // stack mgmt
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    // grabs the next operation from the game
    fn fetch(&mut self) -> u16 {
        let high = self.ram[(self.pc - 2) as usize] as u16;
        let low = self.ram[(self.pc - 1) as usize] as u16;
        let op = (high << 8) | low;
        self.pc += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        // println!("{op:X}");
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        // see http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
        match (digit1, digit2, digit3, digit4) {
            // no-op
            (0, 0, 0, 0) => {}

            // clear screen
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH],

            // return from subroutine
            (0, 0, 0xE, 0xE) => self.pc = self.pop(),

            // jump to nnn
            (1, _, _, _) => self.pc = op_as_address(op),

            // call nnn, putting current pc on stack
            (2, _, _, _) => {
                self.push(self.pc);
                self.pc = op_as_address(op);
            }

            // skips
            (3, x, _, _) => {
                if self.v_reg[x as usize] == op_as_u8(op) {
                    self.pc += 2;
                }
            }
            (4, x, _, _) => {
                if self.v_reg[x as usize] != op_as_u8(op) {
                    self.pc += 2;
                }
            }
            (5, x, y, 0) => {
                if self.v_reg[x as usize] == self.v_reg[y as usize] {
                    self.pc += 2;
                }
            }

            // register work
            (6, x, _, _) => self.v_reg[x as usize] = op_as_u8(op),
            (7, x, _, _) => self.v_reg[x as usize] += op_as_u8(op),
            (8, x, y, 0) => self.v_reg[x as usize] = self.v_reg[y as usize],
            (8, x, y, 1) => self.v_reg[x as usize] |= self.v_reg[y as usize],
            (8, x, y, 2) => self.v_reg[x as usize] &= self.v_reg[y as usize],
            (8, x, y, 3) => self.v_reg[x as usize] ^= self.v_reg[y as usize],
            (8, x, y, 4) => {
                if self.v_reg[x as usize] as usize + self.v_reg[y as usize] as usize > 255 {
                    self.v_reg[0xF] = 1;
                }
                self.v_reg[x as usize] += self.v_reg[y as usize];
            }
            (8, x, y, 5) => {
                self.v_reg[0xF] = if self.v_reg[x as usize] > self.v_reg[y as usize] {
                    1
                } else {
                    0
                };
                self.v_reg[x as usize] -= self.v_reg[y as usize];
            }

            // set I to nnn
            (0xA, _, _, _) => self.i_reg = op_as_address(op),

            // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
            (0xD, x, y, num_rows) => {
                let ram_start = self.i_reg as usize;
                let screen_start = x_y_as_byte_offset(self.v_reg[x as usize], self.v_reg[y as usize]);

                // walk the bytes of the screen and targeted ram
                for n in 0..(num_rows as usize) {
                    println!("ram at {} => {:?}", ram_start + n, self.ram[ram_start + n]);
                    self.v_reg[0xF] = 0;

                    // XOR, and set VF = 1 if a screen byte is erased
                    match (self.screen[screen_start + n], self.ram[ram_start + n]) {
                        (false, u8::MAX) => {
                            self.screen[screen_start + n] = true;
                        }
                        (true, u8::MAX) => {
                            self.screen[screen_start + n] = false;
                            self.v_reg[0xF] = 1;
                        }
                        _ => {}
                    }
                }
            }

            // Set I = I + Vx.
            (0xF, x, 1, 0xE) => self.i_reg += self.v_reg[x as usize] as u16,

            // Set I = location of sprite for digit Vx.
            (0xF, x, 2, 9) => self.i_reg = self.ram[x as usize * 5] as u16,

            // Store registers V0 through Vx in memory starting at location I.
            (0xF, x, 5, 5) => {
                for n in 0..(x as usize) {
                    self.ram[self.i_reg as usize + n] = self.v_reg[n];
                }
            }

            // Read registers V0 through Vx from memory starting at location I.
            (0xF, x, 6, 5) => {
                for n in 0..(x as usize) {
                    self.v_reg[n] = self.ram[self.i_reg as usize + n];
                }
            }

            // maths
            _ => unimplemented!("{:X}", op),
        }
    }
}

impl Default for Emu {
    fn default() -> Self {
        Emu::new()
    }
}

#[inline]
fn op_as_u8(op: u16) -> u8 {
    (op & 0xFF) as u8
}

#[inline]
fn op_as_address(op: u16) -> u16 {
    op & 0xFFF
}

#[inline]
fn x_y_as_byte_offset(x: u8, y: u8) -> usize {
    SCREEN_WIDTH * y as usize + x as usize
}
