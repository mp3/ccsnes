use crate::ppu::memory::{Vram, Cgram, Oam, SpriteAttributes};
use crate::ppu::registers::PpuRegisters;

// SNES sprite sizes
const SPRITE_SIZE_SMALL: [(u8, u8); 4] = [
    (8, 8),   // 0: 8x8, 16x16
    (8, 8),   // 1: 8x8, 32x32
    (8, 8),   // 2: 8x8, 64x64
    (16, 16), // 3: 16x16, 32x32
];

const SPRITE_SIZE_LARGE: [(u8, u8); 4] = [
    (16, 16), // 0: 8x8, 16x16
    (32, 32), // 1: 8x8, 32x32
    (64, 64), // 2: 8x8, 64x64
    (32, 32), // 3: 16x16, 32x32
];

// Sprite priority table for sprite-to-sprite priority
#[derive(Debug, Clone, Copy)]
struct SpritePixel {
    color: u8,
    palette: u8,
    priority: u8,
    sprite_priority: u8, // OAM index for sprite-to-sprite priority
}

pub struct SpriteRenderer {
    // Scanline buffers for each priority level
    priority_buffers: [Vec<Option<SpritePixel>>; 4],
    // Sprite evaluation results for current scanline
    active_sprites: Vec<(u8, SpriteAttributes)>, // (index, attributes)
}

impl SpriteRenderer {
    pub fn new() -> Self {
        Self {
            priority_buffers: [
                vec![None; 256],
                vec![None; 256],
                vec![None; 256],
                vec![None; 256],
            ],
            active_sprites: Vec::with_capacity(32), // Max 32 sprites per scanline
        }
    }
    
    pub fn render_scanline(
        &mut self,
        vram: &Vram,
        cgram: &Cgram,
        oam: &Oam,
        registers: &PpuRegisters,
        scanline: u16,
        buffer: &mut [u8],
    ) {
        // Clear priority buffers
        for buffer in &mut self.priority_buffers {
            buffer.fill(None);
        }
        
        // Get sprite size settings
        let (size_small, size_large) = self.get_sprite_sizes(registers);
        
        // Evaluate sprites for this scanline
        self.evaluate_sprites(oam, scanline, size_small, size_large);
        
        // Render active sprites
        for i in 0..self.active_sprites.len() {
            let (sprite_index, sprite) = self.active_sprites[i];
            self.render_sprite(
                vram,
                registers,
                sprite_index,
                &sprite,
                scanline,
                size_small,
                size_large,
            );
        }
        
        // Composite sprites onto output buffer
        self.composite_sprites(cgram, buffer);
    }
    
    fn get_sprite_sizes(&self, registers: &PpuRegisters) -> ((u8, u8), (u8, u8)) {
        let size_select = (registers.obsel >> 5) & 0x07;
        let size_index = match size_select {
            0 => 0, // 8x8, 16x16
            1 => 1, // 8x8, 32x32
            2 => 2, // 8x8, 64x64
            3 => 3, // 16x16, 32x32
            4 => 3, // 16x16, 64x64
            5 => 1, // 32x32, 64x64
            6 => 3, // 16x32, 32x64 (treated as 16x16, 32x32)
            7 => 3, // 16x32, 32x32 (treated as 16x16, 32x32)
            _ => 0,
        };
        
        (SPRITE_SIZE_SMALL[size_index], SPRITE_SIZE_LARGE[size_index])
    }
    
    fn evaluate_sprites(
        &mut self,
        oam: &Oam,
        scanline: u16,
        size_small: (u8, u8),
        size_large: (u8, u8),
    ) {
        self.active_sprites.clear();
        
        // Check all 128 sprites
        for i in 0..128u8 {
            let sprite = oam.get_sprite(i);
            
            // Get sprite size
            let (_width, height) = if sprite.size {
                size_large
            } else {
                size_small
            };
            
            // Check if sprite is on this scanline
            let sprite_top = sprite.y as i16;
            let sprite_bottom = sprite_top + height as i16;
            
            if scanline as i16 >= sprite_top && (scanline as i16) < sprite_bottom {
                self.active_sprites.push((i, sprite));
                
                // Stop at 32 sprites per scanline
                if self.active_sprites.len() >= 32 {
                    break;
                }
            }
        }
    }
    
    fn render_sprite(
        &mut self,
        vram: &Vram,
        registers: &PpuRegisters,
        sprite_index: u8,
        sprite: &SpriteAttributes,
        scanline: u16,
        size_small: (u8, u8),
        size_large: (u8, u8),
    ) {
        // Get sprite size
        let (width, height) = if sprite.size {
            size_large
        } else {
            size_small
        };
        
        // Calculate row within sprite
        let sprite_y = (scanline as i16 - sprite.y as i16) as u16;
        let row = if sprite.v_flip {
            height as u16 - 1 - sprite_y
        } else {
            sprite_y
        };
        
        // Get name base from OBSEL register
        let name_base = ((registers.obsel & 0x07) as u16) << 13;
        
        // Render each pixel in the sprite row
        for col in 0..width {
            let x = sprite.x + col as i16;
            
            // Skip if off-screen
            if x < 0 || x >= 256 {
                continue;
            }
            
            // Get pixel from tile
            let pixel_x = if sprite.h_flip {
                width - 1 - col
            } else {
                col
            };
            
            // Calculate tile coordinates
            let tile_x = (pixel_x / 8) as u16;
            let tile_y = row / 8;
            let fine_x = pixel_x % 8;
            let fine_y = row % 8;
            
            // Calculate tile number
            let tile_offset = if width > 8 || height > 8 {
                // Large sprite - tiles are arranged in a grid
                tile_y * ((width / 8) as u16) + tile_x
            } else {
                0
            };
            
            let tile_num = sprite.tile + tile_offset as u16;
            
            // Calculate VRAM address
            let vram_addr = name_base + (tile_num << 4); // 16 bytes per tile in 4bpp
            
            // Read tile data (4bpp)
            let byte_offset = fine_y * 2;
            let plane0 = vram.read(vram_addr + byte_offset);
            let plane1 = vram.read(vram_addr + byte_offset + 1);
            let plane2 = vram.read(vram_addr + byte_offset + 8);
            let plane3 = vram.read(vram_addr + byte_offset + 9);
            
            let bit_mask = 0x80 >> fine_x;
            let bit0 = if (plane0 & bit_mask) != 0 { 1 } else { 0 };
            let bit1 = if (plane1 & bit_mask) != 0 { 2 } else { 0 };
            let bit2 = if (plane2 & bit_mask) != 0 { 4 } else { 0 };
            let bit3 = if (plane3 & bit_mask) != 0 { 8 } else { 0 };
            let color_index = bit0 | bit1 | bit2 | bit3;
            
            // Skip transparent pixels
            if color_index == 0 {
                continue;
            }
            
            // Store pixel in priority buffer
            let pixel = SpritePixel {
                color: color_index,
                palette: sprite.palette + 8, // Sprite palettes start at 128
                priority: sprite.priority,
                sprite_priority: sprite_index,
            };
            
            let buffer = &mut self.priority_buffers[sprite.priority as usize];
            let x_pos = x as usize;
            
            // Check sprite-to-sprite priority
            if let Some(existing) = buffer[x_pos] {
                // Lower OAM index = higher priority
                if sprite_index < existing.sprite_priority {
                    buffer[x_pos] = Some(pixel);
                }
            } else {
                buffer[x_pos] = Some(pixel);
            }
        }
    }
    
    fn composite_sprites(&self, cgram: &Cgram, buffer: &mut [u8]) {
        // Composite sprites from highest to lowest priority
        for x in 0..256 {
            for priority in (0..4).rev() {
                if let Some(pixel) = self.priority_buffers[priority][x] {
                    // Get color from CGRAM
                    let cgram_index = pixel.palette * 16 + pixel.color;
                    let color = cgram.read_color(cgram_index);
                    let (r, g, b) = cgram.color_to_rgb(color);
                    
                    // Write to buffer
                    let offset = x * 4;
                    buffer[offset] = r;
                    buffer[offset + 1] = g;
                    buffer[offset + 2] = b;
                    buffer[offset + 3] = 255;
                    
                    // Stop after first non-transparent pixel
                    break;
                }
            }
        }
    }
    
    pub fn get_priority_buffer(&self, priority: u8) -> &[Option<SpritePixel>] {
        &self.priority_buffers[priority as usize]
    }
}