use eframe::egui;
use std::path::PathBuf;
use std::time::Duration;

use sweetboy_core::{Button, Emulator};

const W: usize = 160;
const H: usize = 144;
const FRAME_DURATION: Duration = Duration::from_nanos(16_742_706); // ~59.73 fps
const FAST_FORWARD_MULTIPLIER: u32 = 8;

/// Map DMG shade index (0..3) to grayscale byte.
fn shade_to_rgba(shade: u8) -> [u8; 4] {
    let v = match shade & 0x03 {
        0 => 0xFF,
        1 => 0xAA,
        2 => 0x55,
        _ => 0x00,
    };
    [v, v, v, 0xFF]
}

/// Convert shade-index framebuffer into RGBA pixels (no allocation).
fn blit_rgba(ppu_fb: &[u8; W * H], out: &mut [u8; W * H * 4]) {
    for i in 0..(W * H) {
        let rgba = shade_to_rgba(ppu_fb[i]);
        let o = i * 4;
        out[o] = rgba[0];
        out[o + 1] = rgba[1];
        out[o + 2] = rgba[2];
        out[o + 3] = rgba[3];
    }
}

struct SweetBoyApp {
    emu: Emulator,
    /// The egui texture for the Game Boy screen.
    screen_texture: Option<egui::TextureHandle>,
    /// Pre-allocated RGBA pixel buffer (avoids per-frame allocation).
    rgba_buffer: Box<[u8; W * H * 4]>,
    /// Whether a ROM is loaded.
    rom_loaded: bool,
    /// Path of the currently loaded ROM (for save state naming).
    rom_path: Option<PathBuf>,
    /// Emulation paused.
    paused: bool,
    /// Fast-forward while held.
    fast_forward: bool,
    /// Status message shown in the bottom bar.
    status_msg: String,
}

impl SweetBoyApp {
    fn new() -> Self {
        Self {
            emu: Emulator::new(),
            screen_texture: None,
            rgba_buffer: Box::new([0u8; W * H * 4]),
            rom_loaded: false,
            rom_path: None,
            paused: false,
            fast_forward: false,
            status_msg: String::from("No ROM loaded"),
        }
    }

    fn open_rom_dialog(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("Game Boy ROM", &["gb", "gbc"])
            .pick_file();

        if let Some(path) = file {
            self.load_rom_from_path(path);
        }
    }

    fn load_rom_from_path(&mut self, path: PathBuf) {
        match std::fs::read(&path) {
            Ok(data) => match self.emu.load_rom(&data) {
                Ok(()) => {
                    self.rom_loaded = true;
                    self.paused = false;
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    self.status_msg = format!("Loaded: {}", name);
                    self.rom_path = Some(path);
                }
                Err(e) => {
                    self.status_msg = format!("ROM error: {}", e);
                }
            },
            Err(e) => {
                self.status_msg = format!("File error: {}", e);
            }
        }
    }

    fn save_state_path(&self) -> Option<PathBuf> {
        self.rom_path
            .as_ref()
            .map(|p| p.with_extension("state"))
    }

    fn do_save_state(&mut self) {
        let path = match self.save_state_path() {
            Some(p) => p,
            None => return,
        };
        match self.emu.save_state() {
            Ok(data) => match std::fs::write(&path, &data) {
                Ok(()) => {
                    self.status_msg = format!("State saved to {}", path.display());
                }
                Err(e) => {
                    self.status_msg = format!("Save failed: {}", e);
                }
            },
            Err(e) => {
                self.status_msg = format!("Save failed: {}", e);
            }
        }
    }

    fn do_load_state(&mut self) {
        let path = match self.save_state_path() {
            Some(p) => p,
            None => return,
        };
        match std::fs::read(&path) {
            Ok(data) => match self.emu.load_state(&data) {
                Ok(()) => {
                    self.status_msg = format!("State loaded from {}", path.display());
                }
                Err(e) => {
                    self.status_msg = format!("Load failed: {}", e);
                }
            },
            Err(e) => {
                self.status_msg = format!("Load failed: {}", e);
            }
        }
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // Fast-forward while Space is held
            self.fast_forward = i.key_down(egui::Key::Space);

            // Map keyboard to Game Boy buttons
            let mappings: &[(egui::Key, Button)] = &[
                (egui::Key::ArrowRight, Button::Right),
                (egui::Key::ArrowLeft, Button::Left),
                (egui::Key::ArrowUp, Button::Up),
                (egui::Key::ArrowDown, Button::Down),
                (egui::Key::Z, Button::A),
                (egui::Key::X, Button::B),
                (egui::Key::Enter, Button::Start),
                (egui::Key::Backspace, Button::Select),
            ];

            for &(key, button) in mappings {
                if i.key_pressed(key) {
                    self.emu.press_button(button);
                }
                if i.key_released(key) {
                    self.emu.release_button(button);
                }
            }
        });
    }

    fn step_emulation(&mut self) {
        if !self.rom_loaded || self.paused {
            return;
        }
        let frames = if self.fast_forward {
            FAST_FORWARD_MULTIPLIER
        } else {
            1
        };
        for _ in 0..frames {
            self.emu.step_frame();
        }
    }

    fn upload_framebuffer(&mut self, ctx: &egui::Context) {
        blit_rgba(self.emu.framebuffer(), &mut self.rgba_buffer);

        let image = egui::ColorImage::from_rgba_unmultiplied(
            [W, H],
            &*self.rgba_buffer,
        );

        match &mut self.screen_texture {
            Some(tex) => {
                tex.set(image, egui::TextureOptions::NEAREST);
            }
            None => {
                self.screen_texture = Some(ctx.load_texture(
                    "gb_screen",
                    image,
                    egui::TextureOptions::NEAREST,
                ));
            }
        }
    }
}

impl eframe::App for SweetBoyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle drag-and-drop
        let dropped: Vec<_> = ctx.input(|i| {
            i.raw.dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        for path in dropped {
            if path
                .extension()
                .is_some_and(|ext| ext == "gb" || ext == "gbc")
            {
                self.load_rom_from_path(path);
                break;
            }
        }

        // Input
        self.handle_input(ctx);

        // Emulation
        self.step_emulation();

        // Upload framebuffer texture
        if self.rom_loaded {
            self.upload_framebuffer(ctx);
        }

        // ── Menu bar ──
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open ROM…").clicked() {
                        ui.close_menu();
                        self.open_rom_dialog();
                    }
                    ui.separator();
                    if ui
                        .add_enabled(self.rom_loaded, egui::Button::new("Save State"))
                        .clicked()
                    {
                        ui.close_menu();
                        self.do_save_state();
                    }
                    if ui
                        .add_enabled(self.rom_loaded, egui::Button::new("Load State"))
                        .clicked()
                    {
                        ui.close_menu();
                        self.do_load_state();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Emulation", |ui| {
                    let pause_label = if self.paused { "Resume" } else { "Pause" };
                    if ui
                        .add_enabled(self.rom_loaded, egui::Button::new(pause_label))
                        .clicked()
                    {
                        self.paused = !self.paused;
                        ui.close_menu();
                    }
                    if ui
                        .add_enabled(self.rom_loaded, egui::Button::new("Reset"))
                        .clicked()
                    {
                        self.emu.reset();
                        self.paused = false;
                        self.status_msg = "Emulator reset".into();
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Toggle Fullscreen").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                            !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)),
                        ));
                        ui.close_menu();
                    }
                });
            });
        });

        // ── Status bar ──
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_msg);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.rom_loaded {
                        if self.paused {
                            ui.label("⏸ Paused");
                        } else if self.fast_forward {
                            ui.label("⏩ Fast Forward");
                        } else {
                            ui.label("▶ Running");
                        }
                    }
                });
            });
        });

        // ── Central panel ──
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.rom_loaded {
                // Centered "Open ROM" splash
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 3.0);
                    ui.heading("SweetBoy");
                    ui.label("A Game Boy emulator");
                    ui.add_space(16.0);
                    if ui.button("Open ROM…").clicked() {
                        self.open_rom_dialog();
                    }
                    ui.add_space(8.0);
                    ui.label("or drag-and-drop a .gb file");
                });
            } else if let Some(tex) = &self.screen_texture {
                // Render the Game Boy screen, fit to panel maintaining aspect ratio
                let available = ui.available_size();
                let aspect = W as f32 / H as f32;
                let (w, h) = if available.x / available.y > aspect {
                    (available.y * aspect, available.y)
                } else {
                    (available.x, available.x / aspect)
                };
                let size = egui::vec2(w, h);
                ui.centered_and_justified(|ui| {
                    ui.image(egui::load::SizedTexture::new(tex.id(), size));
                });
            }
        });

        // Request repaint at ~59.73 Hz (or immediate for fast-forward)
        if self.rom_loaded && !self.paused {
            if self.fast_forward {
                ctx.request_repaint();
            } else {
                ctx.request_repaint_after(FRAME_DURATION);
            }
        }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 576.0 + 48.0])
            .with_min_inner_size([320.0, 288.0 + 48.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "SweetBoy",
        options,
        Box::new(|_cc| Ok(Box::new(SweetBoyApp::new()))),
    )
}
