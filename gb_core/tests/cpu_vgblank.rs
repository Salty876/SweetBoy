use gb_core::Cpu;

#[test]
fn interrupt_vblank_pushes_pc_clears_if_and_jumps() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.sp = 0xFFFE;

    cpu.ime = true;

    cpu.bus.write_byte(0xFFFF, 0x01); // IE: VBlank enabled
    cpu.bus.write_byte(0xFF0F, 0x01); // IF: VBlank requested

    cpu.step();

    assert_eq!(cpu.pc, 0x0040);
    assert!(!cpu.ime);

    // IF bit cleared
    assert_eq!(cpu.bus.read_byte(0xFF0F) & 0x01, 0x00);

    // return address pushed (0x0100) little endian
    assert_eq!(cpu.sp, 0xFFFC);
    assert_eq!(cpu.bus.read_byte(0xFFFC), 0x00);
    assert_eq!(cpu.bus.read_byte(0xFFFD), 0x01);
}

#[test]
fn interrupt_priority_vblank_beats_timer() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0200;
    cpu.sp = 0xFFFE;

    cpu.ime = true;

    cpu.bus.write_byte(0xFFFF, 0x05); // IE: VBlank + Timer
    cpu.bus.write_byte(0xFF0F, 0x05); // IF: VBlank + Timer

    cpu.step();

    assert_eq!(cpu.pc, 0x0040); // VBlank wins
    assert_eq!(cpu.bus.read_byte(0xFF0F) & 0x01, 0x00); // vblank cleared
    assert_ne!(cpu.bus.read_byte(0xFF0F) & 0x04, 0x00); // timer still pending
}