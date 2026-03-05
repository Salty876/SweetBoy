use sweetboy_core::Cpu;

#[test]
fn daa_after_add_low_nibble_adjusts_to_next_ten() {
    // 0x09 + 0x01 = 0x0A (flags: N=0, H=0, C=0) => DAA => 0x10
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x0A);
    cpu.regs.set_n(false);
    cpu.regs.set_hc(false);
    cpu.regs.set_carry(false);

    cpu.bus.write_byte(0x0100, 0x27); // DAA
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x10);

    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());      // unchanged
    assert!(!cpu.regs.get_hc());     // cleared
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn daa_after_add_uses_halfcarry_flag_to_add_06() {
    // If H=1 after addition, DAA must add 0x06.
    // Example: A=0x13 with H=1 => DAA => 0x19
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x13);
    cpu.regs.set_n(false);
    cpu.regs.set_hc(true);
    cpu.regs.set_carry(false);

    cpu.bus.write_byte(0x0100, 0x27); // DAA
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x19);

    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn daa_after_add_a_gt_99_sets_carry_and_adds_60() {
    // A > 0x99 should force +0x60 and set carry.
    // A=0xA0 => DAA => 0x00, C=1
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0xA0);
    cpu.regs.set_n(false);
    cpu.regs.set_hc(false);
    cpu.regs.set_carry(false);

    cpu.bus.write_byte(0x0100, 0x27); // DAA
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x00);

    assert!(cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn daa_after_add_adjusts_both_nibbles_9a_to_00_and_sets_carry() {
    // Classic: A=0x9A => +0x66 => 0x00, C=1
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x9A);
    cpu.regs.set_n(false);
    cpu.regs.set_hc(false);
    cpu.regs.set_carry(false);

    cpu.bus.write_byte(0x0100, 0x27); // DAA
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x00);

    assert!(cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn daa_after_add_preserves_existing_carry_and_adds_60() {
    // If carry already set from addition, DAA adds 0x60 and keeps C=1.
    // A=0x42, C=1 => DAA => 0xA2
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x42);
    cpu.regs.set_n(false);
    cpu.regs.set_hc(false);
    cpu.regs.set_carry(true);

    cpu.bus.write_byte(0x0100, 0x27); // DAA
    cpu.step();

    assert_eq!(cpu.regs.a(), 0xA2);

    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn daa_after_sub_uses_halfcarry_to_sub_06() {
    // After SUB/SBC (N=1), if H=1 then subtract 0x06.
    // Example: A=0x0D (from 0x15-0x08), N=1,H=1,C=0 => DAA => 0x07
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x0D);
    cpu.regs.set_n(true);
    cpu.regs.set_hc(true);
    cpu.regs.set_carry(false);

    cpu.bus.write_byte(0x0100, 0x27); // DAA
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x07);

    assert!(!cpu.regs.get_z());
    assert!(cpu.regs.get_n());       // unchanged
    assert!(!cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn daa_after_sub_with_carry_subs_60_and_preserves_carry() {
    // After SUB/SBC, if C=1 then subtract 0x60, and C stays 1.
    // Example: A=0x98, N=1,C=1,H=0 => DAA => 0x38, C=1
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x98);
    cpu.regs.set_n(true);
    cpu.regs.set_hc(false);
    cpu.regs.set_carry(true);

    cpu.bus.write_byte(0x0100, 0x27); // DAA
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x38);

    assert!(!cpu.regs.get_z());
    assert!(cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn daa_after_sub_with_carry_and_halfcarry_subs_66() {
    // Example: A=0xFF, N=1,H=1,C=1 => subtract 0x66 => 0x99, C stays 1
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0xFF);
    cpu.regs.set_n(true);
    cpu.regs.set_hc(true);
    cpu.regs.set_carry(true);

    cpu.bus.write_byte(0x0100, 0x27); // DAA
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x99);

    assert!(!cpu.regs.get_z());
    assert!(cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}
