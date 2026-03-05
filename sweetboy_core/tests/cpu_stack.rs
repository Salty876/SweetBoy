use sweetboy_core::Cpu;

#[test]
fn push_pop_word_roundtrip_and_endianness() {
    let mut cpu = Cpu::new();
    cpu.sp = 0xFFFE;
    cpu.pc = 0x1234;

    cpu.push_word(0xBEEF);

    // PC must NOT change from stack ops
    assert_eq!(cpu.pc, 0x1234);

    // SP should go down by 2
    assert_eq!(cpu.sp, 0xFFFC);

    // With the common push_word: final [SP]=lo, [SP+1]=hi
    assert_eq!(cpu.bus.read_byte(cpu.sp), 0xEF);
    assert_eq!(cpu.bus.read_byte(cpu.sp + 1), 0xBE);

    let v = cpu.pop_word();
    assert_eq!(v, 0xBEEF);
    assert_eq!(cpu.sp, 0xFFFE);
}

#[test]
fn pop_word_reads_lo_then_hi() {
    let mut cpu = Cpu::new();
    cpu.sp = 0xFFFC;

    cpu.bus.write_byte(0xFFFC, 0x34); // lo
    cpu.bus.write_byte(0xFFFD, 0x12); // hi

    let v = cpu.pop_word();
    assert_eq!(v, 0x1234);
    assert_eq!(cpu.sp, 0xFFFE);
}
