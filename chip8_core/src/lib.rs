use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const NUM_REGS: usize = 16;
const RAM_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;

const FONT_SIZE: usize = 80;
const FONTSET: [u8; FONT_SIZE] =  [
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
0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8
}

// impl Default for Emu {
//     fn default() -> Self {
//         Self {
//             pc: START_ADDR,
//             ram: [0; RAM_SIZE],
//             screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
//             v_reg: [0; NUM_REGS],
//             i_reg: 0,
//             sp: 0,
//             stack: [0; STACK_SIZE],
//             keys: [false; NUM_KEYS],
//             dt: 0,
//             st: 0
//         }
//     }
// }

impl Emu {
    pub fn new() -> Self {
        let mut new_emu: Emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0
        };
        new_emu.ram[..FONT_SIZE].copy_from_slice(&FONTSET);

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
        self.ram[..FONT_SIZE].copy_from_slice(&FONTSET);
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Decode & Execute
        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;

        let op = (higher_byte << 8) | lower_byte;
        op
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // Do Nothing - 0000
            (0, 0, 0, 0) => return,
            // Reset screen buffer - 00E0
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            // Return from subroutine - 00EE
            (0, 0, 0xE, 0xE) => {
                let return_addr = self.pop();
                self.pc = return_addr;
            },
            // Jump - 1NNN
            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            },
            // Call Subroutine - 2NNN
            (2, _, _, _) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            },
            // Skip if VX == NN - 3XNN
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            },
            // Skip if VX != NN - 4XNN
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            },
            // Skip if VX == VY - 5XY0
            (5, _, _, _) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            },
            // Set VX = NN - 6XNN
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            },
            // Add NN to VX - 7XNN
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            },
            // Set VX = VY - 8XY0
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            },
            // Bitwise OR - 8XY1
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            }
            // Bitwise AND - 8XY2
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            }
            // Bitwise XOR - 8XY3
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            },
            // VX += VY - 8XY4
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // VX = VX - VY - 8XY5
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                
                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // VX >>= 1 - 8XY6
            (8, _, _, 6) => {
                // Can be made configurable
                let x = digit2 as usize;
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            },
            // VX = VY - VX - 8XY7
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // VX <<= 1 - 8XYE
            (8, _, _, 0xE) => {
                // Can be made configurable
                let x = digit2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            },
            // Skip if VX != VY - 9XY0
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            },
            // Setting I-register to NNN - ANNN
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            }, 
            // Jump to V0 + NNN - BNNN
            (0xB, _, _, _) => {
                // Can be configurable
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            },
            // VX = RAND & NN - CXNN
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            },
            // Display - DXYN
            (0xD, _, _, _) => {
                // get x & y coords
                let x_coord = digit2 as usize;
                let y_coord = digit3 as usize;
                // Height of sprite
                let num_rows = digit4;
                // Keep track if the pixel flipped
                let mut flipped = false;

                for y_line in 0..num_rows {
                    // Determine the address where raw data is present
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line as usize) as usize % SCREEN_HEIGHT;
                            // Idx of pixel
                            let idx = x + SCREEN_WIDTH * y;
                            // Checking if the bit will be flipped
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                // Populate VF register
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            },
            // Skip if key pressed - EX9E
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            },
            // Skip if not pressed - EXA1
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            },
            // Inspect the delay timer - FX07
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            },
            // Wait for input - FX0A
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    // Redo Opcode
                    self.pc -= 2;
                }
            },
            // Set delay timer - FX15
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            },
            // Set sound timer - FX18
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            },
            // Adding I += VX - FX1E
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            },
            // Set font address - FX29
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            },
            // Binary Encoded Decimal - FX33
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;

                let hundereth = (vx / 100.0).floor() as u8;
                let tenth = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundereth;
                self.ram[(self.i_reg + 1) as  usize] = tenth;
                self.ram[(self.i_reg + 2) as usize] = ones;
            },
            // Store in RAM - FX55
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[i + idx] = self.ram[idx];
                }
            },
            // Fetch from RAM to VX - FX65
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
            },
            (_, _, _, _) => unimplemented!("Unimplemented opcaode: {}", op),
        }
    }

    pub fn timer_tick(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // Beep
            }
            self.st -= 1;
        }
    }
}