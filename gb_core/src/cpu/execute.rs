use core::task;
use std::result;

use crate::cpu;

use super::{Cpu};
use super::instructions::*;


// Helper functions for conditional and getting targets and daa

fn condition(test: JumpTest, cpu: &Cpu) -> bool {
    match test {
        JumpTest::NotZero => !cpu.regs.get_z(),
        JumpTest::Zero => cpu.regs.get_z(),
        JumpTest::NotCarry => !cpu.regs.get_carry(),
        JumpTest::Carry => cpu.regs.get_carry(),
        JumpTest::Always => true,
    }
}

fn read_u8_target(cpu: &mut Cpu, t: ArithmeticTarget) -> u8 {
    match t {
        ArithmeticTarget::A => cpu.regs.a(),
        ArithmeticTarget::B => cpu.regs.b(),
        ArithmeticTarget::C => cpu.regs.c(),
        ArithmeticTarget::D => cpu.regs.d(),
        ArithmeticTarget::E => cpu.regs.e(),
        ArithmeticTarget::H => cpu.regs.h(),
        ArithmeticTarget::L => cpu.regs.l(),
        ArithmeticTarget::HLI => cpu.bus.read_byte(cpu.regs.get_hl()),
        ArithmeticTarget::D8 => cpu.next_byte(),
    }
}

fn daa(cpu: &mut Cpu) {
    let mut a = cpu.regs.a();
    let mut adjust = 0;
    let mut carry = false;

    if cpu.regs.get_hc() || (!cpu.regs.get_n() && (a & 0x0F) > 9) {
        adjust |= 0x06;
    }
    if cpu.regs.get_carry() || (!cpu.regs.get_n() && a > 0x99) {
        adjust |= 0x60;
        carry = true;
    }

    if cpu.regs.get_n() {
        a = a.wrapping_sub(adjust);
    } else {
        a = a.wrapping_add(adjust);
    }

    cpu.regs.set_a(a);
    cpu.regs.set_z(a == 0);
    cpu.regs.set_hc(false);
    cpu.regs.set_carry(carry);
}




pub fn execute(cpu: &mut Cpu, instr: Instruction, prefixed: bool) {
    match instr {
        Instruction::NOP => {
            // do nothing, fetch_pc already advanced by opcode fetch
        }

        Instruction::HALT => {
            let pending = cpu.pending_mask();

            if !cpu.ime && pending != 0 {
                cpu.halt_bug = true;
                cpu.halted = false;
            } else {
                cpu.halted = true;
            }
            // do not touch pc or fetch_pc
        }

        Instruction::JP(test) => {
            let cond = condition(test, cpu);
            let target = cpu.next_word(); // always consume operands
            if cond {
                cpu.fetch_pc = target;
            }
        }

        Instruction::JR(test) => {
            let cond = condition(test, cpu);
            let off = cpu.next_byte() as i8;
            if cond {
                cpu.fetch_pc = cpu.fetch_pc.wrapping_add(off as u16);
            }
        }

        Instruction::CALL(test) => {
            let cond = condition(test, cpu);
            let target = cpu.next_word(); // always consume operands
            if cond {
                let ret = cpu.fetch_pc;     // next instruction address
                cpu.push_word(ret);
                cpu.fetch_pc = target;
            }
        }

        Instruction::RET(test) => {
            let cond = condition(test, cpu);
            if cond {
                let addr = cpu.pop_word();
                cpu.fetch_pc = addr;
            }
        }

        Instruction::ADD(target) => {
            let value = read_u8_target(cpu, target);
            let new_value = cpu.add(value);
            cpu.regs.set_a(new_value);
        }

        Instruction::ADC(target) => {
            let carry = if cpu.regs.get_carry() { 1 } else { 0 };
            let value = read_u8_target(cpu, target);
            let a = cpu.regs.a();

            let result = a.wrapping_add(value).wrapping_add(carry);
            cpu.regs.set_a(result);

            cpu.regs.set_z(result == 0);
            cpu.regs.set_n(false);
            cpu.regs.set_hc(((a & 0x0F) + (value & 0x0F) + carry) > 0x0F);
            cpu.regs.set_carry((a as u16) + (value as u16) + (carry as u16) > 0xFF);
        }

        Instruction::ADD16(target) => {
            let value = match target {
                Add16Target::BC => cpu.regs.get_bc(),
                Add16Target::DE => cpu.regs.get_de(),
                Add16Target::HL => cpu.regs.get_hl(),
                Add16Target::SP => cpu.sp,
            };
            let new_val = cpu.add_hl_rr(value);
            cpu.regs.set_hl(new_val);
        }

        Instruction::SUB(target) => {
            let value = read_u8_target(cpu, target);
            let new_value = cpu.sub(value);
            cpu.regs.set_a(new_value);
        }

        Instruction::SBC(target) => {
            let carry = if cpu.regs.get_carry() { 1 } else { 0 };
            let value = read_u8_target(cpu, target);
            let a = cpu.regs.a();

            let result = a.wrapping_sub(value).wrapping_sub(carry);
            cpu.regs.set_a(result);

            cpu.regs.set_z(result == 0);
            cpu.regs.set_n(true);
            cpu.regs.set_hc((a & 0x0F) < ((value & 0x0F) + carry));
            cpu.regs.set_carry((a as u16) < (value as u16) + (carry as u16));
        }

        Instruction::INC(target) => {
            match target {
                ArithmeticTarget::B => {
                    let v = cpu.regs.b();
                    let nv = v.wrapping_add(1);
                    cpu.regs.set_b(nv);
                    cpu.regs.set_z(nv == 0);
                    cpu.regs.set_n(false);
                    cpu.regs.set_hc((v & 0x0F) + 1 > 0x0F);
                }
                ArithmeticTarget::C => {
                    let v = cpu.regs.c();
                    let nv = v.wrapping_add(1);
                    cpu.regs.set_c(nv);
                    cpu.regs.set_z(nv == 0);
                    cpu.regs.set_n(false);
                    cpu.regs.set_hc((v & 0x0F) + 1 > 0x0F);
                }
                ArithmeticTarget::D => {
                    let v = cpu.regs.d();
                    let nv = v.wrapping_add(1);
                    cpu.regs.set_d(nv);
                    cpu.regs.set_z(nv == 0);
                    cpu.regs.set_n(false);
                    cpu.regs.set_hc((v & 0x0F) + 1 > 0x0F);
                }
                ArithmeticTarget::E => {
                    let v = cpu.regs.e();
                    let nv = v.wrapping_add(1);
                    cpu.regs.set_e(nv);
                    cpu.regs.set_z(nv == 0);
                    cpu.regs.set_n(false);
                    cpu.regs.set_hc((v & 0x0F) + 1 > 0x0F);
                }
                ArithmeticTarget::H => {
                    let v = cpu.regs.h();
                    let nv = v.wrapping_add(1);
                    cpu.regs.set_h(nv);
                    cpu.regs.set_z(nv == 0);
                    cpu.regs.set_n(false);
                    cpu.regs.set_hc((v & 0x0F) + 1 > 0x0F);
                }
                ArithmeticTarget::L => {
                    let v = cpu.regs.l();
                    let nv = v.wrapping_add(1);
                    cpu.regs.set_l(nv);
                    cpu.regs.set_z(nv == 0);
                    cpu.regs.set_n(false);
                    cpu.regs.set_hc((v & 0x0F) + 1 > 0x0F);
                }
                ArithmeticTarget::HLI => {
                    let addr = cpu.regs.get_hl();
                    let v = cpu.bus.read_byte(addr);
                    let nv = v.wrapping_add(1);
                    cpu.bus.write_byte(addr, nv);
                    cpu.regs.set_z(nv == 0);
                    cpu.regs.set_n(false);
                    cpu.regs.set_hc((v & 0x0F) + 1 > 0x0F);
                }
                _ => {}
            }
        }

        Instruction::DEC(target) => {
            match target {
                ArithmeticTarget::B => {
                    let v = cpu.regs.b();
                    let nv = v.wrapping_sub(1);
                    cpu.regs.set_b(nv);
                    cpu.regs.set_z(nv == 0);
                    cpu.regs.set_n(true);
                    cpu.regs.set_hc((v & 0x0F) == 0);
                }
                ArithmeticTarget::C => {
                    let v = cpu.regs.c();
                    let nv = v.wrapping_sub(1);
                    cpu.regs.set_c(nv);
                    cpu.regs.set_z(nv == 0);
                    cpu.regs.set_n(true);
                    cpu.regs.set_hc((v & 0x0F) == 0);
                }
                ArithmeticTarget::D => {
                    let v = cpu.regs.d();
                    let nv = v.wrapping_sub(1);
                    cpu.regs.set_d(nv);
                    cpu.regs.set_z(nv == 0);
                    cpu.regs.set_n(true);
                    cpu.regs.set_hc((v & 0x0F) == 0);
                }
                ArithmeticTarget::E => {
                    let v = cpu.regs.e();
                    let nv = v.wrapping_sub(1);
                    cpu.regs.set_e(nv);
                    cpu.regs.set_z(nv == 0);
                    cpu.regs.set_n(true);
                    cpu.regs.set_hc((v & 0x0F) == 0);
                }
                ArithmeticTarget::H => {
                    let v = cpu.regs.h();
                    let nv = v.wrapping_sub(1);
                    cpu.regs.set_h(nv);
                    cpu.regs.set_z(nv == 0);
                    cpu.regs.set_n(true);
                    cpu.regs.set_hc((v & 0x0F) == 0);
                }
                ArithmeticTarget::L => {
                    let v = cpu.regs.l();
                    let nv = v.wrapping_sub(1);
                    cpu.regs.set_l(nv);
                    cpu.regs.set_z(nv == 0);
                    cpu.regs.set_n(true);
                    cpu.regs.set_hc((v & 0x0F) == 0);
                }
                ArithmeticTarget::HLI => {
                    let addr = cpu.regs.get_hl();
                    let v = cpu.bus.read_byte(addr);
                    let nv = v.wrapping_sub(1);
                    cpu.bus.write_byte(addr, nv);
                    cpu.regs.set_z(nv == 0);
                    cpu.regs.set_n(true);
                    cpu.regs.set_hc((v & 0x0F) == 0);
                }
                _ => {}
            }
        }

        Instruction::INC16(target) => {
            match target {
                Add16Target::BC => cpu.regs.set_bc(cpu.regs.get_bc().wrapping_add(1)),
                Add16Target::DE => cpu.regs.set_de(cpu.regs.get_de().wrapping_add(1)),
                Add16Target::HL => cpu.regs.set_hl(cpu.regs.get_hl().wrapping_add(1)),
                Add16Target::SP => cpu.sp = cpu.sp.wrapping_add(1),
            }
        }

        Instruction::DEC16(target) => {
            match target {
                Add16Target::BC => cpu.regs.set_bc(cpu.regs.get_bc().wrapping_sub(1)),
                Add16Target::DE => cpu.regs.set_de(cpu.regs.get_de().wrapping_sub(1)),
                Add16Target::HL => cpu.regs.set_hl(cpu.regs.get_hl().wrapping_sub(1)),
                Add16Target::SP => cpu.sp = cpu.sp.wrapping_sub(1),
            }
        }

        Instruction::LD(load_type) => {
            match load_type {
                LoadType::R8ToR8(target, source) => {
                    let v = match source {
                        LoadByteSource::A => cpu.regs.a(),
                        LoadByteSource::B => cpu.regs.b(),
                        LoadByteSource::C => cpu.regs.c(),
                        LoadByteSource::D => cpu.regs.d(),
                        LoadByteSource::E => cpu.regs.e(),
                        LoadByteSource::H => cpu.regs.h(),
                        LoadByteSource::L => cpu.regs.l(),
                        LoadByteSource::HLI => cpu.bus.read_byte(cpu.regs.get_hl()),
                        LoadByteSource::D8 => cpu.next_byte(),
                    };

                    match target {
                        LoadByteTarget::A => cpu.regs.set_a(v),
                        LoadByteTarget::B => cpu.regs.set_b(v),
                        LoadByteTarget::C => cpu.regs.set_c(v),
                        LoadByteTarget::D => cpu.regs.set_d(v),
                        LoadByteTarget::E => cpu.regs.set_e(v),
                        LoadByteTarget::H => cpu.regs.set_h(v),
                        LoadByteTarget::L => cpu.regs.set_l(v),
                        LoadByteTarget::HLI => cpu.bus.write_byte(cpu.regs.get_hl(), v),
                    };
                }

                LoadType::D16toR16(target) => {
                    let v = cpu.next_word();
                    match target {
                        BigLoadByteTarget::AF => cpu.regs.set_af(v),
                        BigLoadByteTarget::BC => cpu.regs.set_bc(v),
                        BigLoadByteTarget::DE => cpu.regs.set_de(v),
                        BigLoadByteTarget::HL => cpu.regs.set_hl(v),
                        BigLoadByteTarget::SP => cpu.sp = v,
                    };
                }

                LoadType::HLtoSP => {
                    cpu.sp = cpu.regs.get_hl();
                }

                LoadType::SPtoA16 => {
                    let addr = cpu.next_word();
                    let sp = cpu.sp;
                    cpu.bus.write_byte(addr, (sp & 0xFF) as u8);
                    cpu.bus.write_byte(addr.wrapping_add(1), (sp >> 8) as u8);
                }

                LoadType::R16toSP(source) => {
                    let v = match source {
                        BigRegisterTarget::AF => cpu.regs.get_af(),
                        BigRegisterTarget::BC => cpu.regs.get_bc(),
                        BigRegisterTarget::DE => cpu.regs.get_de(),
                        BigRegisterTarget::HL => cpu.regs.get_hl(),
                    };
                    cpu.sp = v;
                }

                LoadType::SP8toHL => {
                    let off = cpu.next_byte() as i8 as i16;
                    let sp = cpu.sp;
                    let result = sp.wrapping_add(off as u16);
                    cpu.regs.set_hl(result);

                    let sp_low = sp & 0xFF;
                    let off_u = (off as u16) & 0xFF;

                    cpu.regs.set_z(false);
                    cpu.regs.set_n(false);
                    cpu.regs.set_carry(((sp_low & 0xFF) + (off_u & 0xFF)) > 0xFF);
                    cpu.regs.set_hc(((sp_low & 0x0F) + (off_u & 0x0F)) > 0x0F);
                }

                LoadType::HLIfromA => {
                    let addr = cpu.regs.get_hl();
                    let a = cpu.regs.a();
                    cpu.bus.write_byte(addr, a);
                    cpu.regs.set_hl(addr.wrapping_add(1));
                }

                LoadType::AfromHLI => {
                    let addr = cpu.regs.get_hl();
                    let v = cpu.bus.read_byte(addr);
                    cpu.regs.set_a(v);
                    cpu.regs.set_hl(addr.wrapping_add(1));
                }

                LoadType::HLDfromA => {
                    let addr = cpu.regs.get_hl();
                    let a = cpu.regs.a();
                    cpu.bus.write_byte(addr, a);
                    cpu.regs.set_hl(addr.wrapping_sub(1));
                }

                LoadType::AfromHLD => {
                    let addr = cpu.regs.get_hl();
                    let v = cpu.bus.read_byte(addr);
                    cpu.regs.set_a(v);
                    cpu.regs.set_hl(addr.wrapping_sub(1));
                }

                LoadType::A16fromA => {
                    let addr = cpu.next_word();
                    let a = cpu.regs.a();
                    cpu.bus.write_byte(addr, a);
                }

                LoadType::AfromA16 => {
                    let addr = cpu.next_word();
                    let v = cpu.bus.read_byte(addr);
                    cpu.regs.set_a(v);
                }

                LoadType::FF00A8fromA => {
                    let a8 = cpu.next_byte() as u16;
                    let addr = 0xFF00u16.wrapping_add(a8);
                    let a = cpu.regs.a();
                    cpu.bus.write_byte(addr, a);
                }

                LoadType::AfromFF00A8 => {
                    let a8 = cpu.next_byte() as u16;
                    let addr = 0xFF00u16.wrapping_add(a8);
                    let v = cpu.bus.read_byte(addr);
                    cpu.regs.set_a(v);
                }

                LoadType::FF00CfromA => {
                    let c = cpu.regs.c() as u16;
                    let addr = 0xFF00u16.wrapping_add(c);
                    let a = cpu.regs.a();
                    cpu.bus.write_byte(addr, a);
                }

                LoadType::AfromFF00C => {
                    let c = cpu.regs.c() as u16;
                    let addr = 0xFF00u16.wrapping_add(c);
                    let v = cpu.bus.read_byte(addr);
                    cpu.regs.set_a(v);
                }

                _ => {}
            }
        }

        Instruction::XOR(target) => {
            let v = read_u8_target(cpu, target);
            let r = cpu.regs.a() ^ v;
            cpu.regs.set_a(r);
            cpu.regs.set_z(r == 0);
            cpu.regs.set_n(false);
            cpu.regs.set_hc(false);
            cpu.regs.set_carry(false);
        }

        Instruction::CP(target) => {
            let v = read_u8_target(cpu, target);
            let a = cpu.regs.a();
            let r = a.wrapping_sub(v);

            cpu.regs.set_z(r == 0);
            cpu.regs.set_n(true);
            cpu.regs.set_hc((a & 0x0F) < (v & 0x0F));
            cpu.regs.set_carry(a < v);
        }

        Instruction::AND(target) => {
            let v = read_u8_target(cpu, target);
            let r = cpu.regs.a() & v;
            cpu.regs.set_a(r);
            cpu.regs.set_z(r == 0);
            cpu.regs.set_n(false);
            cpu.regs.set_hc(true);
            cpu.regs.set_carry(false);
        }

        Instruction::OR(target) => {
            let v = read_u8_target(cpu, target);
            let r = cpu.regs.a() | v;
            cpu.regs.set_a(r);
            cpu.regs.set_z(r == 0);
            cpu.regs.set_n(false);
            cpu.regs.set_hc(false);
            cpu.regs.set_carry(false);
        }

        Instruction::CPL => {
            cpu.regs.set_a(!cpu.regs.a());
            cpu.regs.set_n(true);
            cpu.regs.set_hc(true);
        }

        Instruction::SCF => {
            cpu.regs.set_n(false);
            cpu.regs.set_hc(false);
            cpu.regs.set_carry(true);
        }

        Instruction::CCF => {
            let c = cpu.regs.get_carry();
            cpu.regs.set_n(false);
            cpu.regs.set_hc(false);
            cpu.regs.set_carry(!c);
        }

        Instruction::DAA => {
            daa(cpu);
        },

        Instruction::RLCA => {
            let a = cpu.regs.a();
            let carry = (a >> 7) & 1;
            let r = a.rotate_left(1);
            cpu.regs.set_a(r);

            cpu.regs.set_z(false);
            cpu.regs.set_n(false);
            cpu.regs.set_hc(false);
            cpu.regs.set_carry(carry != 0);
}

        Instruction::RRCA => {
            let a = cpu.regs.a();
            let carry = a & 1;
            let r = a.rotate_right(1);
            cpu.regs.set_a(r);

            cpu.regs.set_z(false);
            cpu.regs.set_n(false);
            cpu.regs.set_hc(false);
            cpu.regs.set_carry(carry != 0);
        },

        Instruction::RLA => {
            let a = cpu.regs.a();
            let old_carry = if cpu.regs.get_carry() { 1 } else { 0 };
            let carry = (a >> 7) & 1;
            let r = (a << 1) | old_carry;
            cpu.regs.set_a(r);

            cpu.regs.set_z(false);
            cpu.regs.set_n(false);
            cpu.regs.set_hc(false);
            cpu.regs.set_carry(carry != 0);
        },

        Instruction::RRA => {
            let a = cpu.regs.a();
            let old_carry = if cpu.regs.get_carry() { 0x80 } else { 0 };
            let carry = a & 1;
            let r = (a >> 1) | old_carry;
            cpu.regs.set_a(r);

            cpu.regs.set_z(false);
            cpu.regs.set_n(false);
            cpu.regs.set_hc(false);
            cpu.regs.set_carry(carry != 0);
        },

        Instruction::EI => {
            cpu.ime_scheduled = true;
        }

        Instruction::DI => {
            cpu.ime = false;
            cpu.ime_scheduled = false;
        }



        _ => {}
    }
}
