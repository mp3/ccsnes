// PPU rendering cache for performance optimization
use crate::ppu::memory::{Vram, Cgram};

const TILE_SIZE: usize = 8;
const TILES_PER_ROW: usize = 32;
const MAX_TILES: usize = 1024;

// Pre-decoded tile data for faster rendering
#[derive(Clone)]
pub struct TileCache {
    // Decoded tile pixels (8x8 pixels, 256 colors each)
    tiles: Vec<[u8; 64]>,
    // Tile dirty flags
    dirty: Vec<bool>,
    // Last VRAM version to detect changes
    vram_version: u64,
}

impl TileCache {
    pub fn new() -> Self {
        Self {
            tiles: vec![[0; 64]; MAX_TILES],
            dirty: vec![true; MAX_TILES],
            vram_version: 0,
        }
    }
    
    pub fn invalidate_all(&mut self) {
        self.dirty.fill(true);
        self.vram_version += 1;
    }
    
    pub fn invalidate_tile(&mut self, tile_index: usize) {
        if tile_index < MAX_TILES {
            self.dirty[tile_index] = true;
        }
    }
    
    // Decode a 2bpp tile (2 bits per pixel, 4 colors)
    pub fn decode_2bpp_tile(&mut self, vram: &Vram, tile_index: usize, base_addr: u16) {
        if tile_index >= MAX_TILES || !self.dirty[tile_index] {
            return;
        }
        
        let tile_addr = base_addr + (tile_index as u16) * 16;
        let tile = &mut self.tiles[tile_index];
        
        for y in 0..8 {
            let low = vram.read(tile_addr + (y * 2) as u16);
            let high = vram.read(tile_addr + (y * 2 + 1) as u16);
            
            for x in 0..8 {
                let bit = 7 - x;
                let color = ((low >> bit) & 1) | (((high >> bit) & 1) << 1);
                tile[y * 8 + x] = color;
            }
        }
        
        self.dirty[tile_index] = false;
    }
    
    // Decode a 4bpp tile (4 bits per pixel, 16 colors)
    pub fn decode_4bpp_tile(&mut self, vram: &Vram, tile_index: usize, base_addr: u16) {
        if tile_index >= MAX_TILES || !self.dirty[tile_index] {
            return;
        }
        
        let tile_addr = base_addr + (tile_index as u16) * 32;
        let tile = &mut self.tiles[tile_index];
        
        for y in 0..8 {
            let plane0 = vram.read(tile_addr + (y * 2) as u16);
            let plane1 = vram.read(tile_addr + (y * 2 + 1) as u16);
            let plane2 = vram.read(tile_addr + (y * 2 + 16) as u16);
            let plane3 = vram.read(tile_addr + (y * 2 + 17) as u16);
            
            for x in 0..8 {
                let bit = 7 - x;
                let color = ((plane0 >> bit) & 1) |
                           (((plane1 >> bit) & 1) << 1) |
                           (((plane2 >> bit) & 1) << 2) |
                           (((plane3 >> bit) & 1) << 3);
                tile[y * 8 + x] = color;
            }
        }
        
        self.dirty[tile_index] = false;
    }
    
    // Decode an 8bpp tile (8 bits per pixel, 256 colors)
    pub fn decode_8bpp_tile(&mut self, vram: &Vram, tile_index: usize, base_addr: u16) {
        if tile_index >= MAX_TILES || !self.dirty[tile_index] {
            return;
        }
        
        let tile_addr = base_addr + (tile_index as u16) * 64;
        let tile = &mut self.tiles[tile_index];
        
        for y in 0..8 {
            for x in 0..8 {
                let byte_offset = y * 8 + x;
                tile[byte_offset] = vram.read(tile_addr + byte_offset as u16);
            }
        }
        
        self.dirty[tile_index] = false;
    }
    
    // Get decoded tile data
    #[inline(always)]
    pub fn get_tile(&self, tile_index: usize) -> &[u8; 64] {
        &self.tiles[tile_index.min(MAX_TILES - 1)]
    }
    
    // Fast tile rendering with flip support
    #[inline(always)]
    pub fn render_tile_to_buffer(
        &self,
        tile_index: usize,
        x: usize,
        y: usize,
        palette_base: u8,
        h_flip: bool,
        v_flip: bool,
        buffer: &mut [u8],
        buffer_width: usize,
        cgram: &Cgram,
    ) {
        let tile = self.get_tile(tile_index);
        
        for ty in 0..8 {
            let src_y = if v_flip { 7 - ty } else { ty };
            let dst_y = y + ty;
            
            if dst_y >= 224 { continue; } // Clip to screen height
            
            for tx in 0..8 {
                let src_x = if h_flip { 7 - tx } else { tx };
                let dst_x = x + tx;
                
                if dst_x >= buffer_width { continue; } // Clip to buffer width
                
                let color_index = tile[src_y * 8 + src_x];
                if color_index == 0 { continue; } // Transparent pixel
                
                let palette_color = palette_base + color_index;
                let color = cgram.read_color(palette_color);
                let (r, g, b) = cgram.color_to_rgb(color);
                
                let offset = (dst_y * buffer_width + dst_x) * 4;
                buffer[offset] = r;
                buffer[offset + 1] = g;
                buffer[offset + 2] = b;
                buffer[offset + 3] = 255; // Alpha
            }
        }
    }
}

// Scanline renderer with optimized pixel operations
pub struct ScanlineRenderer {
    // Priority buffers for each layer
    priority_buffers: Vec<Vec<(u8, u8, u8)>>, // (color_index, priority, layer)
    // Final output buffer
    output_buffer: Vec<u8>,
}

impl ScanlineRenderer {
    pub fn new() -> Self {
        Self {
            priority_buffers: vec![vec![(0, 0, 0); 256]; 4], // 4 priority levels
            output_buffer: vec![0; 256 * 4], // RGBA
        }
    }
    
    pub fn clear(&mut self) {
        for buffer in &mut self.priority_buffers {
            buffer.fill((0, 0, 0));
        }
        self.output_buffer.fill(0);
    }
    
    #[inline(always)]
    pub fn plot_pixel(&mut self, x: usize, color_index: u8, priority: u8, layer: u8) {
        if x >= 256 || color_index == 0 {
            return;
        }
        
        let priority_level = (priority & 3) as usize;
        let current = &self.priority_buffers[priority_level][x];
        
        // Higher layer number = higher priority within same priority level
        if layer >= current.2 {
            self.priority_buffers[priority_level][x] = (color_index, priority, layer);
        }
    }
    
    pub fn composite_to_output(&mut self, cgram: &Cgram) {
        for x in 0..256 {
            let mut final_color = 0u8;
            
            // Check priority levels from highest to lowest
            for priority in (0..4).rev() {
                let (color_index, _, _) = self.priority_buffers[priority][x];
                if color_index != 0 {
                    final_color = color_index;
                    break;
                }
            }
            
            // Convert to RGB
            if final_color != 0 {
                let color = cgram.read_color(final_color);
                let (r, g, b) = cgram.color_to_rgb(color);
                let offset = x * 4;
                self.output_buffer[offset] = r;
                self.output_buffer[offset + 1] = g;
                self.output_buffer[offset + 2] = b;
                self.output_buffer[offset + 3] = 255;
            }
        }
    }
    
    pub fn get_output(&self) -> &[u8] {
        &self.output_buffer
    }
}