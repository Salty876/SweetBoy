use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Transfer = 3,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Ppu {

    // Memory stuff
    #[serde(with = "BigArray")]
    pub vram: [u8; 0x2000], 
    #[serde(with = "BigArray")]
    pub oam: [u8; 0xA0],


    // ppu registers
    pub lcdc: u8,
    pub stat: u8,
    pub scy: u8,   // FF42
    pub scx: u8,   // FF43
    pub ly: u8,    // FF44 (read-only)
    pub lyc: u8,   // FF45
    pub dma: u8,   // FF46 (write triggers DMA)
    pub bgp: u8,   // FF47
    pub obp0: u8,  // FF48
    pub obp1: u8,  // FF49
    pub wy: u8,    // FF4A
    pub wx: u8,    // FF4B

    mode: PpuMode,
    mode_cycles: u16,
    /// Total cycles accumulated for frame timing (used when LCD is off)
    frame_cycles: u32,
    /// Window internal line counter - only increments when window is actually drawn
    window_line: u8,
    /// BG/window color IDs (0-3) for sprite priority - color 0 is "transparent" for priority
    #[serde(with = "BigArray")]
    bg_color_ids: [u8; 160],

    #[serde(with = "BigArray")]
    pub framebuffer: [u8; 160 * 144],
    pub frame_ready: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            lcdc: 0x91,
            stat: 0x80,
            scy: 0,
            scx: 0, 
            ly: 0,
            lyc: 0,
            dma: 0,
            bgp: 0xFC, // boot val
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
            mode: PpuMode::OamScan,
            mode_cycles: 0,
            frame_cycles: 0,
            window_line: 0,
            bg_color_ids: [0; 160],
            framebuffer: [0; 160*144],
            frame_ready: false,
        }
    }

    pub fn read_stat(&self) -> u8 {
        self.stat | 0x80
    }

    pub fn write_stat(&mut self, value:u8) {
        self.stat = (self.stat & 0x07) | (value & 0x78) | 0x80;
    }

    pub fn lcd_enabled(&self) -> bool {
        self.lcdc & 0x80 != 0
    }

    pub fn set_mode(&mut self, mode: PpuMode) {
        self.mode = mode;

        self.stat = (self.stat & !0x03) | (mode as u8 & 0x03);
    }

    pub fn update_coincidence(&mut self) -> bool {
        if self.ly == self.lyc {
            self.stat |= 0x04;
            true
        } else {
            self.stat &= !0x04;
            false
        }
    }

    pub fn step(&mut self, cycles: u16, req_interrupt: &mut dyn FnMut(u8)) {
        // Track total cycles for frame timing even when LCD is off
        self.frame_cycles = self.frame_cycles.wrapping_add(cycles as u32);
        const CYCLES_PER_FRAME: u32 = 70224;
        
        if !self.lcd_enabled() {
            self.ly = 0;
            self.mode_cycles = 0;
            self.set_mode(PpuMode::HBlank);
            self.update_coincidence();
            
            // Still generate frames at ~60fps when LCD is off
            if self.frame_cycles >= CYCLES_PER_FRAME {
                self.frame_cycles -= CYCLES_PER_FRAME;
                self.frame_ready = true;
                // Fill with white when LCD is off
                self.framebuffer.fill(0);
            }
            return;
        }

        self.mode_cycles = self.mode_cycles.wrapping_add(cycles);

        loop {
            match self.mode {
                PpuMode::OamScan => {
                    if self.mode_cycles < 80 { break; }
                    self.mode_cycles -= 80;
                    self.set_mode(PpuMode::Transfer);

                    if self.stat & 0x20 != 0 { req_interrupt(0x02); }
                }

                PpuMode::Transfer => {
                    if self.mode_cycles < 172 { break; }
                    self.mode_cycles -= 172;

                    // Render scanline: background, then window, then sprites
                    if self.ly < 144 {
                        self.render_bg_scanline(self.ly);
                        self.render_window_scanline(self.ly);
                        self.render_sprites_scanline(self.ly);
                    }

                    self.set_mode(PpuMode::HBlank);
                    // Statmode intereupt bit 3
                    if self.stat & 0x08 != 0 { req_interrupt(0x02); }
                }

                PpuMode::HBlank => {
                    if self.mode_cycles < 204 { break; }
                    self.mode_cycles -= 204;
                    self.ly = self.ly.wrapping_add(1);

                    let coincidence = self.update_coincidence();
                    if coincidence && (self.stat & 0x40 != 0) {
                        req_interrupt(0x02) //stat coincidence interupt
                    }

                    if self.ly == 144 {
                        self.set_mode(PpuMode::VBlank);
                        self.frame_ready = true;
                        req_interrupt(0x01); //Vblank int

                        //stat model
                        if self.stat & 0x10 != 0 { req_interrupt(0x02); }
                    } else {
                        self.set_mode(PpuMode::OamScan);

                        // stat mode 2
                        if self.stat & 0x20 != 0 { req_interrupt(0x02); }
                    }
                }


                PpuMode::VBlank => {
                    if self.mode_cycles < 456 {break;}
                    self.mode_cycles -= 456;
                    self.ly = self.ly.wrapping_add(1);

                    let coincidence = self.update_coincidence();
                    if coincidence && (self.stat & 0x40 != 0) {
                        req_interrupt(0x02);
                    }

                    if self.ly > 153 {
                        self.ly = 0;
                        self.window_line = 0; // Reset window line counter at frame start
                        self.update_coincidence();
                        self.set_mode(PpuMode::OamScan);
                        if self.stat & 0x20 != 0 { req_interrupt(0x02); }
                    }
                }
            }
        }
    }

    fn render_bg_scanline(&mut self, ly:u8) {

        if self.lcdc & 0x01 == 0 {
            let row = ly as usize * 160;
            for x in 0..160 {
                self.framebuffer[row + x] = self.bgp & 0x03;
                self.bg_color_ids[x] = 0; // Color 0 when BG disabled
            }
            return;
        }

        let tilemap_base = if self.lcdc & 0x08 != 0 { 0x9c00 } else { 0x9800 };
        let signed_mdoe = self.lcdc & 0x10 == 0; // 0 => 0x8800 

        let scy = self.scy;
        let scx = self.scx;

        let y = ly.wrapping_add(scy);
        let tile_row = (y / 8) as u16;
        let line_in_tile = (y % 8) as u16;

        let row_out = ly as usize * 160;

        for x in 0..160u16 {
            let px = (x as u8).wrapping_add(scx);
            let tile_col = (px / 8) as u16;
            let bit_in_tile = 7 - (px % 8);

            let tile_index = tile_row * 32 + tile_col;
            let tile_id = self.vram_read(tilemap_base + tile_index);

            let tile_addr = if signed_mdoe {
                let id = tile_id as i8 as i16;
                (0x9000i32 + (id as i32) * 16) as u16
            } else {
                0x8000 + (tile_id as u16) * 16
            };

            let addr = tile_addr + line_in_tile * 2;
            let lo = self.vram_read(addr);
            let hi = self.vram_read(addr + 1);

            let lo_bit = (lo >> bit_in_tile) & 1;
            let hi_bit = (hi >> bit_in_tile) & 1;
            let color_id = (hi_bit << 1) | lo_bit;

            let shade = (self.bgp >> (color_id * 2)) & 0x03;
            self.framebuffer[row_out + x as usize] = shade;
            self.bg_color_ids[x as usize] = color_id;
        }
    }

    pub fn vram_read(&self, addr: u16) -> u8{
        let idx = (addr - 0x8000) as usize;
        self.vram[idx]
    }

    /// Render the window layer for one scanline
    fn render_window_scanline(&mut self, ly: u8) {
        // Window enable (bit 5) and LCD/BG enable (bit 0)
        if self.lcdc & 0x20 == 0 || self.lcdc & 0x01 == 0 {
            return;
        }

        // Window is only visible if WY <= LY and WX <= 166
        if ly < self.wy || self.wx > 166 {
            return;
        }

        // Use internal window line counter
        let window_line = self.window_line;
        let tilemap_base = if self.lcdc & 0x40 != 0 { 0x9C00 } else { 0x9800 };
        let signed_mode = self.lcdc & 0x10 == 0;

        let tile_row = (window_line / 8) as u16;
        let line_in_tile = (window_line % 8) as u16;

        let screen_x_start = if self.wx < 7 { 0 } else { self.wx - 7 };
        let row_out = ly as usize * 160;

        for screen_x in screen_x_start..160 {
            let window_x = screen_x - screen_x_start;
            let tile_col = (window_x / 8) as u16;
            let bit_in_tile = 7 - (window_x % 8);

            let tile_index = tile_row * 32 + tile_col;
            let tile_id = self.vram_read(tilemap_base + tile_index);

            let tile_addr = if signed_mode {
                let id = tile_id as i8 as i16;
                (0x9000i32 + (id as i32) * 16) as u16
            } else {
                0x8000 + (tile_id as u16) * 16
            };

            let addr = tile_addr + line_in_tile * 2;
            let lo = self.vram_read(addr);
            let hi = self.vram_read(addr + 1);

            let lo_bit = (lo >> bit_in_tile) & 1;
            let hi_bit = (hi >> bit_in_tile) & 1;
            let color_id = (hi_bit << 1) | lo_bit;

            let shade = (self.bgp >> (color_id * 2)) & 0x03;
            self.framebuffer[row_out + screen_x as usize] = shade;
            self.bg_color_ids[screen_x as usize] = color_id; // Window overwrites BG color ID
        }
        
        // Increment window line counter since we drew a window line
        self.window_line = self.window_line.wrapping_add(1);
    }

    /// Render sprites (OBJ) for one scanline
    fn render_sprites_scanline(&mut self, ly: u8) {
        // Sprite enable (LCDC bit 1)
        if self.lcdc & 0x02 == 0 {
            return;
        }

        let sprite_height: u8 = if self.lcdc & 0x04 != 0 { 16 } else { 8 };
        let row_out = ly as usize * 160;

        // Collect sprites visible on this scanline (max 10 per line on real hardware)
        let mut sprites_on_line: Vec<(u8, u8, u8, u8, usize)> = Vec::with_capacity(10);
        
        for i in 0..40 {
            let oam_addr = i * 4;
            let sprite_y = self.oam[oam_addr];
            let sprite_x = self.oam[oam_addr + 1];
            let tile_id = self.oam[oam_addr + 2];
            let attrs = self.oam[oam_addr + 3];

            // Sprite Y is offset by 16 in OAM
            // Use wrapping subtraction to handle sprites partially above screen
            let line_in_sprite = ly.wrapping_add(16).wrapping_sub(sprite_y);
            
            // Check if sprite intersects this scanline
            if line_in_sprite < sprite_height {
                sprites_on_line.push((sprite_x, sprite_y, tile_id, attrs, i));
                if sprites_on_line.len() >= 10 {
                    break;
                }
            }
        }

        // Sort by X coordinate (lower X = higher priority), then by OAM index
        sprites_on_line.sort_by(|a, b| {
            if a.0 == b.0 {
                a.4.cmp(&b.4)
            } else {
                a.0.cmp(&b.0)
            }
        });

        // Render in reverse order (lower priority first, so higher priority overwrites)
        for (sprite_x, sprite_y, mut tile_id, attrs, _) in sprites_on_line.into_iter().rev() {
            let flip_x = attrs & 0x20 != 0;
            let flip_y = attrs & 0x40 != 0;
            let bg_priority = attrs & 0x80 != 0;
            let palette = if attrs & 0x10 != 0 { self.obp1 } else { self.obp0 };

            // Calculate which line of the sprite we're on (using wrapping to handle Y offset)
            let mut line_in_sprite = ly.wrapping_add(16).wrapping_sub(sprite_y);
            
            // For 8x16 sprites, mask out bit 0 of tile ID
            if sprite_height == 16 {
                tile_id &= 0xFE;
            }

            if flip_y {
                line_in_sprite = sprite_height - 1 - line_in_sprite;
            }

            // Get tile data address (sprites always use 0x8000 addressing)
            let tile_addr = 0x8000u16 + (tile_id as u16) * 16 + (line_in_sprite as u16) * 2;
            let lo = self.vram_read(tile_addr);
            let hi = self.vram_read(tile_addr + 1);

            // Render 8 pixels
            for px in 0..8u8 {
                let screen_x = sprite_x.wrapping_sub(8).wrapping_add(px);
                if screen_x >= 160 {
                    continue;
                }

                let bit = if flip_x { px } else { 7 - px };
                let lo_bit = (lo >> bit) & 1;
                let hi_bit = (hi >> bit) & 1;
                let color_id = (hi_bit << 1) | lo_bit;

                // Color 0 is transparent for sprites
                if color_id == 0 {
                    continue;
                }

                // BG priority: if set and BG/window color ID is non-zero, sprite is behind
                if bg_priority {
                    let bg_color_id = self.bg_color_ids[screen_x as usize];
                    if bg_color_id != 0 {
                        continue;
                    }
                }

                let shade = (palette >> (color_id * 2)) & 0x03;
                self.framebuffer[row_out + screen_x as usize] = shade;
            }
        }
    }

}