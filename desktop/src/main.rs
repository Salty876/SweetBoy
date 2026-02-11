use gb_core::cpu::Cpu;


pub fn main() {
    let rom = std::fs::read("../blaarg/cpu_instr/individual/01-special.gb").unwrap();

    let mut cpu = Cpu::new();
    cpu.load_rom(&rom);

    // power-up state
    cpu.regs.set_af(0x01B0);
    cpu.regs.set_bc(0x0013);
    cpu.regs.set_de(0x00D8);
    cpu.regs.set_hl(0x014D);
    cpu.sp = 0xFFFE;
    cpu.pc = 0x0100;

    loop {
        cpu.step();
        if cpu.halted {
            break;
        }
    }

}


