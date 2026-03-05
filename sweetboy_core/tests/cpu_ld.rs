use sweetboy_core::Cpu;

#[test]
fn ld_a_d8_loads_immediate() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.bus.write_byte(0x0100, 0x3E); // LD A, d8
    cpu.bus.write_byte(0x0101, 0x42);

    cpu.step();

    assert_eq!(cpu.regs.a(), 0x42);
    assert_eq!(cpu.pc, 0x0102);
}

#[test]
fn ld_b_d8_loads_immediate() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.bus.write_byte(0x0100, 0x06); // LD B, d8
    cpu.bus.write_byte(0x0101, 0x99);

    cpu.step();

    assert_eq!(cpu.regs.b(), 0x99);
    assert_eq!(cpu.pc, 0x0102);
}

#[test]
fn ld_a_hl_reads_memory_byte() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_hl(0xC000);
    cpu.bus.write_byte(0xC000, 0xAB);
    cpu.bus.write_byte(0x0100, 0x7E); // LD A, (HL)

    cpu.step();

    assert_eq!(cpu.regs.a(), 0xAB);
    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn ld_hl_a_writes_memory_byte() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_hl(0xC000);
    cpu.regs.set_a(0x55);

    cpu.bus.write_byte(0x0100, 0x77); // LD (HL), A

    cpu.step();

    assert_eq!(cpu.bus.read_byte(0xC000), 0x55);
    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn ld_a_b_copies_register() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_b(0xDE);

    cpu.bus.write_byte(0x0100, 0x78); // LD A, B

    cpu.step();

    assert_eq!(cpu.regs.a(), 0xDE);
    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn ld_hl_b_writes_memory_from_register() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_hl(0xC123);
    cpu.regs.set_b(0x7F);

    cpu.bus.write_byte(0x0100, 0x70); // LD (HL), B

    cpu.step();

    assert_eq!(cpu.bus.read_byte(0xC123), 0x7F);
    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn ld_hli_a_writes_and_increments_hl() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_hl(0xC000);
    cpu.regs.set_a(0x42);

    cpu.bus.write_byte(0x0100, 0x22); // LD (HL+),A
    cpu.step();

    assert_eq!(cpu.bus.read_byte(0xC000), 0x42);
    assert_eq!(cpu.regs.get_hl(), 0xC001);
    assert_eq!(cpu.pc, 0x0101);
}


#[test]
fn ld_a_hld_reads_and_decrements_hl() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_hl(0xC000);
    cpu.bus.write_byte(0xC000, 0x99);

    cpu.bus.write_byte(0x0100, 0x3A); // LD A,(HL-)
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x99);
    assert_eq!(cpu.regs.get_hl(), 0xBFFF);
    assert_eq!(cpu.pc, 0x0101);
}



#[test]
fn ld_a16_a_writes_immediate_address() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0xAB);

    cpu.bus.write_byte(0x0100, 0xEA); // LD (a16),A
    cpu.bus.write_byte(0x0101, 0x34);
    cpu.bus.write_byte(0x0102, 0x12); // addr 0x1234

    cpu.step();

    assert_eq!(cpu.bus.read_byte(0x1234), 0xAB);
    assert_eq!(cpu.pc, 0x0103);
}


#[test]
fn ldh_a8_a_writes_to_ff00_plus_a8() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_a(0x55);

    cpu.bus.write_byte(0x0100, 0xE0); // LDH (a8),A
    cpu.bus.write_byte(0x0101, 0x80); // -> 0xFF80

    cpu.step();

    assert_eq!(cpu.bus.read_byte(0xFF80), 0x55);
    assert_eq!(cpu.pc, 0x0102);
}




#[test]
fn ld_a_c_reads_from_ff00_plus_c_and_preserves_flags() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_c(0x10);
    cpu.bus.write_byte(0xFF10, 0xDE);

    cpu.regs.set_z(true);
    cpu.regs.set_n(true);
    cpu.regs.set_hc(true);
    cpu.regs.set_carry(true);

    cpu.bus.write_byte(0x0100, 0xF2); // LD A,(C)
    cpu.step();

    assert_eq!(cpu.regs.a(), 0xDE);

    assert!(cpu.regs.get_z());
    assert!(cpu.regs.get_n());
    assert!(cpu.regs.get_hc());
    assert!(cpu.regs.get_carry());

    assert_eq!(cpu.pc, 0x0101);
}
