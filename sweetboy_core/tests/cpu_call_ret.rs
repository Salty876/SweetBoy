use sweetboy_core::Cpu;

// Helper: write 16-bit little-endian immediate to memory
fn write_u16_le(cpu: &mut Cpu, addr: u16, value: u16) {
    cpu.bus.write_byte(addr, (value & 0x00FF) as u8);
    cpu.bus.write_byte(addr.wrapping_add(1), (value >> 8) as u8);
}

#[test]
fn call_always_pushes_return_and_jumps() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.sp = 0xFFFE;

    // 0100: CD 00 02   CALL 0x0200
    cpu.bus.write_byte(0x0100, 0xCD);
    write_u16_le(&mut cpu, 0x0101, 0x0200);

    cpu.step();

    // After CALL, PC should jump to target
    assert_eq!(cpu.pc, 0x0200);

    // SP should decrease by 2 (stack grows downward)
    assert_eq!(cpu.sp, 0xFFFC);

    // Return address should be 0x0103 (next instruction after CALL)
    // With typical push_word: memory[SP] = low, memory[SP+1] = high
    assert_eq!(cpu.bus.read_byte(cpu.sp), 0x03);
    assert_eq!(cpu.bus.read_byte(cpu.sp.wrapping_add(1)), 0x01);
}

#[test]
fn ret_always_pops_pc() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0200;
    cpu.sp = 0xFFFC;

    // Stack contains return address 0x0103 (lo at SP, hi at SP+1)
    cpu.bus.write_byte(0xFFFC, 0x03);
    cpu.bus.write_byte(0xFFFD, 0x01);

    // 0200: C9   RET
    cpu.bus.write_byte(0x0200, 0xC9);

    cpu.step();

    assert_eq!(cpu.pc, 0x0103);
    assert_eq!(cpu.sp, 0xFFFE);
}

#[test]
fn call_nz_taken_when_z_is_false() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.sp = 0xFFFE;

    // Ensure Z=0 => NZ condition is true
    cpu.regs.set_z(false);

    // 0100: C4 00 02   CALL NZ, 0x0200
    cpu.bus.write_byte(0x0100, 0xC4);
    write_u16_le(&mut cpu, 0x0101, 0x0200);

    cpu.step();

    assert_eq!(cpu.pc, 0x0200);
    assert_eq!(cpu.sp, 0xFFFC);
    assert_eq!(cpu.bus.read_byte(cpu.sp), 0x03);
    assert_eq!(cpu.bus.read_byte(cpu.sp.wrapping_add(1)), 0x01);
}

#[test]
fn call_nz_not_taken_when_z_is_true() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.sp = 0xFFFE;

    // Ensure Z=1 => NZ condition is false
    cpu.regs.set_z(true);

    // 0100: C4 00 02   CALL NZ, 0x0200
    cpu.bus.write_byte(0x0100, 0xC4);
    write_u16_le(&mut cpu, 0x0101, 0x0200);

    cpu.step();

    // Not taken: PC should advance past immediate
    assert_eq!(cpu.pc, 0x0103);

    // Not taken: SP and memory unchanged
    assert_eq!(cpu.sp, 0xFFFE);
}

#[test]
fn ret_z_taken_when_z_is_true() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0200;
    cpu.sp = 0xFFFC;

    cpu.regs.set_z(true);

    // Stack contains return address 0x1234
    cpu.bus.write_byte(0xFFFC, 0x34);
    cpu.bus.write_byte(0xFFFD, 0x12);

    // 0200: C8   RET Z
    cpu.bus.write_byte(0x0200, 0xC8);

    cpu.step();

    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cpu.sp, 0xFFFE);
}

#[test]
fn ret_z_not_taken_when_z_is_false() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0200;
    cpu.sp = 0xFFFC;

    cpu.regs.set_z(false);

    // Put some bytes on stack (should NOT be popped)
    cpu.bus.write_byte(0xFFFC, 0x34);
    cpu.bus.write_byte(0xFFFD, 0x12);

    // 0200: C8   RET Z
    cpu.bus.write_byte(0x0200, 0xC8);

    cpu.step();

    // Not taken: PC just advances by 1 (no immediate)
    assert_eq!(cpu.pc, 0x0201);

    // Not taken: SP unchanged and stack not consumed
    assert_eq!(cpu.sp, 0xFFFC);
    assert_eq!(cpu.bus.read_byte(0xFFFC), 0x34);
    assert_eq!(cpu.bus.read_byte(0xFFFD), 0x12);
}

#[test]
fn call_c_taken_when_carry_is_true() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.sp = 0xFFFE;

    cpu.regs.set_carry(true);

    // 0100: DC 00 02   CALL C, 0x0200
    cpu.bus.write_byte(0x0100, 0xDC);
    write_u16_le(&mut cpu, 0x0101, 0x0200);

    cpu.step();

    assert_eq!(cpu.pc, 0x0200);
    assert_eq!(cpu.sp, 0xFFFC);
    assert_eq!(cpu.bus.read_byte(cpu.sp), 0x03);
    assert_eq!(cpu.bus.read_byte(cpu.sp.wrapping_add(1)), 0x01);
}

#[test]
fn ret_nc_taken_when_carry_is_false() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0200;
    cpu.sp = 0xFFFC;

    cpu.regs.set_carry(false);

    // Stack contains return address 0x4567
    cpu.bus.write_byte(0xFFFC, 0x67);
    cpu.bus.write_byte(0xFFFD, 0x45);

    // 0200: D0   RET NC
    cpu.bus.write_byte(0x0200, 0xD0);

    cpu.step();

    assert_eq!(cpu.pc, 0x4567);
    assert_eq!(cpu.sp, 0xFFFE);
}
