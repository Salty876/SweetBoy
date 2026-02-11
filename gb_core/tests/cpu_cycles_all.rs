use gb_core::Cpu;

// Fill these with the official cycle counts you want to enforce.
// Non-CB: 256 entries, CB: 256 entries
// Use 0 as "skip" for opcodes you test separately (conditional/timing weirdness).
const EXPECTED_CYCLES: [u8; 256] = [0; 256];
const EXPECTED_CB_CYCLES: [u8; 256] = [0; 256];

// A list of opcodes that are path-dependent or special and should be tested separately.
// You can expand this list as needed.
fn is_special(op: u8) -> bool {
    matches!(
        op,
        // conditional JP/JR/CALL/RET
        0x20 | 0x28 | 0x30 | 0x38 | // JR cc,e8
        0xC2 | 0xCA | 0xD2 | 0xDA | // JP cc,a16
        0xC4 | 0xCC | 0xD4 | 0xDC | // CALL cc,a16
        0xC0 | 0xC8 | 0xD0 | 0xD8 | // RET cc
        // HALT/STOP can be environment dependent
        0x10 | 0x76
    )
}

// Put CPU into a safe, deterministic state so “random opcode” won’t corrupt stuff.
fn setup_cpu_for_cycle_test(op: u8) -> Cpu {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    // Keep interrupts off so timing isn't affected by servicing.
    cpu.ime = false;
    cpu.ime_scheduled = false;

    // Ensure no pending interrupts.
    cpu.bus.write_byte(0xFFFF, 0x00); // IE
    cpu.bus.write_byte(0xFF0F, 0x00); // IF

    // Reasonable stack.
    cpu.sp = 0xFFFE;

    // Put HL somewhere safe for (HL) reads/writes.
    cpu.regs.set_hl(0xC000);
    cpu.bus.write_byte(0xC000, 0x00);

    // Provide safe immediates (d8, e8, a16) in case the opcode reads them.
    cpu.bus.write_byte(0x0100, op);
    cpu.bus.write_byte(0x0101, 0x00);
    cpu.bus.write_byte(0x0102, 0x00);

    // Also give “a16” a safe address (0xC000) by setting bytes accordingly.
    // little endian: 0xC000 => 0x00, 0xC0
    cpu.bus.write_byte(0x0101, 0x00);
    cpu.bus.write_byte(0x0102, 0xC0);

    // Avoid STOP/HALT freezing the harness.
    cpu.halted = false;
    cpu.stopped = false;

    cpu
}

#[test]
fn cycles_all_non_cb_opcodes_match_table() {
    for op in 0u16..=0xFF {
        let op = op as u8;

        if is_special(op) {
            continue;
        }

        let expected = EXPECTED_CYCLES[op as usize];
        if expected == 0 {
            // Not asserted here; either fill the table or handle in a dedicated test.
            continue;
        }

        let mut cpu = setup_cpu_for_cycle_test(op);

        // If an opcode might read/write memory-mapped IO, still okay; bus should handle it.
        cpu.step();
        let cycles = cpu.last_cycle_timestamp;

        assert_eq!(
            cycles, expected,
            "cycle mismatch for opcode 0x{:02X}",
            op
        );
    }
}

#[test]
fn cycles_all_cb_opcodes_match_table() {
    for cb in 0u16..=0xFF {
        let cb = cb as u8;

        let expected = EXPECTED_CB_CYCLES[cb as usize];
        if expected == 0 {
            continue;
        }

        let mut cpu = Cpu::new();
        cpu.pc = 0x0100;

        cpu.ime = false;
        cpu.ime_scheduled = false;
        cpu.bus.write_byte(0xFFFF, 0x00);
        cpu.bus.write_byte(0xFF0F, 0x00);

        cpu.sp = 0xFFFE;

        // HL safe for CB (HL)
        cpu.regs.set_hl(0xC000);
        cpu.bus.write_byte(0xC000, 0x00);

        cpu.bus.write_byte(0x0100, 0xCB);
        cpu.bus.write_byte(0x0101, cb);

        cpu.step();
        let cycles = cpu.last_cycle_timestamp;


        assert_eq!(
            cycles, expected,
            "cycle mismatch for CB opcode 0xCB 0x{:02X}",
            cb
        );
    }
}

// ---- Path-dependent tests (taken vs not taken) ----

#[test]
fn cycles_jr_cc_taken_vs_not_taken() {
    // JR NZ,e8 (0x20): taken if Z=0, not taken if Z=1
    // Repeat pattern for Z/NC/C.
    let cases = [
        (0x20u8, false, 12u8, 8u8), // NZ
        (0x28u8, true,  12u8, 8u8), // Z
        (0x30u8, false, 12u8, 8u8), // NC
        (0x38u8, true,  12u8, 8u8), // C
    ];

    for &(op, flag_true_means_taken, taken_cycles, not_taken_cycles) in &cases {
        // taken
        {
            let mut cpu = setup_cpu_for_cycle_test(op);
            cpu.bus.write_byte(0x0101, 0x02); // small offset
            if op == 0x20 { cpu.regs.set_z(flag_true_means_taken); } // NZ: taken when Z=false
            if op == 0x28 { cpu.regs.set_z(flag_true_means_taken); } // Z: taken when Z=true
            if op == 0x30 { cpu.regs.set_carry(!flag_true_means_taken); } // NC: taken when C=false
            if op == 0x38 { cpu.regs.set_carry(flag_true_means_taken); }  // C: taken when C=true

            cpu.step();
        let cycles = cpu.last_cycle_timestamp;

            assert_eq!(cycles, taken_cycles, "JR taken cycles wrong for 0x{:02X}", op);
        }

        // not taken
        {
            let mut cpu = setup_cpu_for_cycle_test(op);
            cpu.bus.write_byte(0x0101, 0x02);
            if op == 0x20 { cpu.regs.set_z(!flag_true_means_taken); }
            if op == 0x28 { cpu.regs.set_z(!flag_true_means_taken); }
            if op == 0x30 { cpu.regs.set_carry(flag_true_means_taken); }
            if op == 0x38 { cpu.regs.set_carry(!flag_true_means_taken); }

            cpu.step();
        let cycles = cpu.last_cycle_timestamp;

            assert_eq!(cycles, not_taken_cycles, "JR not-taken cycles wrong for 0x{:02X}", op);
        }
    }
}

#[test]
fn cycles_jp_cc_taken_vs_not_taken() {
    // JP cc,a16: taken 16, not 12
    let cases = [
        (0xC2u8, "NZ"),
        (0xCAu8, "Z"),
        (0xD2u8, "NC"),
        (0xDAu8, "C"),
    ];

    for &(op, cond) in &cases {
        // taken setup
        let mut cpu = setup_cpu_for_cycle_test(op);
        if cond == "NZ" { cpu.regs.set_z(false); }
        if cond == "Z"  { cpu.regs.set_z(true); }
        if cond == "NC" { cpu.regs.set_carry(false); }
        if cond == "C"  { cpu.regs.set_carry(true); }
        cpu.step();
        let cycles = cpu.last_cycle_timestamp;

        assert_eq!(cycles, 16, "JP {} taken cycles wrong", cond);

        // not taken setup
        let mut cpu = setup_cpu_for_cycle_test(op);
        if cond == "NZ" { cpu.regs.set_z(true); }
        if cond == "Z"  { cpu.regs.set_z(false); }
        if cond == "NC" { cpu.regs.set_carry(true); }
        if cond == "C"  { cpu.regs.set_carry(false); }
        cpu.step();
        let cycles = cpu.last_cycle_timestamp;

        assert_eq!(cycles, 12, "JP {} not-taken cycles wrong", cond);
    }
}

#[test]
fn cycles_call_cc_taken_vs_not_taken() {
    // CALL cc,a16: taken 24, not 12
    let cases = [
        (0xC4u8, "NZ"),
        (0xCCu8, "Z"),
        (0xD4u8, "NC"),
        (0xDCu8, "C"),
    ];

    for &(op, cond) in &cases {
        // taken
        let mut cpu = setup_cpu_for_cycle_test(op);
        if cond == "NZ" { cpu.regs.set_z(false); }
        if cond == "Z"  { cpu.regs.set_z(true); }
        if cond == "NC" { cpu.regs.set_carry(false); }
        if cond == "C"  { cpu.regs.set_carry(true); }
        cpu.step();
        let cycles = cpu.last_cycle_timestamp;

        assert_eq!(cycles, 24, "CALL {} taken cycles wrong", cond);

        // not taken
        let mut cpu = setup_cpu_for_cycle_test(op);
        if cond == "NZ" { cpu.regs.set_z(true); }
        if cond == "Z"  { cpu.regs.set_z(false); }
        if cond == "NC" { cpu.regs.set_carry(true); }
        if cond == "C"  { cpu.regs.set_carry(false); }
        cpu.step();
        let cycles = cpu.last_cycle_timestamp;

        assert_eq!(cycles, 12, "CALL {} not-taken cycles wrong", cond);
    }
}

#[test]
fn cycles_ret_cc_taken_vs_not_taken() {
    // RET cc: taken 20, not 8
    let cases = [
        (0xC0u8, "NZ"),
        (0xC8u8, "Z"),
        (0xD0u8, "NC"),
        (0xD8u8, "C"),
    ];

    for &(op, cond) in &cases {
        // taken: must have a valid return address on stack
        let mut cpu = setup_cpu_for_cycle_test(op);
        cpu.sp = 0xFFFC;
        cpu.bus.write_byte(0xFFFC, 0x00);
        cpu.bus.write_byte(0xFFFD, 0x02); // return 0x0200

        if cond == "NZ" { cpu.regs.set_z(false); }
        if cond == "Z"  { cpu.regs.set_z(true); }
        if cond == "NC" { cpu.regs.set_carry(false); }
        if cond == "C"  { cpu.regs.set_carry(true); }

        cpu.step();
        let cycles = cpu.last_cycle_timestamp;

        assert_eq!(cycles, 20, "RET {} taken cycles wrong", cond);

        // not taken: no pop
        let mut cpu = setup_cpu_for_cycle_test(op);
        if cond == "NZ" { cpu.regs.set_z(true); }
        if cond == "Z"  { cpu.regs.set_z(false); }
        if cond == "NC" { cpu.regs.set_carry(true); }
        if cond == "C"  { cpu.regs.set_carry(false); }

        cpu.step();
        let cycles = cpu.last_cycle_timestamp;

        assert_eq!(cycles, 8, "RET {} not-taken cycles wrong", cond);
    }
}
