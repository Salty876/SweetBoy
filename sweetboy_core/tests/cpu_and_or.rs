use sweetboy_core::Cpu;

#[test]
fn and_a_sets_zero_and_h() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x00);

    cpu.bus.write_byte(0x0100, 0xA7); // AND A
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x00);

    assert!(cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(cpu.regs.get_hc()); // MUST be set
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}


#[test]
fn and_b_sets_h_and_clears_c() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x3C);
    cpu.regs.set_b(0x0F);

    // junk flags first
    cpu.regs.set_z(false);
    cpu.regs.set_n(true);
    cpu.regs.set_hc(false);
    cpu.regs.set_carry(true);

    cpu.bus.write_byte(0x0100, 0xA0); // AND B
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x0C);

    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}


#[test]
fn and_hl_reads_memory_and_sets_zero() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0xF0);
    cpu.regs.set_hl(0xC000);
    cpu.bus.write_byte(0xC000, 0x0F);

    cpu.bus.write_byte(0x0100, 0xA6); // AND (HL)
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x00);
    assert!(cpu.regs.get_z());
    assert!(cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}


#[test]
fn and_d8_advances_pc_by_2() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0xFF);

    cpu.bus.write_byte(0x0100, 0xE6); // AND d8
    cpu.bus.write_byte(0x0101, 0x0F);
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x0F);
    assert!(!cpu.regs.get_z());
    assert!(cpu.regs.get_hc());

    assert_eq!(cpu.pc, 0x0102);
}


#[test]
fn or_a_preserves_a_and_sets_flags() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x00);

    cpu.bus.write_byte(0x0100, 0xB7); // OR A
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x00);

    assert!(cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}


#[test]
fn or_b_sets_a_and_clears_all_flags() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x10);
    cpu.regs.set_b(0x01);

    cpu.regs.set_z(true);
    cpu.regs.set_n(true);
    cpu.regs.set_hc(true);
    cpu.regs.set_carry(true);

    cpu.bus.write_byte(0x0100, 0xB0); // OR B
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x11);

    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}


#[test]
fn or_d8_sets_zero_and_advances_pc_by_2() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x00);

    cpu.bus.write_byte(0x0100, 0xF6); // OR d8
    cpu.bus.write_byte(0x0101, 0x00);
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x00);
    assert!(cpu.regs.get_z());

    assert_eq!(cpu.pc, 0x0102);
}
