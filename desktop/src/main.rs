use gb_core::cpu::Cpu;

pub fn main() {
    let mut cpu = Cpu::new();
    
    let mut steps: u64 = 0;
    let mut last_pc = 0u16;
    let mut same_pc_count: u32 = 0;
    let mut count = 0;
    let mut serial_output = String::new();

    let rom_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../Pokemon - Red Version (USA, Europe) (SGB Enhanced).gb".into());

    let rom = std::fs::read(&rom_path)
        .unwrap_or_else(|e| panic!("Failed to read ROM '{}': {}", rom_path, e));

    cpu.regs.set_af(0x01B0);
    cpu.regs.set_bc(0x0013);
    cpu.regs.set_de(0x00D8);
    cpu.regs.set_hl(0x014D);
    cpu.sp = 0xFFFE;
    cpu.pc = 0x0100;

    // also force no interrupts for now
    cpu.bus.write_byte(0xFFFF, 0x00); // IE
    cpu.bus.write_byte(0xFF0F, 0x00); // IF
    cpu.ime = false;
    cpu.ime_scheduled = false;


    cpu.load_rom(&rom);
    println!(
    "ROM[0100]={:02X} {:02X} {:02X} {:02X}",
    cpu.bus.read_byte(0x0100),
    cpu.bus.read_byte(0x0101),
    cpu.bus.read_byte(0x0102),
    cpu.bus.read_byte(0x0103),
    );
    println!("PC={:04X}", cpu.pc);

    loop {
    let pc = cpu.pc;
    let op = cpu.bus.read_byte(pc);

    cpu.step();
    steps += 1;



        // Detect JR -2 spin loop (0x18 0xFE) — test is done
        if cpu.pc == last_pc {
            same_pc_count += 1;
            if same_pc_count > 100 {
                println!("\n\nSpin loop detected at PC={:04X}, test finished.", cpu.pc);
                println!("Serial output:\n{}", serial_output);
                break;
            }
        } else {
            same_pc_count = 0;
            last_pc = cpu.pc;
        }

        if steps % 100_000 == 0 {
        println!("PC={:04X} OP={:02X} A={:02X} F={:02X} SP={:04X} TIMA={:02X} TAC={:02X} IF={:02X} IE={:02X} IME={}",
            cpu.pc,
            cpu.bus.read_byte(cpu.pc),
            cpu.regs.a(),
            cpu.regs.f(),
            cpu.sp,
            cpu.bus.read_byte(0xFF05),
            cpu.bus.read_byte(0xFF07),
            cpu.bus.read_byte(0xFF0F),
            cpu.bus.read_byte(0xFFFF),
            cpu.ime,
        );
        count += 1;
        if count > 200 {
            println!("Timeout after {} steps", steps);
            break;
        }
    }



    }

}
