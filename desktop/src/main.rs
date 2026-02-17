use gb_core::cpu::Cpu;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const SCALE: usize = 3;
const CYCLES_PER_FRAME: u32 = 70_224; // T-cycles per frame (~59.7 Hz)
const DOTS_PER_LINE: u32 = 456;       // T-cycles per scanline

pub fn main() {
    // ── Load ROM ──
    let rom_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../Tetris (World) (Rev 1).gb".into());

    let rom = std::fs::read(&rom_path)
        .unwrap_or_else(|e| panic!("Failed to read ROM '{}': {}", rom_path, e));

    println!("Loaded: {} ({} KB)", rom_path, rom.len() / 1024);
    print_rom_header(&rom);

    if rom.len() > 0x8000 {
        println!("WARNING: ROM is > 32 KB — MBC bank switching is not implemented yet.");
        println!("         Only the first 32 KB will be visible. Expect issues!\n");
    }

    // ── Init CPU (post-boot DMG state) ──
    let mut cpu = Cpu::new();
    cpu.regs.set_af(0x01B0);
    cpu.regs.set_bc(0x0013);
    cpu.regs.set_de(0x00D8);
    cpu.regs.set_hl(0x014D);
    cpu.sp = 0xFFFE;
    cpu.pc = 0x0100;
    cpu.ime = false;
    cpu.ime_scheduled = false;

    cpu.load_rom(&rom);

    // Post-boot IO register state
    cpu.bus.write_byte(0xFF40, 0x91); // LCDC: LCD on, BG on
    cpu.bus.write_byte(0xFF41, 0x85); // STAT
    cpu.bus.write_byte(0xFF42, 0x00); // SCY
    cpu.bus.write_byte(0xFF43, 0x00); // SCX
    cpu.bus.write_byte(0xFF47, 0xFC); // BGP: classic palette
    cpu.bus.write_byte(0xFF48, 0xFF); // OBP0
    cpu.bus.write_byte(0xFF49, 0xFF); // OBP1
    cpu.bus.write_byte(0xFFFF, 0x01); // IE: VBlank enabled

    // ── Window ──
    let mut window = Window::new(
        "LebronBoy",
        WIDTH * SCALE,
        HEIGHT * SCALE,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.set_target_fps(60);

    let mut framebuffer = vec![0u32; WIDTH * HEIGHT];

    // ── Main loop ──
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Update joypad from keyboard
        update_joypad(&mut cpu, &window);

        // Run one frame of CPU cycles with fake scanline timing
        run_frame(&mut cpu);

        // Render background tiles from VRAM → framebuffer
        render_background(&cpu, &mut framebuffer);

        window
            .update_with_buffer(&framebuffer, WIDTH, HEIGHT)
            .unwrap();
    }
}

// ─────────────────────────── Frame / fake PPU timing ───────────────────────────

fn run_frame(cpu: &mut Cpu) {
    let mut frame_cycles: u32 = 0;
    let mut line_cycles: u32 = 0;
    let mut ly: u8 = 0;

    while frame_cycles < CYCLES_PER_FRAME {
        cpu.step();
        let step_t = cpu.last_cycle_timestamp as u32;
        frame_cycles += step_t;
        line_cycles += step_t;

        if line_cycles >= DOTS_PER_LINE {
            line_cycles -= DOTS_PER_LINE;
            ly += 1;

            if ly == 144 {
                // Enter VBlank — fire interrupt
                cpu.bus.request_interrupt(0x01);
            } else if ly > 153 {
                ly = 0;
            }

            // Write LY directly (no PPU yet)
            cpu.bus.memory[0xFF44] = ly;

            // Simple LYC==LY compare (STAT interrupt bit 6)
            let lyc = cpu.bus.memory[0xFF45];
            if ly == lyc {
                // Set coincidence flag (bit 2 of STAT)
                cpu.bus.memory[0xFF41] |= 0x04;
            } else {
                cpu.bus.memory[0xFF41] &= !0x04;
            }
        }
    }
}

// ─────────────────────────── Joypad ───────────────────────────

fn update_joypad(cpu: &mut Cpu, window: &Window) {
    // D-pad: Right=0, Left=1, Up=2, Down=3  (active low)
    let mut direction: u8 = 0x0F;
    if window.is_key_down(Key::Right) { direction &= !0x01; }
    if window.is_key_down(Key::Left)  { direction &= !0x02; }
    if window.is_key_down(Key::Up)    { direction &= !0x04; }
    if window.is_key_down(Key::Down)  { direction &= !0x08; }

    // Buttons: A=0, B=1, Select=2, Start=3  (active low)
    let mut action: u8 = 0x0F;
    if window.is_key_down(Key::Z)     { action &= !0x01; } // A
    if window.is_key_down(Key::X)     { action &= !0x02; } // B
    if window.is_key_down(Key::Space) { action &= !0x04; } // Select
    if window.is_key_down(Key::Enter) { action &= !0x08; } // Start

    cpu.bus.joypad_direction = direction;
    cpu.bus.joypad_action = action;
}

// ─────────────────────────── BG Renderer ───────────────────────────

fn render_background(cpu: &Cpu, fb: &mut [u32]) {
    let lcdc = cpu.bus.memory[0xFF40];

    // If LCD is off, show blank
    if lcdc & 0x80 == 0 {
        fb.fill(SHADES[0]);
        return;
    }

    let scy = cpu.bus.memory[0xFF42] as usize;
    let scx = cpu.bus.memory[0xFF43] as usize;
    let bgp = cpu.bus.memory[0xFF47];

    let palette = [
        SHADES[((bgp >> 0) & 3) as usize],
        SHADES[((bgp >> 2) & 3) as usize],
        SHADES[((bgp >> 4) & 3) as usize],
        SHADES[((bgp >> 6) & 3) as usize],
    ];

    // BG tile map: LCDC bit 3
    let map_base: usize = if lcdc & 0x08 != 0 { 0x9C00 } else { 0x9800 };
    // Tile data addressing: LCDC bit 4
    let signed_addr = lcdc & 0x10 == 0;

    for sy in 0..HEIGHT {
        let map_y = (sy + scy) & 0xFF;
        let tile_row = map_y / 8;
        let pixel_y = map_y % 8;

        for sx in 0..WIDTH {
            let map_x = (sx + scx) & 0xFF;
            let tile_col = map_x / 8;
            let pixel_x = 7 - (map_x % 8);

            let tile_id = cpu.bus.memory[map_base + tile_row * 32 + tile_col];

            let tile_addr = if signed_addr {
                // Signed: tile 0 at 0x9000, range 0x8800-0x97FF
                (0x9000i32 + (tile_id as i8 as i32) * 16) as usize
            } else {
                0x8000 + tile_id as usize * 16
            };

            let lo = cpu.bus.memory[tile_addr + pixel_y * 2];
            let hi = cpu.bus.memory[tile_addr + pixel_y * 2 + 1];
            let color_id = ((hi >> pixel_x) & 1) << 1 | ((lo >> pixel_x) & 1);

            fb[sy * WIDTH + sx] = palette[color_id as usize];
        }
    }
}

/// Classic DMG green shades
const SHADES: [u32; 4] = [
    0x00E0_F8D0, // lightest
    0x0088_C070, // light
    0x0034_6856, // dark
    0x0008_1820, // darkest
];

// ─────────────────────────── ROM Header Info ───────────────────────────

fn print_rom_header(rom: &[u8]) {
    if rom.len() < 0x150 {
        return;
    }

    // Title: 0x0134..0x0143
    let title: String = rom[0x0134..0x0143]
        .iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect();

    let cart_type = rom[0x0147];
    let rom_size = rom[0x0148];
    let ram_size = rom[0x0149];

    let cart_name = match cart_type {
        0x00 => "ROM ONLY",
        0x01 => "MBC1",
        0x02 => "MBC1+RAM",
        0x03 => "MBC1+RAM+BATTERY",
        0x05 => "MBC2",
        0x06 => "MBC2+BATTERY",
        0x0F => "MBC3+TIMER+BATTERY",
        0x10 => "MBC3+TIMER+RAM+BATTERY",
        0x11 => "MBC3",
        0x12 => "MBC3+RAM",
        0x13 => "MBC3+RAM+BATTERY",
        0x19 => "MBC5",
        0x1A => "MBC5+RAM",
        0x1B => "MBC5+RAM+BATTERY",
        _ => "Unknown",
    };

    let rom_kb = 32 << rom_size;

    println!("Title:  {}", title);
    println!("Cart:   0x{:02X} ({})", cart_type, cart_name);
    println!("ROM:    {} KB  |  RAM size byte: 0x{:02X}", rom_kb, ram_size);
    println!("---");
}
