#[derive(Clone)]
pub struct Bus {
    pub memory: [u8; 0x10000],
    ie: u8,
    i_flag: u8,
}

impl Bus {
    pub fn new() -> Self {
        Self { memory: [0; 0x10000], ie: 0, i_flag: 0 }
    }

    #[inline]
    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF0F => self.i_flag,
            0xFFFF => self.ie,
            _ => self.memory[addr as usize]
        }
        
    }

    #[inline]
    pub fn write_byte(&mut self, addr: u16, value: u8) {

        match addr {
        0xFF02 => {
            // Serial output: write to 0xFF01, then 0xFF02 with 0x81 to print the character in 0xFF01.
            // For testing, we can just print to console.
            if value == 0x81 {
                let char_to_print = self.read_byte(0xFF01) as char;
                print!("{}", char_to_print);
            }
        }

        0xFF01 => {
            // Serial data register: just store the value, no actual serial emulation needed for tests.
            self.memory[0xFF01] = value;
        }
        
        0xFF0F => {
            self.i_flag = value;
        }
        0xFFFF => {
            self.ie = value;
        }
        _ => {
            self.memory[addr as usize] = value;
        }
    }
        
    }

    pub fn request_interrupt(&mut self, bit: u8) {
        self.i_flag |= bit & 0x1F;
    }

    pub fn clear_interrupt(&mut self, bit: u8) {
        self.i_flag &= !(bit & 0x1F);
    }

    #[inline]
    pub fn read_word(&self, addr: u16) -> u16 {
        let lo = self.read_byte(addr) as u16;
        let hi = self.read_byte(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    #[inline]
    pub fn write_word(&mut self, addr: u16, value: u16) {
        let lo = (value & 0x00FF) as u8;
        let hi = (value >> 8) as u8;
        self.write_byte(addr, lo);
        self.write_byte(addr.wrapping_add(1), hi);
    }
}



