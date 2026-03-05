pub mod bus;
pub mod cpu;
pub mod ppu;
pub mod error;

pub use cpu::Cpu;
pub use bus::Bus;
pub use ppu::Ppu;
pub use error::EmulatorError;

/// Game Boy button identifiers for type-safe input.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Button {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Start,
    Select,
}

/// Trait for future audio output.
/// Implement this and pass to [`Emulator::set_audio_sink`] when APU is ready.
pub trait AudioSink {
    fn queue_samples(&mut self, samples: &[f32]);
}

/// Main emulator struct that ties CPU, Bus, and PPU together.
///
/// Provides a clean, platform-agnostic API for any frontend (desktop, WASM, etc.).
pub struct Emulator {
    pub cpu: Cpu,
    /// Copy of loaded ROM for reset support.
    rom_data: Vec<u8>,
}

impl Emulator {
    /// Create a new emulator instance with default state.
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            rom_data: Vec::new(),
        }
    }

    /// Load ROM data and detect cartridge type for MBC support.
    pub fn load_rom(&mut self, data: &[u8]) -> Result<(), EmulatorError> {
        if data.len() < 0x150 {
            return Err(EmulatorError::InvalidRom(
                format!("ROM too small: {} bytes (minimum 336)", data.len()),
            ));
        }
        self.rom_data = data.to_vec();
        self.cpu = Cpu::new();
        self.cpu.bus.load_rom(data);
        Ok(())
    }

    /// Returns true if a ROM is currently loaded.
    pub fn rom_loaded(&self) -> bool {
        !self.rom_data.is_empty()
    }

    /// Run emulation for exactly one frame (until PPU signals VBlank).
    pub fn step_frame(&mut self) {
        // Cap iterations to avoid infinite loop if PPU is stuck
        let max_cycles = 70224 * 2;
        let mut cycles = 0u32;
        while !self.cpu.bus.ppu.frame_ready && cycles < max_cycles {
            self.cpu.step();
            cycles += self.cpu.last_cycle_timestamp as u32;
        }
        self.cpu.bus.ppu.frame_ready = false;
    }

    /// Execute one CPU instruction and advance emulation.
    pub fn step_instruction(&mut self) {
        self.cpu.step();
    }

    /// Access the current framebuffer (160x144 shade indices, 0-3).
    pub fn framebuffer(&self) -> &[u8; 160 * 144] {
        &self.cpu.bus.ppu.framebuffer
    }

    /// Returns true if PPU has a complete frame ready.
    pub fn frame_ready(&self) -> bool {
        self.cpu.bus.ppu.frame_ready
    }

    /// Clear the frame-ready flag after consuming the frame.
    pub fn clear_frame_ready(&mut self) {
        self.cpu.bus.ppu.frame_ready = false;
    }

    /// Register a button press (active-low joypad).
    pub fn press_button(&mut self, button: Button) {
        self.set_button(button, true);
    }

    /// Register a button release (active-low joypad).
    pub fn release_button(&mut self, button: Button) {
        self.set_button(button, false);
    }

    fn set_button(&mut self, button: Button, pressed: bool) {
        let (is_dpad, bit) = match button {
            Button::Right  => (true,  0),
            Button::Left   => (true,  1),
            Button::Up     => (true,  2),
            Button::Down   => (true,  3),
            Button::A      => (false, 0),
            Button::B      => (false, 1),
            Button::Select => (false, 2),
            Button::Start  => (false, 3),
        };
        let mask = 1u8 << bit;
        let reg = if is_dpad {
            &mut self.cpu.bus.joypad_dpad
        } else {
            &mut self.cpu.bus.joypad_buttons
        };
        if pressed {
            *reg &= !mask; // Clear bit = pressed (active-low)
        } else {
            *reg |= mask;  // Set bit = released
        }
    }

    /// Serialize the full emulator state to bytes.
    /// ROM data is NOT included — only a length check is stored.
    pub fn save_state(&self) -> Result<Vec<u8>, EmulatorError> {
        let rom_len = self.rom_data.len() as u64;
        let cpu_bytes = bincode::serialize(&self.cpu)
            .map_err(|e| EmulatorError::SaveStateFailed(e.to_string()))?;

        // Format: [8 bytes rom_len] [cpu state...]
        let mut out = Vec::with_capacity(8 + cpu_bytes.len());
        out.extend_from_slice(&rom_len.to_le_bytes());
        out.extend_from_slice(&cpu_bytes);
        Ok(out)
    }

    /// Restore emulator state from bytes previously created by [`save_state`].
    /// The same ROM must already be loaded.
    pub fn load_state(&mut self, data: &[u8]) -> Result<(), EmulatorError> {
        if data.len() < 8 {
            return Err(EmulatorError::LoadStateFailed("Data too short".into()));
        }
        let rom_len = u64::from_le_bytes(
            data[..8].try_into().unwrap(),
        );
        if rom_len != self.rom_data.len() as u64 {
            return Err(EmulatorError::LoadStateFailed(format!(
                "ROM size mismatch: state expects {} bytes, loaded ROM is {} bytes",
                rom_len,
                self.rom_data.len(),
            )));
        }
        let cpu: Cpu = bincode::deserialize(&data[8..])
            .map_err(|e| EmulatorError::LoadStateFailed(e.to_string()))?;
        self.cpu = cpu;
        Ok(())
    }

    /// Reset emulation to initial state with the current ROM.
    pub fn reset(&mut self) {
        if !self.rom_data.is_empty() {
            let rom = self.rom_data.clone();
            self.cpu = Cpu::new();
            self.cpu.bus.load_rom(&rom);
        }
    }

    /// Convenience accessor for the Bus.
    pub fn bus(&self) -> &Bus {
        &self.cpu.bus
    }

    /// Mutable accessor for the Bus.
    pub fn bus_mut(&mut self) -> &mut Bus {
        &mut self.cpu.bus
    }

    /// Legacy key handler — prefer [`press_button`] / [`release_button`].
    pub fn on_key(&mut self, keycode: u8, pressed: bool) {
        let bit = keycode & 0x03;
        let mask = 1u8 << bit;
        if keycode < 4 {
            if pressed {
                self.cpu.bus.joypad_dpad &= !mask;
            } else {
                self.cpu.bus.joypad_dpad |= mask;
            }
        } else {
            if pressed {
                self.cpu.bus.joypad_buttons &= !mask;
            } else {
                self.cpu.bus.joypad_buttons |= mask;
            }
        }
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}
