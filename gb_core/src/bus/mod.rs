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



