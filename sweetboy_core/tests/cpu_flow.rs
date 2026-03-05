use sweetboy_core::Cpu;

#[test]
// #[ignore] // remove ignore once opcode 0xC3 is implemented
fn jp_a16_jumps_to_target() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    // JP 0x0200  (little endian: 00 02)
    cpu.bus.write_byte(0x0100, 0xC3);
    cpu.bus.write_byte(0x0101, 0x00);
    cpu.bus.write_byte(0x0102, 0x02);

    cpu.step();

    assert_eq!(cpu.pc, 0x0200);
}

#[test]
// #[ignore] // remove ignore once 0xCD (CALL) + 0xC9 (RET) are implemented
fn call_then_ret_returns_to_next_instruction() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.sp = 0xFFFE;

    // 0100: CD 00 02   CALL 0x0200
    // 0103: 00         NOP
    // 0200: C9         RET
    cpu.bus.write_byte(0x0100, 0xCD);
    cpu.bus.write_byte(0x0101, 0x00);
    cpu.bus.write_byte(0x0102, 0x02);
    cpu.bus.write_byte(0x0103, 0x00);

    cpu.bus.write_byte(0x0200, 0xC9);

    cpu.step();
    assert_eq!(cpu.pc, 0x0200);
    assert_eq!(cpu.sp, 0xFFFC);

    cpu.step();
    assert_eq!(cpu.pc, 0x0103);
    assert_eq!(cpu.sp, 0xFFFE);

    cpu.step();
    assert_eq!(cpu.pc, 0x0104);
}
