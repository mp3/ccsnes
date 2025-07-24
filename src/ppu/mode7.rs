use crate::ppu::memory::{Vram, Cgram};
use crate::ppu::registers::PpuRegisters;

/// Mode 7 transformation matrix and rendering
pub struct Mode7Renderer {
    // Mode 7 matrix parameters (16-bit signed)
    pub m7a: i16,  // Matrix A (cosine)
    pub m7b: i16,  // Matrix B (sine)
    pub m7c: i16,  // Matrix C (-sine)
    pub m7d: i16,  // Matrix D (cosine)
    
    // Center coordinates
    pub m7x: i16,  // Center X
    pub m7y: i16,  // Center Y
    
    // Scroll values
    pub m7hofs: i16,  // Horizontal scroll
    pub m7vofs: i16,  // Vertical scroll
    
    // Internal state
    write_toggle: bool,
    prev_value: u8,
}

impl Mode7Renderer {
    pub fn new() -> Self {
        Self {
            m7a: 0x0100,  // Default to identity matrix
            m7b: 0,
            m7c: 0,
            m7d: 0x0100,
            m7x: 0,
            m7y: 0,
            m7hofs: 0,
            m7vofs: 0,
            write_toggle: false,
            prev_value: 0,
        }
    }
    
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0x211B => {
                // M7A - Mode 7 Matrix A (16-bit write)
                if !self.write_toggle {
                    self.m7a = (self.m7a & 0xFF00u16 as i16) | value as i16;
                } else {
                    self.m7a = (self.m7a & 0x00FF) | ((value as i16) << 8);
                }
                self.write_toggle = !self.write_toggle;
            }
            0x211C => {
                // M7B - Mode 7 Matrix B (16-bit write)
                if !self.write_toggle {
                    self.m7b = (self.m7b & 0xFF00u16 as i16) | value as i16;
                } else {
                    self.m7b = (self.m7b & 0x00FF) | ((value as i16) << 8);
                }
                self.write_toggle = !self.write_toggle;
            }
            0x211D => {
                // M7C - Mode 7 Matrix C (16-bit write)
                if !self.write_toggle {
                    self.m7c = (self.m7c & 0xFF00u16 as i16) | value as i16;
                } else {
                    self.m7c = (self.m7c & 0x00FF) | ((value as i16) << 8);
                }
                self.write_toggle = !self.write_toggle;
            }
            0x211E => {
                // M7D - Mode 7 Matrix D (16-bit write)
                if !self.write_toggle {
                    self.m7d = (self.m7d & 0xFF00u16 as i16) | value as i16;
                } else {
                    self.m7d = (self.m7d & 0x00FF) | ((value as i16) << 8);
                }
                self.write_toggle = !self.write_toggle;
            }
            0x211F => {
                // M7X - Mode 7 Center X (13-bit write)
                if !self.write_toggle {
                    self.m7x = (self.m7x & 0xFF00u16 as i16) | value as i16;
                } else {
                    // For 13-bit signed value, only lower 5 bits are used from the high byte
                    let high_5_bits = value & 0x1F;
                    
                    // Create the 13-bit value (bits 0-12)
                    let low_byte = self.m7x as u16 & 0x00FF;
                    let value_13bit = ((high_5_bits as u16) << 8) | low_byte;
                    
                    // Sign extend from bit 12 (0x1000) to make it a proper 16-bit signed value
                    let result = if value_13bit & 0x1000 != 0 {
                        // Negative number: For 13-bit signed to 16-bit signed conversion
                        // We need to set bits 13-15 to 1 and keep bits 0-11 unchanged
                        // Mask off bit 12 and above, then OR with sign extension
                        ((value_13bit & 0x0FFF) | 0xE000) as i16
                    } else {
                        // Positive number: bits 13-15 remain 0
                        value_13bit as i16
                    };
                    
                    self.m7x = result;
                }
                self.write_toggle = !self.write_toggle;
                
                // Also write to M7HOFS
                self.m7hofs = self.m7x;
            }
            0x2120 => {
                // M7Y - Mode 7 Center Y (13-bit write)
                if !self.write_toggle {
                    self.m7y = (self.m7y & 0xFF00u16 as i16) | value as i16;
                } else {
                    // For 13-bit signed value, only lower 5 bits are used from the high byte
                    let high_5_bits = value & 0x1F;
                    
                    // Create the 13-bit value (bits 0-12)
                    let low_byte = self.m7y as u16 & 0x00FF;
                    let value_13bit = ((high_5_bits as u16) << 8) | low_byte;
                    
                    // Sign extend from bit 12 (0x1000) to make it a proper 16-bit signed value
                    let result = if value_13bit & 0x1000 != 0 {
                        // Negative number: For 13-bit signed to 16-bit signed conversion
                        // We need to set bits 13-15 to 1 and keep bits 0-11 unchanged
                        // Mask off bit 12 and above, then OR with sign extension
                        ((value_13bit & 0x0FFF) | 0xE000) as i16
                    } else {
                        // Positive number: bits 13-15 remain 0
                        value_13bit as i16
                    };
                    
                    self.m7y = result;
                }
                self.write_toggle = !self.write_toggle;
                
                // Also write to M7VOFS
                self.m7vofs = self.m7y;
            }
            _ => {}
        }
    }
    
    /// Render a Mode 7 scanline
    pub fn render_scanline(
        &self,
        vram: &Vram,
        cgram: &Cgram,
        registers: &PpuRegisters,
        scanline: u16,
        buffer: &mut [u8],
    ) {
        // Mode 7 uses a 128x128 tilemap at VRAM $0000-$3FFF
        // Tiles are 8x8 pixels, direct color (8bpp) at VRAM $0000-$3FFF
        
        let screen_y = scanline as i32;
        
        for screen_x in 0..256 {
            // Apply transformation matrix
            // Transform screen coordinates to texture coordinates
            let sx = screen_x as i32 - 128;  // Center around screen center
            let sy = screen_y as i32 - 112;
            
            // Apply matrix transformation
            // [tx]   [m7a m7b] [sx]   [m7x]
            // [ty] = [m7c m7d] [sy] + [m7y]
            let tx = ((self.m7a as i32 * sx + self.m7b as i32 * sy) >> 8) + self.m7hofs as i32;
            let ty = ((self.m7c as i32 * sx + self.m7d as i32 * sy) >> 8) + self.m7vofs as i32;
            
            // Get pixel from tilemap
            let pixel = self.get_mode7_pixel(vram, tx, ty, registers);
            
            // Convert to RGB
            let (r, g, b) = if pixel != 0 {
                cgram.color_to_rgb(cgram.read_color(pixel))
            } else {
                (0, 0, 0)  // Transparent
            };
            
            // Write to buffer
            let offset = screen_x * 4;
            buffer[offset] = r;
            buffer[offset + 1] = g;
            buffer[offset + 2] = b;
            buffer[offset + 3] = if pixel != 0 { 255 } else { 0 };
        }
    }
    
    fn get_mode7_pixel(&self, vram: &Vram, tx: i32, ty: i32, registers: &PpuRegisters) -> u8 {
        // Handle wrapping/repeat modes
        let (tile_x, tile_y, out_of_bounds) = self.handle_mode7_wrapping(tx, ty, registers);
        
        if out_of_bounds {
            // Return transparent or fixed color based on settings
            return if registers.m7sel & 0x40 != 0 {
                // Fill with character 0
                0
            } else {
                // Transparent
                0
            };
        }
        
        // Get tile number from tilemap
        let tilemap_x = (tile_x / 8) & 0x7F;
        let tilemap_y = (tile_y / 8) & 0x7F;
        let tilemap_addr = (tilemap_y * 128 + tilemap_x) * 2;
        let tile_num = vram.read(tilemap_addr as u16) as u16;
        
        // Get pixel within tile
        let pixel_x = tile_x & 7;
        let pixel_y = tile_y & 7;
        
        // Mode 7 tiles are 8x8, 8bpp (64 bytes per tile)
        let tile_addr = tile_num * 64 + (pixel_y * 8 + pixel_x) as u16;
        vram.read(tile_addr)
    }
    
    fn handle_mode7_wrapping(&self, tx: i32, ty: i32, registers: &PpuRegisters) -> (i32, i32, bool) {
        let repeat_mode = registers.m7sel & 0x03;
        
        match repeat_mode {
            0 => {
                // Screen repeat
                let wrapped_x = tx & 0x3FF;  // Wrap at 1024
                let wrapped_y = ty & 0x3FF;
                (wrapped_x, wrapped_y, false)
            }
            1 => {
                // Affine texture wrapping (same as screen repeat for now)
                let wrapped_x = tx & 0x3FF;
                let wrapped_y = ty & 0x3FF;
                (wrapped_x, wrapped_y, false)
            }
            2 => {
                // Bitmap repeat
                if tx < 0 || tx >= 1024 || ty < 0 || ty >= 1024 {
                    (0, 0, true)  // Out of bounds
                } else {
                    (tx, ty, false)
                }
            }
            3 => {
                // Bitmap repeat with fill
                if tx < 0 || tx >= 1024 || ty < 0 || ty >= 1024 {
                    (0, 0, true)  // Out of bounds, will be filled
                } else {
                    (tx, ty, false)
                }
            }
            _ => (tx, ty, false),
        }
    }
    
    /// Check if Mode 7 EXTBG is enabled (for BG2 in Mode 7)
    pub fn is_extbg_enabled(&self, registers: &PpuRegisters) -> bool {
        registers.setini & 0x40 != 0
    }
    
    /// Render Mode 7 EXTBG (BG2)
    pub fn render_extbg_scanline(
        &self,
        vram: &Vram,
        cgram: &Cgram,
        registers: &PpuRegisters,
        scanline: u16,
        buffer: &mut [u8],
    ) {
        // EXTBG uses the high bit of Mode 7 tiles as priority
        // Same rendering as Mode 7 but with priority handling
        
        let screen_y = scanline as i32;
        
        for screen_x in 0..256 {
            let sx = screen_x as i32 - 128;
            let sy = screen_y as i32 - 112;
            
            let tx = ((self.m7a as i32 * sx + self.m7b as i32 * sy) >> 8) + self.m7hofs as i32;
            let ty = ((self.m7c as i32 * sx + self.m7d as i32 * sy) >> 8) + self.m7vofs as i32;
            
            // Get pixel and priority from tilemap
            let (pixel, priority) = self.get_mode7_pixel_with_priority(vram, tx, ty, registers);
            
            if priority {
                // High priority pixels for EXTBG
                let (r, g, b) = if pixel != 0 {
                    cgram.color_to_rgb(cgram.read_color(pixel))
                } else {
                    (0, 0, 0)
                };
                
                let offset = screen_x * 4;
                buffer[offset] = r;
                buffer[offset + 1] = g;
                buffer[offset + 2] = b;
                buffer[offset + 3] = if pixel != 0 { 255 } else { 0 };
            }
        }
    }
    
    fn get_mode7_pixel_with_priority(
        &self,
        vram: &Vram,
        tx: i32,
        ty: i32,
        registers: &PpuRegisters,
    ) -> (u8, bool) {
        let (tile_x, tile_y, out_of_bounds) = self.handle_mode7_wrapping(tx, ty, registers);
        
        if out_of_bounds {
            return (0, false);
        }
        
        let tilemap_x = (tile_x / 8) & 0x7F;
        let tilemap_y = (tile_y / 8) & 0x7F;
        let tilemap_addr = (tilemap_y * 128 + tilemap_x) * 2;
        
        // Read both tile number and attributes
        let tile_data = vram.read16(tilemap_addr as u16);
        let tile_num = (tile_data & 0xFF) as u16;
        let priority = (tile_data & 0x8000) != 0;  // Bit 15 is priority in EXTBG
        
        let pixel_x = tile_x & 7;
        let pixel_y = tile_y & 7;
        
        let tile_addr = tile_num * 64 + (pixel_y * 8 + pixel_x) as u16;
        let pixel = vram.read(tile_addr);
        
        (pixel, priority)
    }
}