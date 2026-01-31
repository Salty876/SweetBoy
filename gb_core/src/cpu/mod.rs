use std::io::StderrLock;

use crate::{bus::Bus, cpu::{self, execute::execute}};

pub mod registers;
pub mod instructions;
mod execute;

use registers::Registers;
use instructions::Instruction;

pub struct Cpu {
    pub regs: Registers,
    pub pc: u16,
    pub sp: u16,
    pub bus: Bus,
    pub halted: bool,

    pub ime: bool,
    pub ime_scheduled: bool,

    pub fetch_pc: u16,
    pub fetch_pc_valid: bool,
    pub halt_bug: bool
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: Registers::new(),
            pc: 0x0000,
            sp: 0xFFFE,
            bus: Bus::new(),
            halted: false,

            ime: false,
            ime_scheduled: false,

            fetch_pc: 0,
            fetch_pc_valid: false,
            halt_bug: false,

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
        if self.halted {
            return;
        }

        

      self.begin_instruction();

    let mut opcode = self.fetch_u8();
    let prefixed = opcode == 0xCB;
    if prefixed {
        opcode = self.fetch_u8();
    }

    let instr = Instruction::decode(opcode, prefixed)
        .unwrap_or_else(|| panic!("Unknown opcode: 0x{:02X} (prefixed={})", opcode, prefixed));

    execute(self, instr, prefixed);

    self.end_instruction();

       
    if self.ime_scheduled {
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
        // temporary simple mapping: copy to 0x0000...
        let len = rom.len().min(0x8000); // ROM0+ROMX basic
        self.bus.memory[..len].copy_from_slice(&rom[..len]);
        self.pc = 0x0100; // skip boot (or set 0x0000 if using boot rom)
    }

    pub fn run_steps(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
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

        let (new_value, did_overflow) = self.regs.a_reg.overflowing_sub(value);
        // self.registers.f_reg.z_flag = new_value == 0;
        // self.registers.f_reg.n_flag = true;
        // self.registers.f_reg.c_flag = did_overflow;
        // self.registers.f_reg.h_flag = (self.registers.a_reg & 0xF) < (value & 0xF);
        self.regs.set_z(new_value == 0);
        self.regs.set_n(true);
        self.regs.set_carry(did_overflow);
        self.regs.set_hc((self.regs.a_reg & 0xF) < (value & 0xF));
        new_value
    }
}

