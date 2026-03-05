use sweetboy_core::Cpu;

#[test]
fn halt_bug_causes_next_immediate_to_read_opcode_byte() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    // IME = 0
    cpu.ime = false;

    // Make an interrupt pending BEFORE HALT
    cpu.bus.write_byte(0xFFFF, 0x01); // IE: enable VBlank
    cpu.bus.write_byte(0xFF0F, 0x01); // IF: request VBlank

    // Program:
    // 0100: HALT
    // 0101: LD A, d8
    // 0102: 0x12 (the intended immediate)
    cpu.bus.write_byte(0x0100, 0x76); // HALT
    cpu.bus.write_byte(0x0101, 0x3E); // LD A,d8
    cpu.bus.write_byte(0x0102, 0x12);

    // Step 1: execute HALT -> triggers halt bug, PC advances to 0101 (normal)
    cpu.step();
    assert_eq!(cpu.pc, 0x0101);

    // Step 2: fetch glitch: opcode fetched from 0101 but PC NOT incremented for fetch
    // LD A,d8 will then read "immediate" from 0101 again (0x3E), not 0x12.
    cpu.step();

    assert_eq!(cpu.regs.a(), 0x3E);
    assert_eq!(cpu.pc, 0x0102); // ends one byte earlier than normal (normal would be 0103)
}
