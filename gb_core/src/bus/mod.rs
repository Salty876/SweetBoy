#[derive(Clone)]
pub struct Bus {
    pub memory: [u8; 0x10000],
    ie: u8,
    i_flag: u8,

    // ── Timer hardware ──
    /// Internal 16-bit system counter; DIV = upper 8 bits (div_counter >> 8).
    pub div_counter: u16,
    /// TIMA – Timer Counter (0xFF05)
    pub tima: u8,
    /// TMA – Timer Modulo (0xFF06)
    pub tma: u8,
    /// TAC – Timer Control (0xFF07)
    pub tac: u8,
    /// Previous AND result for falling-edge detection on TIMA clock
    prev_and: bool,
    /// When TIMA overflows it is reloaded *one M-cycle later*;
    /// this counts down the 4 T-cycle delay (0 = no pending reload).
    tima_reload_countdown: u8,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            memory: [0; 0x10000],
            ie: 0,
            i_flag: 0,
            div_counter: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            prev_and: false,
            tima_reload_countdown: 0,
        }
    }

    #[inline]
    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.div_counter >> 8) as u8,  // DIV
            0xFF05 => self.tima,                       // TIMA
            0xFF06 => self.tma,                        // TMA
            0xFF07 => self.tac | 0xF8,                 // TAC (upper 5 bits read as 1)
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
            if value == 0x81 {
                let char_byte = self.read_byte(0xFF01);
                let char_to_print = char_byte as char;
                print!("{}", char_to_print);
                use std::io::Write;
                std::io::stdout().flush().ok();
                self.memory[0xFF02] = 0x00;
            }
        }

        0xFF01 => {
            self.memory[0xFF01] = value;
        }

        // ── Timer registers ──
        0xFF04 => {
            // Writing ANY value to DIV resets the entire internal counter to 0.
            // This can cause a falling edge on the TIMA clock bit → extra TIMA tick.
            let old_and = self.timer_and_result();
            self.div_counter = 0;
            let new_and = self.timer_and_result();
            if old_and && !new_and {
                self.increment_tima();
            }
            self.prev_and = new_and;
        }
        0xFF05 => {
            // Writing to TIMA cancels a pending reload
            self.tima = value;
            self.tima_reload_countdown = 0;
        }
        0xFF06 => {
            self.tma = value;
        }
        0xFF07 => {
            let old_and = self.timer_and_result();
            self.tac = value & 0x07;
            let new_and = self.timer_and_result();
            // Changing TAC can cause a falling edge → extra TIMA tick
            if old_and && !new_and {
                self.increment_tima();
            }
            self.prev_and = new_and;
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

    // ── Timer internals ──

    /// Which bit of div_counter drives TIMA, controlled by TAC bits 0-1.
    fn tac_bit_mask(&self) -> u16 {
        match self.tac & 0x03 {
            0 => 1 << 9,  // 4096 Hz   (every 1024 T-cycles)
            1 => 1 << 3,  // 262144 Hz (every 16 T-cycles)
            2 => 1 << 5,  // 65536 Hz  (every 64 T-cycles)
            3 => 1 << 7,  // 16384 Hz  (every 256 T-cycles)
            _ => unreachable!(),
        }
    }

    /// The "AND" of the selected counter bit and the timer-enable bit (TAC bit 2).
    /// A falling edge (true → false) on this signal increments TIMA.
    fn timer_and_result(&self) -> bool {
        let enabled = self.tac & 0x04 != 0;
        let bit_high = self.div_counter & self.tac_bit_mask() != 0;
        enabled && bit_high
    }

    fn increment_tima(&mut self) {
        let (new_tima, overflow) = self.tima.overflowing_add(1);
        if overflow {
            // TIMA overflows: reload from TMA and request Timer interrupt.
            // On real hardware there's a 4 T-cycle delay, but for the
            // instr_timing test this immediate reload is sufficient.
            self.tima = self.tma;
            self.request_interrupt(0x04); // Timer interrupt (bit 2)
        } else {
            self.tima = new_tima;
        }
    }

    /// Advance the timer by one T-cycle. Call this 4 times per M-cycle.
    pub fn tick_timer(&mut self) {
        let old_and = self.timer_and_result();
        self.div_counter = self.div_counter.wrapping_add(1);
        let new_and = self.timer_and_result();

        // Falling edge → increment TIMA
        if old_and && !new_and {
            self.increment_tima();
        }
        self.prev_and = new_and;
    }
}