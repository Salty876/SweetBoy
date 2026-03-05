# SweetBoy

A Game Boy (DMG) emulator written in Rust, featuring a clean separation between the platform-agnostic emulation core (`sweetboy_core`) and the native desktop frontend powered by [egui](https://github.com/emilk/egui).

![Rust](https://img.shields.io/badge/language-Rust-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

---

## Features

- **Cycle-accurate CPU** — all documented opcodes and CB-prefix instructions implemented
- **PPU** — background, window, and sprite rendering with proper scanline timing
- **MBC support** — MBC1 and MBC3 cartridge types (covers the majority of the Game Boy library)
- **Save states** — save and restore emulator state at any time via serde + bincode
- **Joypad input** — keyboard-mapped d-pad and buttons
- **Timer hardware** — DIV, TIMA, TMA, TAC with falling-edge detection
- **Interrupt handling** — VBlank, LCD STAT, Timer, Serial, and Joypad with correct priority
- **Fast-forward** — hold Space for 8× speed
- **Drag-and-drop** — drop a `.gb` file onto the window to load it
- **Native GUI** — menu bar, file dialogs, fullscreen toggle, and status bar via eframe

---

## Architecture

```
┌────────────────────────────────────────────────────────────────┐
│  desktop/            Native frontend (eframe / egui)           │
│  ├─ Menu bar (File · Emulation · View)                         │
│  ├─ Game Boy screen rendering (160×144, scaled)                │
│  ├─ Keyboard input mapping                                     │
│  └─ Save state file I/O                                        │
├────────────────────────────────────────────────────────────────┤
│  sweetboy_core/      Platform-agnostic emulation library       │
│  ├─ Emulator         Public API facade                         │
│  ├─ Cpu              Fetch-decode-execute, interrupts, CB ops  │
│  ├─ Bus              Memory map, MBC banking, timer, joypad    │
│  ├─ Ppu              Scanline renderer, OAM, VRAM              │
│  └─ error            Typed error enum                          │
└────────────────────────────────────────────────────────────────┘
```

### Core API

The `Emulator` struct in `sweetboy_core` exposes a minimal, frontend-independent API:

| Method | Description |
|---|---|
| `load_rom(&[u8])` | Load ROM bytes and detect MBC type |
| `step_frame()` | Run one full frame (70 224 T-cycles) |
| `framebuffer()` | Read the 160×144 shade-index framebuffer |
| `press_button(Button)` / `release_button(Button)` | Type-safe joypad input |
| `save_state()` → `Vec<u8>` | Serialize complete CPU+Bus+PPU state |
| `load_state(&[u8])` | Restore from a save-state blob |
| `reset()` | Reload the current ROM from scratch |

---

## Save State Design

Save states use **serde + bincode** for zero-copy binary serialization.

**Format:**

```
[ 8 bytes : ROM length (little-endian u64) ]
[ N bytes : bincode-serialized Cpu struct   ]
```

ROM data is excluded from the serialized state to keep files small. On load, the ROM length header is validated against the currently loaded ROM before restoring state.

All large arrays (`vram`, `oam`, `memory`, `framebuffer`) use the `serde-big-array` crate to work around serde's default array size limit.

---

## Controls

| Key | Game Boy Button |
|---|---|
| Arrow keys | D-Pad |
| Z | A |
| X | B |
| Enter | Start |
| Backspace | Select |
| Space (hold) | Fast-forward (8×) |

**Menu shortcuts:**

| Action | Access |
|---|---|
| Open ROM | File → Open ROM / drag-and-drop |
| Save State | File → Save State |
| Load State | File → Load State |
| Pause / Resume | Emulation → Pause / Resume |
| Reset | Emulation → Reset |
| Fullscreen | View → Toggle Fullscreen |

---

## Building & Running

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (1.85+ recommended for edition 2024)
- No external C libraries required — pure Rust dependencies

### Build

```bash
cargo build --release
```

### Run

```bash
# Launch the GUI (then use File → Open ROM or drag-and-drop)
cargo run --release -p desktop

# Or pass a ROM directly
cargo run --release -p desktop -- path/to/rom.gb
```

### Test

```bash
cargo test -p sweetboy_core
```

---

## Project Structure

```
.
├── Cargo.toml                 # Workspace root
├── sweetboy_core/             # Emulation core library
│   ├── src/
│   │   ├── lib.rs             # Public API: Emulator, Button, AudioSink
│   │   ├── error.rs           # EmulatorError enum
│   │   ├── cpu/
│   │   │   ├── mod.rs         # CPU struct, step(), interrupt service
│   │   │   ├── execute.rs     # Instruction execution (all opcodes)
│   │   │   ├── instructions.rs# Instruction enum + decode tables
│   │   │   └── registers.rs   # 8-bit registers, flags, 16-bit pairs
│   │   ├── bus/
│   │   │   └── mod.rs         # Memory bus, MBC1/MBC3, timer, joypad
│   │   └── ppu/
│   │       └── mod.rs         # PPU scanline renderer
│   └── tests/                 # Integration tests
├── desktop/                   # Native desktop frontend
│   └── src/
│       └── main.rs            # eframe app with menus, rendering, input
├── blaarg/                    # Blargg test ROMs
└── gb-test-roms/              # Additional test ROM suite
```

---

## Dependencies

### sweetboy_core

| Crate | Purpose |
|---|---|
| `serde` | Serialization framework for save states |
| `bincode` | Compact binary encoding of save-state data |
| `serde-big-array` | Serde support for arrays > 32 elements |
| `bitflags` | CPU flag register as a bitfield (with serde) |

### desktop

| Crate | Purpose |
|---|---|
| `eframe` | Cross-platform native GUI framework (egui) |
| `rfd` | Native file open/save dialogs |

---

## Performance

- **Frame timing:** 70 224 T-cycles per frame at 4 194 304 Hz ≈ 59.73 FPS
- **Target frame duration:** 16.74 ms
- **Rendering:** Pre-allocated RGBA buffer — no per-frame heap allocation
- **Fast-forward:** Runs 8 frames per repaint when Space is held

---

## Audio

An `AudioSink` trait is defined in `sweetboy_core` as a placeholder for future APU implementation. No audio is produced currently.

---

## Future Improvements

- APU (audio processing unit) with sample-accurate mixing
- Game Boy Color (CGB) support
- Serial link / multiplayer stub
- Configurable key bindings
- Per-game save RAM persistence (.sav files)
- WASM frontend for browser-based play
- Scanline / LCD shader effects
- Debugger overlay (registers, memory viewer, breakpoints)

---

## Acknowledgments

- [Pan Docs](https://gbdev.io/pandocs/) — comprehensive Game Boy technical reference
- [Blargg's test ROMs](https://github.com/retrio/gb-test-roms) — CPU instruction and timing tests
- [egui](https://github.com/emilk/egui) — immediate-mode GUI library for Rust
