use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::env;
use std::fs::File;
use std::io::Read;

use gb_core::Emulator;

const W: usize = 160;
const H: usize = 144;
const SCALE: u32 = 4; // 2,3,4,5...

// Map DMG shade index (0..3) -> grayscale byte.
// 0 is "white", 3 is "black" in many emulators.
// If your output looks inverted, swap these values.
fn shade_to_rgb(shade: u8) -> (u8, u8, u8) {
    let v = match shade & 0x03 {
        0 => 0xFF,
        1 => 0xAA,
        2 => 0x55,
        _ => 0x00,
    };
    (v, v, v)
}

// Convert your PPU framebuffer (shade indices) into RGBA8888 pixels.
fn blit_rgba(ppu_fb: &[u8; W * H], out_rgba: &mut [u8; W * H * 4]) {
    for i in 0..(W * H) {
        let (r, g, b) = shade_to_rgb(ppu_fb[i]);
        let o = i * 4;
        out_rgba[o + 0] = r;
        out_rgba[o + 1] = g;
        out_rgba[o + 2] = b;
        out_rgba[o + 3] = 0xFF;
    }
}

pub fn run_sdl(mut emu: Emulator) -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;

    let window = video
        .window("LebronBoy", (W as u32) * SCALE, (H as u32) * SCALE)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync() // simplest pacing: present at monitor refresh
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::ABGR8888, W as u32, H as u32)
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl.event_pump()?;

    // Temp pixel buffer for upload to SDL texture
    let mut rgba = [0u8; W * H * 4];

    'running: loop {
        // ---- Input/events ----
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,

                // Example joypad mapping (active-low bits)
                Event::KeyDown { keycode: Some(k), repeat: false, .. } => {
                    handle_key(&mut emu, k, true);
                }
                Event::KeyUp { keycode: Some(k), repeat: false, .. } => {
                    handle_key(&mut emu, k, false);
                }

                _ => {}
            }
        }

        // ---- Emulation step ----
        // Run until we have a full frame (PPU sets frame_ready at VBlank start).
        while !emu.cpu.bus.ppu.frame_ready {
            emu.step_instruction();
        }

        // ---- Render ----
        emu.cpu.bus.ppu.frame_ready = false;

        blit_rgba(&emu.cpu.bus.ppu.framebuffer, &mut rgba);

        texture
            .update(None, &rgba, W * 4)
            .map_err(|e| e.to_string())?;

        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();

        // vsync handles frame pacing, no manual sleep needed
    }

    Ok(())
}

/// Map SDL keycodes to joypad input
fn handle_key(emu: &mut Emulator, keycode: Keycode, pressed: bool) {
    // TODO: Wire this up to actual joypad register (0xFF00)
    // Common mappings:
    // Arrow keys -> D-pad
    // Z -> A button
    // X -> B button
    // Enter -> Start
    // Backspace/Shift -> Select
    match keycode {
        Keycode::Right => emu.on_key(0, pressed),
        Keycode::Left => emu.on_key(1, pressed),
        Keycode::Up => emu.on_key(2, pressed),
        Keycode::Down => emu.on_key(3, pressed),
        Keycode::Z => emu.on_key(4, pressed),      // A
        Keycode::X => emu.on_key(5, pressed),      // B
        Keycode::Return => emu.on_key(6, pressed), // Start
        Keycode::Backspace => emu.on_key(7, pressed), // Select
        _ => {}
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <rom_file>", args[0]);
        eprintln!("Example: {} game.gb", args[0]);
        std::process::exit(1);
    }

    let rom_path = &args[1];
    
    // Load ROM file
    let mut rom_file = File::open(rom_path)
        .unwrap_or_else(|e| {
            eprintln!("Failed to open ROM file '{}': {}", rom_path, e);
            std::process::exit(1);
        });
    
    let mut rom_data = Vec::new();
    rom_file.read_to_end(&mut rom_data)
        .unwrap_or_else(|e| {
            eprintln!("Failed to read ROM file: {}", e);
            std::process::exit(1);
        });

    println!("Loaded ROM: {} ({} bytes)", rom_path, rom_data.len());

    // Create emulator and load ROM
    let mut emu = Emulator::new();
    emu.load(&rom_data);

    // Run the emulator with SDL frontend
    if let Err(e) = run_sdl(emu) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
