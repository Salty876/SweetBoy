use sweetboy_core::Cpu;

#[test]
fn jr_e8_forward_taken() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    // 0100: 18 02   JR +2  => target = 0x0102 + 2 = 0x0104
    cpu.bus.write_byte(0x0100, 0x18);
    cpu.bus.write_byte(0x0101, 0x02);

    cpu.step();

    assert_eq!(cpu.pc, 0x0104);
}

#[test]
fn jr_e8_backward_taken() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    // 0100: 18 FE   JR -2  (0xFE as i8 = -2) => target = 0x0102 - 2 = 0x0100
    cpu.bus.write_byte(0x0100, 0x18);
    cpu.bus.write_byte(0x0101, 0xFE);

    cpu.step();

    assert_eq!(cpu.pc, 0x0100);
}

#[test]
fn jr_nz_taken_when_z_is_false() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_z(false);

    // 0100: 20 02   JR NZ, +2 => 0x0104
    cpu.bus.write_byte(0x0100, 0x20);
    cpu.bus.write_byte(0x0101, 0x02);

    cpu.step();

    assert_eq!(cpu.pc, 0x0104);
}

#[test]
fn jr_nz_not_taken_when_z_is_true() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_z(true);

    // 0100: 20 02   JR NZ, +2 (not taken) => PC += 2
    cpu.bus.write_byte(0x0100, 0x20);
    cpu.bus.write_byte(0x0101, 0x02);

    cpu.step();

    assert_eq!(cpu.pc, 0x0102);
}

#[test]
fn jr_z_taken_when_z_is_true() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_z(true);

    // 0100: 28 FE   JR Z, -2 => target = 0x0102 - 2 = 0x0100
    cpu.bus.write_byte(0x0100, 0x28);
    cpu.bus.write_byte(0x0101, 0xFE);

    cpu.step();

    assert_eq!(cpu.pc, 0x0100);
}

#[test]
fn jr_nc_taken_when_c_is_false() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_carry(false);

    // 0100: 30 01   JR NC, +1 => target = 0x0102 + 1 = 0x0103
    cpu.bus.write_byte(0x0100, 0x30);
    cpu.bus.write_byte(0x0101, 0x01);

    cpu.step();

    assert_eq!(cpu.pc, 0x0103);
}

#[test]
fn jr_c_not_taken_when_c_is_false() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_carry(false);

    // 0100: 38 7F   JR C, +127 (not taken) => PC += 2
    cpu.bus.write_byte(0x0100, 0x38);
    cpu.bus.write_byte(0x0101, 0x7F);

    cpu.step();

    assert_eq!(cpu.pc, 0x0102);
}
