use sweetboy_core::Cpu;

#[test]
fn step_nop_advances_pc_by_1() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.bus.write_byte(0x0100, 0x00); // NOP

    cpu.step();

    assert_eq!(cpu.pc, 0x0101);
}

#[test]
fn halt_stops_stepping() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.bus.write_byte(0x0100, 0x76); // HALT

    cpu.step();
    assert!(cpu.halted);

    let pc_after = cpu.pc;
    cpu.step();
    assert_eq!(cpu.pc, pc_after);
}
