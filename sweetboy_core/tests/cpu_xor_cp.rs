use sweetboy_core::Cpu;

#[test]
fn xor_a_sets_a_to_zero_and_sets_flags() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x5A);
    cpu.bus.write_byte(0x0100, 0xAF); // XOR A

    cpu.step();

    assert_eq!(cpu.regs.a(), 0x00);

    assert!(cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn xor_b_updates_a_and_clears_flags() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0xF0);
    cpu.regs.set_b(0x0F);

    // set flags to junk first to ensure XOR clears them
    cpu.regs.set_z(false);
    cpu.regs.set_n(true);
    cpu.regs.set_hc(true);
    cpu.regs.set_carry(true);

    cpu.bus.write_byte(0x0100, 0xA8); // XOR B
    cpu.step();

    assert_eq!(cpu.regs.a(), 0xFF);

    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn xor_hl_reads_memory_and_sets_zero() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0xAB);
    cpu.regs.set_hl(0xC000);
    cpu.bus.write_byte(0xC000, 0xAB);

    cpu.bus.write_byte(0x0100, 0xAE); // XOR (HL)
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x00);
    assert!(cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn xor_d8_uses_immediate_and_advances_pc_by_2() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x3C);

    cpu.bus.write_byte(0x0100, 0xEE); // XOR d8
    cpu.bus.write_byte(0x0101, 0x3C);

    cpu.step();

    assert_eq!(cpu.regs.a(), 0x00);
    assert!(cpu.regs.get_z());
    assert_eq!(cpu.pc, 0x0102);
}

#[test]
fn cp_b_sets_flags_like_sub_without_modifying_a() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x10);
    cpu.regs.set_b(0x01);

    cpu.bus.write_byte(0x0100, 0xB8); // CP B
    cpu.step();

    // A unchanged
    assert_eq!(cpu.regs.a(), 0x10);

    // 0x10 - 0x01 = 0x0F
    assert!(!cpu.regs.get_z());
    assert!(cpu.regs.get_n());

    // half-borrow: low nibble 0x0 < 0x1 => true
    assert!(cpu.regs.get_hc());

    // carry/borrow: A < value? 0x10 < 0x01 => false
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn cp_a_sets_zero_and_n_and_clears_borrows() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x42);

    cpu.bus.write_byte(0x0100, 0xBF); // CP A
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x42);

    assert!(cpu.regs.get_z());
    assert!(cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn cp_hl_sets_carry_when_a_is_less_than_mem() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x10);
    cpu.regs.set_hl(0xC000);
    cpu.bus.write_byte(0xC000, 0x20);

    cpu.bus.write_byte(0x0100, 0xBE); // CP (HL)
    cpu.step();

    // A unchanged
    assert_eq!(cpu.regs.a(), 0x10);

    // Borrow should occur: 0x10 < 0x20
    assert!(!cpu.regs.get_z());
    assert!(cpu.regs.get_n());
    assert!(cpu.regs.get_carry());

    // half-borrow: 0x0 < 0x0 => false
    assert!(!cpu.regs.get_hc());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn cp_d8_immediate_advances_pc_by_2_and_sets_zero_when_equal() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x77);

    cpu.bus.write_byte(0x0100, 0xFE); // CP d8
    cpu.bus.write_byte(0x0101, 0x77);

    cpu.step();

    assert_eq!(cpu.regs.a(), 0x77);

    assert!(cpu.regs.get_z());
    assert!(cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0102);
}
