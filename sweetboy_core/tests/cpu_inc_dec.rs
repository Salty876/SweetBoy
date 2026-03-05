use sweetboy_core::Cpu;

/* =======================
   INC r tests
   ======================= */

#[test]
fn inc_r_simple_increment() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_b(0x12);
    cpu.regs.set_carry(true); // C should be preserved

    cpu.bus.write_byte(0x0100, 0x04); // INC B
    cpu.step();

    assert_eq!(cpu.regs.b(), 0x13);
    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(cpu.regs.get_carry()); // unchanged
    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn inc_r_sets_zero_flag() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_b(0xFF);

    cpu.bus.write_byte(0x0100, 0x04); // INC B
    cpu.step();

    assert_eq!(cpu.regs.b(), 0x00);
    assert!(cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
}

#[test]
fn inc_r_sets_half_carry() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_b(0x0F);

    cpu.bus.write_byte(0x0100, 0x04); // INC B
    cpu.step();

    assert_eq!(cpu.regs.b(), 0x10);
    assert!(cpu.regs.get_hc());
    assert!(!cpu.regs.get_n());
}

/* =======================
   DEC r tests
   ======================= */

#[test]
fn dec_r_simple_decrement() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_c(0x22);
    cpu.regs.set_carry(true); // C must be preserved

    cpu.bus.write_byte(0x0100, 0x0D); // DEC C
    cpu.step();

    assert_eq!(cpu.regs.c(), 0x21);
    assert!(!cpu.regs.get_z());
    assert!(cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(cpu.regs.get_carry()); // unchanged
    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn dec_r_sets_zero_flag() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_c(0x01);

    cpu.bus.write_byte(0x0100, 0x0D); // DEC C
    cpu.step();

    assert_eq!(cpu.regs.c(), 0x00);
    assert!(cpu.regs.get_z());
    assert!(cpu.regs.get_n());
}

#[test]
fn dec_r_sets_half_borrow() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_c(0x10);

    cpu.bus.write_byte(0x0100, 0x0D); // DEC C
    cpu.step();

    assert_eq!(cpu.regs.c(), 0x0F);
    assert!(cpu.regs.get_hc()); // borrow from bit 4
    assert!(cpu.regs.get_n());
}

/* =======================
   INC / DEC (HL) tests
   ======================= */

#[test]
fn inc_hl_increments_memory() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_hl(0xC000);
    cpu.bus.write_byte(0xC000, 0x3E);

    cpu.bus.write_byte(0x0100, 0x34); // INC (HL)
    cpu.step();

    assert_eq!(cpu.bus.read_byte(0xC000), 0x3F);
    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn dec_hl_sets_zero_and_half_borrow() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_hl(0xC000);
    cpu.bus.write_byte(0xC000, 0x10);

    cpu.bus.write_byte(0x0100, 0x35); // DEC (HL)
    cpu.step();

    assert_eq!(cpu.bus.read_byte(0xC000), 0x0F);
    assert!(!cpu.regs.get_z());
    assert!(cpu.regs.get_hc());
    assert!(cpu.regs.get_n());
}
