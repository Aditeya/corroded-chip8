use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const START_ADDR: u16 = 0x200;

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
    0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

pub struct Chip8 {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip_8 = Self {
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

        chip_8.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        chip_8
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        println!("opcode: {op:04X}");

        // Decode & Execute
        self.execute(op);
    }

    pub fn tick_timers(&mut self) {
        self.dt = self.dt.saturating_sub(1);

        if self.st == 1 {
            // TODO: add beep
        }
        self.st = self.st.saturating_sub(1);
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        // F    F    F    F
        // 1111 1111 1111 1111
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return, // NOP
            (0, 0, 0xE, 0) => { // CLS
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            (0, 0, 0xE, 0xE) => { // RET
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }
            (1, _, _, _) => { // JMP NNN
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }
            (2, _, _, _) => { // CALL NNN
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }
            (3, _, _, _) => { // SKIP VX == NN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }
            (4, _, _, _) => { // SKIP VX != NN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }
            (5, _, _, 0) => { // SKIP VX == VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }
            (6, _, _, _) => { // VX = NN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            }
            (7, _, _, _) => { // VX += NN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }
            (8, _, _, _) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                match digit4 {
                    0 => self.v_reg[x] = self.v_reg[y],  // VX = VY
                    1 => self.v_reg[x] |= self.v_reg[y], // VX |= VY
                    2 => self.v_reg[x] &= self.v_reg[y], // VX &= VY
                    3 => self.v_reg[x] ^= self.v_reg[y], // VX ^= VY
                    4 => { // VX += VY
                        let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                        let new_vf = u8::from(carry);

                        self.v_reg[x] = new_vx;
                        self.v_reg[0xF] = new_vf;
                    }
                    5 => { // VX -= VY
                        let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                        let new_vf = u8::from(!borrow);

                        self.v_reg[x] = new_vx;
                        self.v_reg[0xF] = new_vf;
                    }
                    6 => { // VX >>= 1
                        let lsb = self.v_reg[x] & 1;
                        self.v_reg[x] >>= 1;
                        self.v_reg[0xF] = lsb;
                    }
                    7 => { // VX = VY - VX
                        let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                        let new_vf = u8::from(!borrow);

                        self.v_reg[x] = new_vx;
                        self.v_reg[0xF] = new_vf;
                    }
                    0xE => { // VX <<= 1
                        let msb = (self.v_reg[x] >> 7 ) & 1;
                        self.v_reg[x] <<= 1;
                        self.v_reg[0xF] = msb;
                    }
                    _ => unimplemented!("Unimplemented opcode: {op:04X}"),
                }
            }
            (9, _, _, 0) => { // SKIP VX != VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }
            (0xA, _, _, _) => { // I = NNN
                let nnn = op & 0x0FFF;
                self.i_reg = nnn;
            }
            (0xB, _, _, _) => { // JMP (V0 + NNN)
                let nnn = op & 0x0FFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }
            (0xC, _, _, _) => { // VX = rand() & NN
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            }
            (0xD, _, _, num_rows) => { // DRAW
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;
                let mut flipped = false;

                for y_line in 0..num_rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            let idx = x + SCREEN_WIDTH * y;
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                self.v_reg[0xF] = u8::from(flipped);
            }
            (0xE, _, 9, 0xE) => { // SKIP KEY PRESS
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 1) => { // SKIP KEY PRESS
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            }
            (0xF, _, 0, 7) => { // VX = DT
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            }
            (0xF, _, 0, 0xA) => { // WAIT KEY
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
                    // Redo opcode
                    self.pc -= 2;
                }
            }
            (0xF, _, 1, 5) => { // DT = VX
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            }
            (0xF, _, 1, 8) => { // ST = VX
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            }
            (0xF, _, 1, 0xE) => { // I += VX
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }
            (0xF, _, 2, 9) => { // I = FONT
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            }
            (0xF, _, 3, 3) => { // BCD
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;

                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx / 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg+1) as usize] = tens;
                self.ram[(self.i_reg+2) as usize] = ones;
            }
            (0xF, _, 5, 5) => { // STORE V0-VX
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i+idx] = self.v_reg[idx];
                }
            }
            (0xF, _, 6, 5) => { // LOAD V0-VX
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i+idx];
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {op:04X}"),
        }
    }
}

impl Chip8 {
    // TODO: handle safety
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    // TODO: handle safety
    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}
