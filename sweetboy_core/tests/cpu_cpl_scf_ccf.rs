use sweetboy_core::Cpu;


#[test]
fn cpl_inverts_a_and_sets_n_h_preserves_z_c() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x0F);
    cpu.regs.set_z(true);
    cpu.regs.set_carry(true);

    cpu.bus.write_byte(0x0100, 0x2F);
    cpu.step();

    assert_eq!(cpu.regs.a(), 0xF0);
    assert!(cpu.regs.get_z());        // preserved
    assert!(cpu.regs.get_n());        // set
    assert!(cpu.regs.get_hc());       // set
    assert!(cpu.regs.get_carry());    // preserved
    assert_eq!(cpu.pc, 0x0101);
}


#[test]
fn scf_sets_clears_n_h_preserves_z() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_z(true);
    cpu.regs.set_n(true);
    cpu.regs.set_hc(true);
    cpu.regs.set_carry(false);

    cpu.bus.write_byte(0x0100, 0x37);
    cpu.step();

    assert!(cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(cpu.regs.get_carry());
    assert_eq!(cpu.pc, 0x0101);
}


#[test]
fn ccf_toggles_clears_n_h_preserves_z() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_z(true);
    cpu.regs.set_carry(true);
    cpu.regs.set_n(true);
    cpu.regs.set_hc(true);

    cpu.bus.write_byte(0x0100, 0x3F);
    cpu.step();

    assert!(cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(!cpu.regs.get_carry());
    assert_eq!(cpu.pc, 0x0101);
}
