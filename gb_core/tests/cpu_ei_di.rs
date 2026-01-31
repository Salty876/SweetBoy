use gb_core::Cpu;

#[test]
fn ei_does_not_enable_ime_immediately() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.ime = false;

    cpu.bus.write_byte(0x0100, 0xFB); // EI

    cpu.step();

    assert!(!cpu.ime);
    assert!(cpu.ime_scheduled);
    assert_eq!(cpu.pc, 0x0101);
}


#[test]
fn ei_enables_ime_after_next_instruction() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.ime = false;

    cpu.bus.write_byte(0x0100, 0xFB); // EI
    cpu.bus.write_byte(0x0101, 0x00); // NOP

    cpu.step(); // EI
    assert!(!cpu.ime);
    assert!(cpu.ime_scheduled);

    cpu.step(); // NOP, then IME flips
    assert!(cpu.ime);
    assert!(!cpu.ime_scheduled);
    assert_eq!(cpu.pc, 0x0102);
}



#[test]
fn di_cancels_scheduled_ei() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.ime = false;

    cpu.bus.write_byte(0x0100, 0xFB); // EI
    cpu.bus.write_byte(0x0101, 0xF3); // DI

    cpu.step(); // EI schedules
    assert!(cpu.ime_scheduled);

    cpu.step(); // DI cancels
    assert!(!cpu.ime);
    assert!(!cpu.ime_scheduled);
    assert_eq!(cpu.pc, 0x0102);
}
