use sweetboy_core::Cpu;

//
// ADC tests
//

#[test]
fn adc_b_no_carry_in_no_halfcarry_no_carry() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x10);
    cpu.regs.set_b(0x20);

    // ensure carry-in = 0
    cpu.regs.set_carry(false);

    cpu.bus.write_byte(0x0100, 0x88); // ADC A,B
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x30);

    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn adc_b_uses_carry_in_sets_halfcarry() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x0F);
    cpu.regs.set_b(0x00);

    // carry-in = 1 => 0x0F + 0 + 1 = 0x10 (half-carry)
    cpu.regs.set_carry(true);

    // junk other flags to ensure ADC overwrites them
    cpu.regs.set_z(true);
    cpu.regs.set_n(true);
    cpu.regs.set_hc(false);

    cpu.bus.write_byte(0x0100, 0x88); // ADC A,B
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x10);

    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn adc_b_sets_carry_and_zero_on_overflow_with_carry_in() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0xFF);
    cpu.regs.set_b(0x00);
    cpu.regs.set_carry(true); // carry-in

    // 0xFF + 0x00 + 1 = 0x00, carry out = 1
    cpu.bus.write_byte(0x0100, 0x88); // ADC A,B
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x00);

    assert!(cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(cpu.regs.get_hc());      // 0xF + 0 + 1 => half-carry
    assert!(cpu.regs.get_carry());   // overflow carry

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn adc_hl_reads_memory_and_sets_flags() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x8F);
    cpu.regs.set_hl(0xC000);
    cpu.bus.write_byte(0xC000, 0x01);
    cpu.regs.set_carry(true); // carry-in

    // 0x8F + 0x01 + 1 = 0x91, half-carry? 0xF + 1 + 1 = 0x11 => yes
    cpu.bus.write_byte(0x0100, 0x8E); // ADC A,(HL)
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x91);

    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn adc_d8_advances_pc_by_2_and_sets_carry() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0xF0);
    cpu.regs.set_carry(true);

    cpu.bus.write_byte(0x0100, 0xCE); // ADC A,d8
    cpu.bus.write_byte(0x0101, 0x10); // 0xF0 + 0x10 + 1 = 0x01, carry out = 1
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x01);

    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());     // 0x0 + 0x0 + 1 => 1 (no half-carry)
    assert!(cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0102);
}

//
// SBC tests
//

#[test]
fn sbc_b_no_borrow_no_halfborrow() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x10);
    cpu.regs.set_b(0x01);
    cpu.regs.set_carry(false); // borrow-in = 0

    cpu.bus.write_byte(0x0100, 0x98); // SBC A,B
    cpu.step();

    // 0x10 - 0x01 = 0x0F
    assert_eq!(cpu.regs.a(), 0x0F);

    assert!(!cpu.regs.get_z());
    assert!(cpu.regs.get_n());
    assert!(cpu.regs.get_hc());      // half-borrow: 0x0 < 0x1
    assert!(!cpu.regs.get_carry());  // no full borrow

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn sbc_b_with_borrow_wraps_and_sets_borrow_flags() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x00);
    cpu.regs.set_b(0x00);
    cpu.regs.set_carry(true); // borrow-in = 1

    cpu.bus.write_byte(0x0100, 0x98); // SBC A,B
    cpu.step();

    // 0x00 - 0x00 - 1 = 0xFF, borrow out = 1
    assert_eq!(cpu.regs.a(), 0xFF);

    assert!(!cpu.regs.get_z());
    assert!(cpu.regs.get_n());
    assert!(cpu.regs.get_hc());     // 0x0 < (0x0 + 1)
    assert!(cpu.regs.get_carry());  // full borrow

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn sbc_b_zero_result_sets_z() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x01);
    cpu.regs.set_b(0x00);
    cpu.regs.set_carry(true); // borrow-in

    // 0x01 - 0x00 - 1 = 0x00
    cpu.bus.write_byte(0x0100, 0x98); // SBC A,B
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x00);

    assert!(cpu.regs.get_z());
    assert!(cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());     // 0x1 < (0x0 + 1)? false (equal)
    assert!(!cpu.regs.get_carry());  // no full borrow (1 >= 1)

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn sbc_hl_reads_memory_and_sets_full_borrow() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x10);
    cpu.regs.set_hl(0xC000);
    cpu.bus.write_byte(0xC000, 0x20);
    cpu.regs.set_carry(false);

    cpu.bus.write_byte(0x0100, 0x9E); // SBC A,(HL)
    cpu.step();

    // 0x10 - 0x20 = 0xF0 with borrow
    assert_eq!(cpu.regs.a(), 0xF0);

    assert!(!cpu.regs.get_z());
    assert!(cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());     // 0x0 < 0x0 ? false
    assert!(cpu.regs.get_carry());   // full borrow

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn sbc_d8_advances_pc_by_2_and_sets_halfborrow() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x10);
    cpu.regs.set_carry(true); // borrow-in

    cpu.bus.write_byte(0x0100, 0xDE); // SBC A,d8
    cpu.bus.write_byte(0x0101, 0x01); // 0x10 - 0x01 - 1 = 0x0E
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x0E);

    assert!(!cpu.regs.get_z());
    assert!(cpu.regs.get_n());
    assert!(cpu.regs.get_hc());     // 0x0 < (0x1 + 1) => true
    assert!(!cpu.regs.get_carry()); // no full borrow (0x10 >= 0x02)

    assert_eq!(cpu.pc, 0x0102);
}
