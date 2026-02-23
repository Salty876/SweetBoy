pub mod bus;
pub mod cpu;
pub mod ppu;

pub use cpu::Cpu;
pub use bus::Bus;
pub use ppu::Ppu;

/// Main emulator struct that ties CPU, Bus, and PPU together.
pub struct Emulator {
    pub cpu: Cpu,
}

impl Emulator {
    /// Create a new emulator instance with default state.
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
        }
    }

    /// Load ROM data and detect cartridge type for MBC support.
    pub fn load(&mut self, data: &[u8]) {
        self.cpu.bus.load_rom(data);
    }

    /// Execute one CPU instruction and advance emulation.
    pub fn step_instruction(&mut self) {
        self.cpu.step();
    }

    /// Convenience accessor for the Bus.
    pub fn bus(&self) -> &Bus {
        &self.cpu.bus
    }

    /// Mutable accessor for the Bus (needed for frame_ready flag, etc.)
    pub fn bus_mut(&mut self) -> &mut Bus {
        &mut self.cpu.bus
    }

    /// Handle key press/release for joypad input.
    /// keycode 0-3: D-pad (Right, Left, Up, Down)
    /// keycode 4-7: Buttons (A, B, Start, Select)
    /// Joypad bits are active LOW (0 = pressed, 1 = released)
    pub fn on_key(&mut self, keycode: u8, pressed: bool) {
        let bit = keycode & 0x03; // bit position within the register
        let mask = 1u8 << bit;
        
        if keycode < 4 {
            // D-pad
            if pressed {
                self.cpu.bus.joypad_dpad &= !mask; // Clear bit = pressed
            } else {
                self.cpu.bus.joypad_dpad |= mask;  // Set bit = released
            }
        } else {
            // Buttons (A, B, Start, Select)
            if pressed {
                self.cpu.bus.joypad_buttons &= !mask; // Clear bit = pressed
            } else {
                self.cpu.bus.joypad_buttons |= mask;  // Set bit = released
            }
        }
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}
