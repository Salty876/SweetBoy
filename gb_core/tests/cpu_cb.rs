use gb_core::Cpu;

#[test]
fn cb_rlc_b_rotates_left_sets_carry_and_z() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_b(0x80);

    cpu.bus.write_byte(0x0100, 0xCB);
    cpu.bus.write_byte(0x0101, 0x00); // RLC B

    cpu.step();

    assert_eq!(cpu.regs.b(), 0x01);

    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0102);
}



#[test]
fn cb_rl_hl_rotates_through_carry_and_writes_back() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_hl(0xC000);
    cpu.bus.write_byte(0xC000, 0x00);

    cpu.regs.set_carry(true);

    cpu.bus.write_byte(0x0100, 0xCB);
    cpu.bus.write_byte(0x0101, 0x16); // RL (HL)

    cpu.step();

    assert_eq!(cpu.bus.read_byte(0xC000), 0x01);

    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0102);
}



#[test]
fn cb_bit_7_h_sets_z_h_and_preserves_carry() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_h(0x7F);
    cpu.regs.set_carry(true);

    cpu.bus.write_byte(0x0100, 0xCB);
    cpu.bus.write_byte(0x0101, 0x7C); // BIT 7,H

    cpu.step();

    assert!(cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(cpu.regs.get_hc());
    assert!(cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0102);
}



#[test]
fn cb_res_0_hl_clears_bit_and_does_not_touch_flags() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_hl(0xC000);
    cpu.bus.write_byte(0xC000, 0xFF);

    cpu.regs.set_z(true);
    cpu.regs.set_n(true);
    cpu.regs.set_hc(true);
    cpu.regs.set_carry(true);

    cpu.bus.write_byte(0x0100, 0xCB);
    cpu.bus.write_byte(0x0101, 0x86); // RES 0,(HL)

    cpu.step();

    assert_eq!(cpu.bus.read_byte(0xC000), 0xFE);

    assert!(cpu.regs.get_z());
    assert!(cpu.regs.get_n());
    assert!(cpu.regs.get_hc());
    assert!(cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0102);
}



#[test]
fn cb_swap_c_swaps_nibbles_sets_z_and_clears_carry() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_c(0xF0);
    cpu.regs.set_carry(true);

    cpu.bus.write_byte(0x0100, 0xCB);
    cpu.bus.write_byte(0x0101, 0x31); // SWAP C

    cpu.step();

    assert_eq!(cpu.regs.c(), 0x0F);

    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0102);
}
