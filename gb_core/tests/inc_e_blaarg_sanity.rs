use gb_core::Cpu;

#[test]
fn inc_e_sets_z_when_wrapping_to_zero() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_e(0xFF);
    cpu.regs.set_z(false);
    cpu.regs.set_n(true);
    cpu.regs.set_hc(false);
    cpu.regs.set_carry(true); // should remain unchanged

    cpu.bus.write_byte(0x0100, 0x1C); // INC E
    cpu.step();

    assert_eq!(cpu.regs.e(), 0x00);
    assert!(cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(cpu.regs.get_hc());
    assert!(cpu.regs.get_carry()); // unchanged
    assert_eq!(cpu.pc, 0x0101);
}
