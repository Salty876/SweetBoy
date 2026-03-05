use sweetboy_core::Cpu;

#[test]
fn ldh_a8_a_writes_to_ff00_plus_a8() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x55);

    cpu.bus.write_byte(0x0100, 0xE0);
    cpu.bus.write_byte(0x0101, 0x80);

    cpu.step();

    assert_eq!(cpu.bus.read_byte(0xFF80), 0x55);
    assert_eq!(cpu.pc, 0x0102);
}

#[test]
fn ldh_a_a8_reads_from_ff00_plus_a8() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.bus.write_byte(0xFF42, 0x9A);

    cpu.bus.write_byte(0x0100, 0xF0);
    cpu.bus.write_byte(0x0101, 0x42);

    cpu.step();

    assert_eq!(cpu.regs.a(), 0x9A);
    assert_eq!(cpu.pc, 0x0102);
}

#[test]
fn ld_c_a_writes_to_ff00_plus_c() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_c(0x10);
    cpu.regs.set_a(0xDE);

    cpu.bus.write_byte(0x0100, 0xE2);

    cpu.step();

    assert_eq!(cpu.bus.read_byte(0xFF10), 0xDE);
    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn ld_a_c_reads_from_ff00_plus_c() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_c(0x02);
    cpu.bus.write_byte(0xFF02, 0x77);

    cpu.bus.write_byte(0x0100, 0xF2);

    cpu.step();

    assert_eq!(cpu.regs.a(), 0x77);
    assert_eq!(cpu.pc, 0x0101);
}
