use sweetboy_core::Cpu;

#[test]
fn rlca_rotates_and_sets_carry_z_cleared() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x81); // 1000_0001 -> RLCA => 0000_0011, C=1
    cpu.regs.set_z(true); // ensure cleared

    cpu.bus.write_byte(0x0100, 0x07);
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x03);
    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(cpu.regs.get_carry());
    assert_eq!(cpu.pc, 0x0101);
}


#[test]
fn rrca_rotates_and_sets_carry_z_cleared() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x01); // 0000_0001 -> 1000_0000, C=1
    cpu.regs.set_z(true);

    cpu.bus.write_byte(0x0100, 0x0F);
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x80);
    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(cpu.regs.get_carry());
    assert_eq!(cpu.pc, 0x0101);
}


#[test]
fn rla_rotates_through_carry() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x80);
    cpu.regs.set_carry(true); // old_c=1 => result low bit becomes 1

    cpu.bus.write_byte(0x0100, 0x17);
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x01);
    assert!(!cpu.regs.get_z());
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(cpu.regs.get_carry()); // old bit7
    assert_eq!(cpu.pc, 0x0101);
}


#[test]
fn rra_rotates_through_carry() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x01);
    cpu.regs.set_carry(false); // old_c=0 => msb becomes 0

    cpu.bus.write_byte(0x0100, 0x1F);
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x00);
    assert!(cpu.regs.get_z() == false); // Z forced 0 even though result is 0
    assert!(!cpu.regs.get_n());
    assert!(!cpu.regs.get_hc());
    assert!(cpu.regs.get_carry()); // old bit0
    assert_eq!(cpu.pc, 0x0101);
}
