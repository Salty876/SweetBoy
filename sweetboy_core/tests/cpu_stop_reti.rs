use sweetboy_core::Cpu;

#[test]
fn stop_consumes_padding_and_advances_pc_by_2_and_stops() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.bus.write_byte(0x0100, 0x10); // STOP
    cpu.bus.write_byte(0x0101, 0x00); // padding
    cpu.bus.write_byte(0x0102, 0x3E); // LD A,d8 (should not run)
    cpu.bus.write_byte(0x0103, 0x77);

    cpu.step();

    assert_eq!(cpu.pc, 0x0102);
    assert!(cpu.stopped);

    cpu.step();
    assert_eq!(cpu.pc, 0x0102);
    assert_eq!(cpu.regs.a(), 0x00);
}

#[test]
fn stop_does_not_modify_flags() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.regs.set_z(true);
    cpu.regs.set_n(true);
    cpu.regs.set_hc(true);
    cpu.regs.set_carry(true);

    cpu.bus.write_byte(0x0100, 0x10); // STOP
    cpu.bus.write_byte(0x0101, 0x00); // padding

    cpu.step();

    assert!(cpu.regs.get_z());
    assert!(cpu.regs.get_n());
    assert!(cpu.regs.get_hc());
    assert!(cpu.regs.get_carry());
    assert_eq!(cpu.pc, 0x0102);
}

#[test]
fn reti_pops_pc_and_sets_ime_true_immediately() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.sp = 0xFFFC;

    cpu.ime = false;
    cpu.ime_scheduled = false;

    // Put return address 0x1234 at stack (little endian)
    cpu.bus.write_byte(0xFFFC, 0x34);
    cpu.bus.write_byte(0xFFFD, 0x12);

    cpu.bus.write_byte(0x0100, 0xD9); // RETI

    cpu.step();

    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cpu.sp, 0xFFFE);
    assert!(cpu.ime);
    assert!(!cpu.ime_scheduled);
}

#[test]
fn reti_does_not_modify_flags() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.sp = 0xFFFC;

    cpu.regs.set_z(true);
    cpu.regs.set_n(true);
    cpu.regs.set_hc(true);
    cpu.regs.set_carry(true);

    cpu.bus.write_byte(0xFFFC, 0x00);
    cpu.bus.write_byte(0xFFFD, 0x02); // return 0x0200

    cpu.bus.write_byte(0x0100, 0xD9); // RETI

    cpu.step();

    assert_eq!(cpu.pc, 0x0200);

    assert!(cpu.regs.get_z());
    assert!(cpu.regs.get_n());
    assert!(cpu.regs.get_hc());
    assert!(cpu.regs.get_carry());
}
