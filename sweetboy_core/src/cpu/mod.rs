use crate::{bus::Bus, cpu::execute::execute};
use serde::{Serialize, Deserialize};

pub mod registers;
pub mod instructions;
mod execute;

use registers::Registers;
use instructions::Instruction;

#[derive(Serialize, Deserialize)]
pub struct Cpu {
    pub regs: Registers,
    pub pc: u16,
    pub sp: u16,
    pub cycles: u64,
    pub bus: Bus,
    pub halted: bool,

    pub ime: bool,
    pub ime_scheduled: bool,
    pub ime_pending: bool,

    pub fetch_pc: u16,
    pub fetch_pc_valid: bool,
    pub halt_bug: bool,
    pub stopped: bool,

    pub last_cycle_timestamp: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: Registers::new(),
            pc: 0x0100,
            sp: 0xFFFE,
            cycles: 0,
            bus: Bus::new(),
            halted: false,

            ime: false,
            ime_scheduled: false,
            ime_pending: false,

            fetch_pc: 0,
            fetch_pc_valid: false,
            halt_bug: false,
            stopped: false,
            last_cycle_timestamp: 0,

        }
    }


    #[inline]
    fn begin_instruction(&mut self) {
        self.fetch_pc = self.pc;
        self.fetch_pc_valid = true;
    }

    #[inline]
    fn end_instruction(&mut self) {
        self.pc = self.fetch_pc;
        self.fetch_pc_valid = false;
    }

    #[inline]
    pub fn fetch_u8(&mut self) -> u8 {
        if !self.fetch_pc_valid {
            self.begin_instruction();
        }

        let v = self.bus.read_byte(self.fetch_pc);

        // HALT bug: suppress ONE increment on the *next instruction's first fetch*
        if self.halt_bug {
            self.halt_bug = false;
        } else {
            self.fetch_pc = self.fetch_pc.wrapping_add(1);
        }

        v
    }

    #[inline]
    pub fn fetch_u16(&mut self) -> u16 {
        let lo = self.fetch_u8() as u16;
        let hi = self.fetch_u8() as u16;
        lo | (hi << 8)
    }



    // Inturupys helper funcs
    fn pending_mask(&self) -> u8 {
        let ie = self.bus.read_byte(0xFFFF) & 0x1F;
        let iflag = self.bus.read_byte(0xFF0F) & 0x1F;
        ie & iflag
    }

    fn highest_priority(mask: u8) -> Option<(u8, u16)> {
        if mask & 0x01 != 0 { return Some((0x01, 0x0040)); } // VBlank
        if mask & 0x02 != 0 { return Some((0x02, 0x0048)); } // LCD
        if mask & 0x04 != 0 { return Some((0x04, 0x0050)); } // Timer
        if mask & 0x08 != 0 { return Some((0x08, 0x0058)); } // Serial
        if mask & 0x10 != 0 { return Some((0x10, 0x0060)); } // Joypad
        None
    }


    pub fn step(&mut self) {

        let pending = self.pending_mask();

        if self.halted && pending != 0 {
            self.halted = false;
        }

        if self.stopped || self.halted {
            // Even while halted/stopped, PPU and timer still run (4 T-cycles / 1 M-cycle)
            self.bus.tick(4);
            return;
        }


        if self.service_interrupts() {
            // Interrupt dispatch takes 20 T-cycles (5 M-cycles)
            self.bus.tick(20);
            return;
            }

        

        self.begin_instruction();

        let opcode = self.fetch_u8();
        
        // prefixed instruction

        if opcode == 0xCB {
            let cb_opcode = self.fetch_u8();
            Self::exec_cb(self, cb_opcode);
            // CB instructions: 8 T-cycles, or 16 for (HL) variants
            let cb_cycles: u8 = if (cb_opcode & 0x07) == 0x06 {
                // BIT n,(HL) is 12 cycles; other (HL) ops are 16
                if (cb_opcode >> 6) == 1 { 12 } else { 16 }
            } else {
                8
            };
            self.last_cycle_timestamp = cb_cycles;
            self.bus.tick(cb_cycles as u16);
            self.end_instruction();
            return;
        }

        let instr = Instruction::decode(opcode, false)
            .unwrap_or_else(|| panic!("Unknown opcode: 0x{:02X} (prefixed={}) PC={:04X}", opcode, false, self.pc));
        let cycles = execute(self, instr, false);
        self.last_cycle_timestamp = cycles;
        self.bus.tick(cycles as u16);

        let was_ei = matches!(instr, Instruction::EI);

        self.end_instruction();

        if self.ime_scheduled && !was_ei {
            self.ime = true;
            self.ime_scheduled = false;
        }

        }

    #[inline]
    pub fn next_byte(&mut self) -> u8 {
        self.fetch_u8()
    }

    #[inline]
    pub fn next_word(&mut self) -> u16 {
        self.fetch_u16()
    }

    #[inline]
    pub fn push_word(&mut self, value: u16) {
        // stack grows down
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, (value >> 8) as u8); // hi
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, (value & 0xFF) as u8); // lo
    }

    #[inline]
    pub fn pop_word(&mut self) -> u16 {
        let lo = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let hi = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        (hi << 8) | lo
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.bus.load_rom(rom);
        self.pc = 0x0100; // skip boot (or set 0x0000 if using boot rom)
    }

    pub fn run_steps(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }

    /// Advance the internal timer by `t_cycles` T-cycles.
    #[allow(dead_code)]
    fn tick_timer(&mut self, t_cycles: u32) {
        for _ in 0..t_cycles {
            self.bus.tick_timer();
        }
    }

    fn service_interrupts(&mut self) -> bool {
        let pending = self.pending_mask();

        if self.ime && pending != 0 {
            if let Some((bit, addr)) = Cpu::highest_priority(pending) {
                // Clear IF flag
                let iflag = self.bus.read_byte(0xFF0F);
                self.bus.write_byte(0xFF0F, iflag & !bit);

                // Disable IME
                self.ime = false;
                self.ime_scheduled = false;
                self.halted = false;

                // Push PC to stack
                self.push_word(self.pc);

                // Jump to interrupt vector
                self.pc = addr;

                self.fetch_pc = self.pc;
                self.fetch_pc_valid = false;

                return true;
            }
        }

        false
    }

    pub fn add(&mut self, value: u8) -> u8{

        let (new_value, did_overflow) = self.regs.a_reg.overflowing_add(value);
        // self.registers.f_reg.z_flag = new_value == 0;
        // self.registers.f_reg.n_flag = false;
        // self.registers.f_reg.c_flag = did_overflow;
        // self.registers.f_reg.h_flag = (self.registers.a_reg & 0xF) + (value & 0xF) > 0xF;
        self.regs.set_z(new_value == 0);
        self.regs.set_n(false);
        self.regs.set_carry(did_overflow);
        self.regs.set_hc((self.regs.a_reg & 0xF) + (value & 0xF) > 0xF);
        new_value
    }

    pub fn add_hl_rr(&mut self, register: u16) -> u16{
        let hl = self.regs.get_hl();

        // u32 helps test cleanly
        let sum = (hl as u32) + (register as u32);
        let res = (sum & 0xFFFF) as u16;


        // Set flags - ADD HL,rr only affects N, H, C (not Z)
        // Z flag remains unchanged
        self.regs.set_n(false);
        self.regs.set_hc(((hl & 0x0FFF) + (register & 0x0FFF)) > 0x0FFF);
        self.regs.set_carry(sum > 0xFFFF);

        return res;
    }

    pub fn sub(&mut self, value: u8) -> u8{
        let a  = self.regs.a();

        let (new_value, borrow) = a.overflowing_sub(value);
        // self.registers.f_reg.z_flag = new_value == 0;
        // self.registers.f_reg.n_flag = true;
        // self.registers.f_reg.c_flag = did_overflow;
        // self.registers.f_reg.h_flag = (self.registers.a_reg & 0xF) < (value & 0xF);
        self.regs.set_z(new_value == 0);
        self.regs.set_n(true);
        self.regs.set_carry(borrow);
        self.regs.set_hc((a & 0xF) < (value & 0xF));
        new_value
    }

    // CB Shi
    fn read_cb(&mut self, reg: u8) -> u8 {
        match reg {
            0x00 => self.regs.b(),
            0x01 => self.regs.c(),
            0x02 => self.regs.d(),
            0x03 => self.regs.e(),
            0x04 => self.regs.h(),
            0x05 => self.regs.l(),
            0x06 => {
                let addr = self.regs.get_hl();
                self.bus.read_byte(addr)
            }
            0x07 => self.regs.a(),
            _ => panic!("Invalid CB register code: 0x{:02X}", reg),
        }
    }

    fn write_cb(&mut self, reg: u8, value: u8) {
        match reg {
            0x00 => self.regs.set_b(value),
            0x01 => self.regs.set_c(value),
            0x02 => self.regs.set_d(value),
            0x03 => self.regs.set_e(value),
            0x04 => self.regs.set_h(value),
            0x05 => self.regs.set_l(value),
            0x06 => {
                let addr = self.regs.get_hl();
                self.bus.write_byte(addr, value);
            }
            0x07 => self.regs.set_a(value),
            _ => panic!("Invalid CB register code: 0x{:02X}", reg),
        }
    }

   fn exec_cb(cpu: &mut Cpu, op: u8) {
        let r = op & 0x07;
        let group = op >> 6;

        match group {
            0 => {
                let sub = (op >> 3) & 0x07;
                let x = cpu.read_cb( r);

                let (res, carry) = match sub {
                    0 => { // RLC
                        let c = (x >> 7) & 1;
                        ((x << 1) | c, c != 0)
                    }
                    1 => { // RRC
                        let c = x & 1;
                        ((x >> 1) | (c << 7), c != 0)
                    }
                    2 => { // RL
                        let cin = if cpu.regs.get_carry() { 1 } else { 0 };
                        let c = (x >> 7) & 1;
                        ((x << 1) | cin, c != 0)
                    }
                    3 => { // RR
                        let cin = if cpu.regs.get_carry() { 0x80 } else { 0 };
                        let c = x & 1;
                        ((x >> 1) | cin, c != 0)
                    }
                    4 => { // SLA
                        let c = (x >> 7) & 1;
                        (x << 1, c != 0)
                    }
                    5 => { // SRA (keep msb)
                        let c = x & 1;
                        ((x >> 1) | (x & 0x80), c != 0)
                    }
                    6 => { // SWAP
                        let hi = x >> 4;
                        let lo = x & 0x0F;
                        ((lo << 4) | hi, false)
                    }
                    7 => { // SRL
                        let c = x & 1;
                        (x >> 1, c != 0)
                    }
                    _ => (x, false),
                };

                cpu.write_cb( r, res);

                cpu.regs.set_z(res == 0);
                cpu.regs.set_n(false);
                cpu.regs.set_hc(false);
                cpu.regs.set_carry(carry);
            }

            1 => {
                let bit = (op >> 3) & 0x07;
                let x = cpu.read_cb( r);
                let zero = (x & (1 << bit)) == 0;

                cpu.regs.set_z(zero);
                cpu.regs.set_n(false);
                cpu.regs.set_hc(true);
            }

            2 => {
                let bit = (op >> 3) & 0x07;
                let x = cpu.read_cb( r);
                let res = x & !(1 << bit);
                cpu.write_cb( r, res);
            }

            3 => {
                let bit = (op >> 3) & 0x07;
                let x = cpu.read_cb( r);
                let res = x | (1 << bit);
                cpu.write_cb( r, res);
            }

            _ => {}
        }
}

}

