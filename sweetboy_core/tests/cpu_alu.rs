use sweetboy_core::Cpu;

#[derive(Clone, Copy, Debug)]
struct AluCase {
    opcode: u8,
    imm: Option<u8>,      // Some(d8) for immediate ops
    a_in: u8,
    b_in: u8,             // used when opcode targets B (or ignored)
    carry_in: bool,

    a_out: u8,
    z: bool,
    n: bool,
    h: bool,
    c: bool,

    pc_advance: u16,      // 1 or 2
    a_unchanged: bool,    // for CP
}

#[test]
fn alu_smoke_table_driven() {
    let cases: &[AluCase] = &[
        // XOR B: 0xF0 ^ 0x0F = 0xFF, all flags cleared
        AluCase {
            opcode: 0xA8, imm: None,
            a_in: 0xF0, b_in: 0x0F, carry_in: true, // carry-in should be ignored
            a_out: 0xFF, z: false, n: false, h: false, c: false,
            pc_advance: 1, a_unchanged: false,
        },

        // AND B: 0x3C & 0x0F = 0x0C, H forced set
        AluCase {
            opcode: 0xA0, imm: None,
            a_in: 0x3C, b_in: 0x0F, carry_in: true,
            a_out: 0x0C, z: false, n: false, h: true, c: false,
            pc_advance: 1, a_unchanged: false,
        },

        // OR d8: 0x10 | 0x01 = 0x11
        AluCase {
            opcode: 0xF6, imm: Some(0x01),
            a_in: 0x10, b_in: 0x00, carry_in: true,
            a_out: 0x11, z: false, n: false, h: false, c: false,
            pc_advance: 2, a_unchanged: false,
        },

        // ADD A,B: 0x0F + 0x01 = 0x10 => H set
        AluCase {
            opcode: 0x80, imm: None,
            a_in: 0x0F, b_in: 0x01, carry_in: true,
            a_out: 0x10, z: false, n: false, h: true, c: false,
            pc_advance: 1, a_unchanged: false,
        },

        // ADC A,B with carry-in: 0xFF + 0x00 + 1 = 0x00 => Z,C,H set
        AluCase {
            opcode: 0x88, imm: None,
            a_in: 0xFF, b_in: 0x00, carry_in: true,
            a_out: 0x00, z: true, n: false, h: true, c: true,
            pc_advance: 1, a_unchanged: false,
        },

        // SUB B: 0x10 - 0x01 = 0x0F => N=1, H=1, C=0
        AluCase {
            opcode: 0x90, imm: None,
            a_in: 0x10, b_in: 0x01, carry_in: true,
            a_out: 0x0F, z: false, n: true, h: true, c: false,
            pc_advance: 1, a_unchanged: false,
        },

        // SBC A,B with borrow-in: 0x00 - 0x00 - 1 = 0xFF => N=1, H=1, C=1
        AluCase {
            opcode: 0x98, imm: None,
            a_in: 0x00, b_in: 0x00, carry_in: true,
            a_out: 0xFF, z: false, n: true, h: true, c: true,
            pc_advance: 1, a_unchanged: false,
        },

        // CP d8: compare 0x10 vs 0x10 => Z=1,N=1,H=0,C=0; A unchanged
        AluCase {
            opcode: 0xFE, imm: Some(0x10),
            a_in: 0x10, b_in: 0x00, carry_in: true,
            a_out: 0x10, z: true, n: true, h: false, c: false,
            pc_advance: 2, a_unchanged: true,
        },

        // CP d8: compare 0x10 vs 0x20 => borrow => N=1,C=1; A unchanged
        AluCase {
            opcode: 0xFE, imm: Some(0x20),
            a_in: 0x10, b_in: 0x00, carry_in: false,
            a_out: 0x10, z: false, n: true, h: false, c: true,
            pc_advance: 2, a_unchanged: true,
        },
    ];

    for (i, case) in cases.iter().enumerate() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x0100;

        cpu.regs.set_a(case.a_in);
        cpu.regs.set_b(case.b_in);

        // Set junk flags first so ops must overwrite deterministically.
        cpu.regs.set_z(true);
        cpu.regs.set_n(true);
        cpu.regs.set_hc(true);
        cpu.regs.set_carry(true);

        // Apply desired carry-in (for ADC/SBC), others should ignore it anyway.
        cpu.regs.set_carry(case.carry_in);

        cpu.bus.write_byte(0x0100, case.opcode);
        if let Some(imm) = case.imm {
            cpu.bus.write_byte(0x0101, imm);
        }

        cpu.step();

        // For CP, A must remain unchanged (a_out == a_in typically).
        if case.a_unchanged {
            assert_eq!(cpu.regs.a(), case.a_in, "case {i}: A should be unchanged");
        } else {
            assert_eq!(cpu.regs.a(), case.a_out, "case {i}: wrong A");
        }

        assert_eq!(cpu.regs.get_z(), case.z, "case {i}: wrong Z");
        assert_eq!(cpu.regs.get_n(), case.n, "case {i}: wrong N");
        assert_eq!(cpu.regs.get_hc(), case.h, "case {i}: wrong H");
        assert_eq!(cpu.regs.get_carry(), case.c, "case {i}: wrong C");

        assert_eq!(cpu.pc, 0x0100 + case.pc_advance, "case {i}: wrong PC");
    }
}
