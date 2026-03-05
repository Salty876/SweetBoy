use sweetboy_core::Cpu;

#[test]
fn interrupt_vblank_pushes_pc_and_jumps_to_0040() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.sp = 0xFFFE;

    cpu.ime = true;

    cpu.bus.write_byte(0xFFFF, 0x01); // IE enable VBlank
    cpu.bus.write_byte(0xFF0F, 0x01); // IF request VBlank

    cpu.step();

    assert_eq!(cpu.pc, 0x0040);

    // Return address 0x0100 pushed little endian at SP=0xFFFC
    assert_eq!(cpu.sp, 0xFFFC);
    assert_eq!(cpu.bus.read_byte(0xFFFC), 0x00);
    assert_eq!(cpu.bus.read_byte(0xFFFD), 0x01);

    // IF bit cleared
    assert_eq!(cpu.bus.read_byte(0xFF0F) & 0x01, 0x00);

    // IME cleared
    assert!(!cpu.ime);
}
