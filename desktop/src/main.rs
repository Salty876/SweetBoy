use gb_core::cpu::Cpu;


pub fn main() {
    let mut cpu = Cpu::new();
    loop {
        
        cpu.step();
        if cpu.halted {
            break;
        }
    }
}


