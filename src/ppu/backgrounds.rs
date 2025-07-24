use crate::ppu::memory::{Vram, Cgram};
use crate::ppu::registers::PpuRegisters;

// Tile size constants
const TILE_SIZE: usize = 8;

// Background modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BgMode {
    Mode0, // 4 BGs, 2bpp each
    Mode1, // BG1/2: 4bpp, BG3: 2bpp
    Mode2, // BG1/2: 4bpp, offset-per-tile
    Mode3, // BG1: 8bpp, BG2: 4bpp
    Mode4, // BG1: 8bpp, BG2: 2bpp, offset-per-tile
    Mode5, // BG1: 4bpp, BG2: 2bpp, hi-res
    Mode6, // BG1: 4bpp, offset-per-tile, hi-res
    Mode7, // Mode 7 rotation/scaling
}

impl From<u8> for BgMode {
    fn from(value: u8) -> Self {
        match value & 0x07 {
            0 => BgMode::Mode0,
            1 => BgMode::Mode1,
            2 => BgMode::Mode2,
            3 => BgMode::Mode3,
            4 => BgMode::Mode4,
            5 => BgMode::Mode5,
            6 => BgMode::Mode6,
            7 => BgMode::Mode7,
            _ => BgMode::Mode0,
        }
    }
}

// Background layer info
#[derive(Debug, Clone, Copy)]
pub struct BackgroundInfo {
    pub tilemap_base: u16,    // VRAM address of tilemap
    pub tile_base: u16,       // VRAM address of tiles
    pub tilemap_size: (u8, u8), // Width/height (0=32, 1=64)
    pub tile_size: bool,      // false=8x8, true=16x16
    pub priority: [bool; 2],  // Priority bits
    pub h_scroll: u16,        // Horizontal scroll
    pub v_scroll: u16,        // Vertical scroll
}

pub struct BackgroundRenderer {
    // Temporary scanline buffer for each BG layer
    bg1_buffer: Vec<u8>,
    bg2_buffer: Vec<u8>,
    bg3_buffer: Vec<u8>,
    bg4_buffer: Vec<u8>,
}

impl BackgroundRenderer {
    pub fn new() -> Self {
        Self {
            bg1_buffer: vec![0; 256 * 4], // RGBA buffer
            bg2_buffer: vec![0; 256 * 4],
            bg3_buffer: vec![0; 256 * 4],
            bg4_buffer: vec![0; 256 * 4],
        }
    }
    
    pub fn get_bg_info(registers: &PpuRegisters, bg_num: u8) -> BackgroundInfo {
        let (sc_reg, bg_reg) = match bg_num {
            1 => (registers.bg1sc, registers.bg12nba),
            2 => (registers.bg2sc, registers.bg12nba),
            3 => (registers.bg3sc, registers.bg34nba),
            4 => (registers.bg4sc, registers.bg34nba),
            _ => panic!("Invalid BG number"),
        };
        
        // Extract tilemap base address (bits 2-7 of BGnSC)
        let tilemap_base = ((sc_reg & 0xFC) as u16) << 9; // * 1KB
        
        // Extract tile base address
        let tile_base = if bg_num <= 2 {
            ((bg_reg & 0x0F) as u16) << 12 // BG1/2 use low nibble
        } else {
            (((bg_reg & 0xF0) >> 4) as u16) << 12 // BG3/4 use high nibble
        };
        
        // Extract tilemap size (bits 0-1 of BGnSC)
        let size_bits = sc_reg & 0x03;
        let tilemap_size = (
            (size_bits & 0x01) as u8,      // Horizontal size
            ((size_bits & 0x02) >> 1) as u8 // Vertical size
        );
        
        // Get tile size from BGMODE register
        let tile_size = match bg_num {
            1 => (registers.bgmode & 0x10) != 0,
            2 => (registers.bgmode & 0x20) != 0,
            3 => (registers.bgmode & 0x40) != 0,
            4 => (registers.bgmode & 0x80) != 0,
            _ => false,
        };
        
        // Get scroll values
        let (h_scroll, v_scroll) = match bg_num {
            1 => (registers.bg1hofs, registers.bg1vofs),
            2 => (registers.bg2hofs, registers.bg2vofs),
            3 => (registers.bg3hofs, registers.bg3vofs),
            4 => (registers.bg4hofs, registers.bg4vofs),
            _ => (0, 0),
        };
        
        BackgroundInfo {
            tilemap_base,
            tile_base,
            tilemap_size,
            tile_size,
            priority: [false, false], // TODO: Extract from tilemap entries
            h_scroll,
            v_scroll,
        }
    }

    pub fn render_scanline(
        &mut self,
        vram: &Vram,
        cgram: &Cgram,
        registers: &PpuRegisters,
        scanline: u16,
    ) -> &[u8] {
        let bg_mode = BgMode::from(registers.bgmode);
        
        // Clear buffers
        self.bg1_buffer.fill(0);
        self.bg2_buffer.fill(0);
        self.bg3_buffer.fill(0);
        self.bg4_buffer.fill(0);
        
        // Render appropriate backgrounds based on mode
        match bg_mode {
            BgMode::Mode0 => {
                // 4 backgrounds, 2bpp each
                Self::render_bg_2bpp(vram, cgram, registers, 1, scanline, &mut self.bg1_buffer);
                Self::render_bg_2bpp(vram, cgram, registers, 2, scanline, &mut self.bg2_buffer);
                Self::render_bg_2bpp(vram, cgram, registers, 3, scanline, &mut self.bg3_buffer);
                Self::render_bg_2bpp(vram, cgram, registers, 4, scanline, &mut self.bg4_buffer);
            }
            BgMode::Mode1 => {
                // BG1/2: 4bpp, BG3: 2bpp
                Self::render_bg_4bpp(vram, cgram, registers, 1, scanline, &mut self.bg1_buffer);
                Self::render_bg_4bpp(vram, cgram, registers, 2, scanline, &mut self.bg2_buffer);
                Self::render_bg_2bpp(vram, cgram, registers, 3, scanline, &mut self.bg3_buffer);
            }
            BgMode::Mode3 => {
                // BG1: 8bpp, BG2: 4bpp
                Self::render_bg_8bpp(vram, cgram, registers, 1, scanline, &mut self.bg1_buffer);
                Self::render_bg_4bpp(vram, cgram, registers, 2, scanline, &mut self.bg2_buffer);
            }
            _ => {
                // TODO: Implement other modes
            }
        }
        
        // For now, just return BG1 buffer
        &self.bg1_buffer
    }
    
    fn render_bg_2bpp(
        vram: &Vram,
        cgram: &Cgram,
        registers: &PpuRegisters,
        bg_num: u8,
        scanline: u16,
        buffer: &mut [u8],
    ) {
        let bg_info = Self::get_bg_info(registers, bg_num);
        let y = (scanline as u32 + bg_info.v_scroll as u32) & 0x1FF;
        let tile_y = y / TILE_SIZE as u32;
        let fine_y = y % TILE_SIZE as u32;
        
        // Render each pixel in the scanline
        for x in 0..256u16 {
            let scroll_x = (x as u32 + bg_info.h_scroll as u32) & 0x1FF;
            let tile_x = scroll_x / TILE_SIZE as u32;
            let fine_x = scroll_x % TILE_SIZE as u32;
            
            // Calculate tilemap address
            let tilemap_x = tile_x & 31;
            let tilemap_y = tile_y & 31;
            let tilemap_addr = bg_info.tilemap_base + (tilemap_y * 32 + tilemap_x) as u16 * 2;
            
            // Read tilemap entry
            let tilemap_entry = vram.read16(tilemap_addr);
            let tile_num = tilemap_entry & 0x3FF;
            let palette_num = ((tilemap_entry >> 10) & 0x07) as u8;
            let h_flip = (tilemap_entry & 0x4000) != 0;
            let v_flip = (tilemap_entry & 0x8000) != 0;
            
            // Calculate pixel position within tile
            let pixel_x = if h_flip { 7 - fine_x } else { fine_x };
            let pixel_y = if v_flip { 7 - fine_y } else { fine_y };
            
            // Read tile data (2bpp = 2 bits per pixel)
            let tile_addr = bg_info.tile_base + tile_num * 8;
            let byte_offset = pixel_y * 2; // 2 bytes per row in 2bpp
            
            let low_byte = vram.read(tile_addr + byte_offset as u16);
            let high_byte = vram.read(tile_addr + byte_offset as u16 + 1);
            
            let bit_mask = 0x80 >> pixel_x;
            let low_bit = if (low_byte & bit_mask) != 0 { 1 } else { 0 };
            let high_bit = if (high_byte & bit_mask) != 0 { 2 } else { 0 };
            let color_index = low_bit | high_bit;
            
            // Skip transparent pixels
            if color_index == 0 {
                continue;
            }
            
            // Get color from CGRAM
            let cgram_index = palette_num * 4 + color_index;
            let color = cgram.read_color(cgram_index);
            let (r, g, b) = cgram.color_to_rgb(color);
            
            // Write to buffer
            let buffer_offset = (x as usize) * 4;
            buffer[buffer_offset] = r;
            buffer[buffer_offset + 1] = g;
            buffer[buffer_offset + 2] = b;
            buffer[buffer_offset + 3] = 255;
        }
    }
    
    fn render_bg_4bpp(
        vram: &Vram,
        cgram: &Cgram,
        registers: &PpuRegisters,
        bg_num: u8,
        scanline: u16,
        buffer: &mut [u8],
    ) {
        let bg_info = Self::get_bg_info(registers, bg_num);
        let y = (scanline as u32 + bg_info.v_scroll as u32) & 0x1FF;
        let tile_y = y / TILE_SIZE as u32;
        let fine_y = y % TILE_SIZE as u32;
        
        for x in 0..256u16 {
            let scroll_x = (x as u32 + bg_info.h_scroll as u32) & 0x1FF;
            let tile_x = scroll_x / TILE_SIZE as u32;
            let fine_x = scroll_x % TILE_SIZE as u32;
            
            let tilemap_x = tile_x & 31;
            let tilemap_y = tile_y & 31;
            let tilemap_addr = bg_info.tilemap_base + (tilemap_y * 32 + tilemap_x) as u16 * 2;
            
            let tilemap_entry = vram.read16(tilemap_addr);
            let tile_num = tilemap_entry & 0x3FF;
            let palette_num = ((tilemap_entry >> 10) & 0x07) as u8;
            let h_flip = (tilemap_entry & 0x4000) != 0;
            let v_flip = (tilemap_entry & 0x8000) != 0;
            
            let pixel_x = if h_flip { 7 - fine_x } else { fine_x };
            let pixel_y = if v_flip { 7 - fine_y } else { fine_y };
            
            // 4bpp = 4 bits per pixel, 4 bytes per row
            let tile_addr = bg_info.tile_base + tile_num * 16;
            let byte_offset = pixel_y * 2;
            
            let plane0 = vram.read(tile_addr + byte_offset as u16);
            let plane1 = vram.read(tile_addr + byte_offset as u16 + 1);
            let plane2 = vram.read(tile_addr + byte_offset as u16 + 8);
            let plane3 = vram.read(tile_addr + byte_offset as u16 + 9);
            
            let bit_mask = 0x80 >> pixel_x;
            let bit0 = if (plane0 & bit_mask) != 0 { 1 } else { 0 };
            let bit1 = if (plane1 & bit_mask) != 0 { 2 } else { 0 };
            let bit2 = if (plane2 & bit_mask) != 0 { 4 } else { 0 };
            let bit3 = if (plane3 & bit_mask) != 0 { 8 } else { 0 };
            let color_index = bit0 | bit1 | bit2 | bit3;
            
            if color_index == 0 {
                continue;
            }
            
            let cgram_index = palette_num * 16 + color_index;
            let color = cgram.read_color(cgram_index);
            let (r, g, b) = cgram.color_to_rgb(color);
            
            let buffer_offset = (x as usize) * 4;
            buffer[buffer_offset] = r;
            buffer[buffer_offset + 1] = g;
            buffer[buffer_offset + 2] = b;
            buffer[buffer_offset + 3] = 255;
        }
    }
    
    fn render_bg_8bpp(
        vram: &Vram,
        cgram: &Cgram,
        registers: &PpuRegisters,
        bg_num: u8,
        scanline: u16,
        buffer: &mut [u8],
    ) {
        let bg_info = Self::get_bg_info(registers, bg_num);
        let y = (scanline as u32 + bg_info.v_scroll as u32) & 0x1FF;
        let tile_y = y / TILE_SIZE as u32;
        let fine_y = y % TILE_SIZE as u32;
        
        for x in 0..256u16 {
            let scroll_x = (x as u32 + bg_info.h_scroll as u32) & 0x1FF;
            let tile_x = scroll_x / TILE_SIZE as u32;
            let fine_x = scroll_x % TILE_SIZE as u32;
            
            let tilemap_x = tile_x & 31;
            let tilemap_y = tile_y & 31;
            let tilemap_addr = bg_info.tilemap_base + (tilemap_y * 32 + tilemap_x) as u16 * 2;
            
            let tilemap_entry = vram.read16(tilemap_addr);
            let tile_num = tilemap_entry & 0x3FF;
            let h_flip = (tilemap_entry & 0x4000) != 0;
            let v_flip = (tilemap_entry & 0x8000) != 0;
            
            let pixel_x = if h_flip { 7 - fine_x } else { fine_x };
            let pixel_y = if v_flip { 7 - fine_y } else { fine_y };
            
            // 8bpp = 8 bits per pixel, 8 bytes per row
            let tile_addr = bg_info.tile_base + tile_num * 32;
            let byte_offset = pixel_y * 4;
            
            // Read all 8 bitplanes
            let mut color_index = 0u8;
            for plane in 0..8 {
                let plane_offset = (plane / 2) * 8 + (plane % 2);
                let plane_byte = vram.read(tile_addr + byte_offset as u16 + plane_offset);
                let bit_mask = 0x80 >> pixel_x;
                if (plane_byte & bit_mask) != 0 {
                    color_index |= 1 << plane;
                }
            }
            
            if color_index == 0 {
                continue;
            }
            
            let color = cgram.read_color(color_index);
            let (r, g, b) = cgram.color_to_rgb(color);
            
            let buffer_offset = (x as usize) * 4;
            buffer[buffer_offset] = r;
            buffer[buffer_offset + 1] = g;
            buffer[buffer_offset + 2] = b;
            buffer[buffer_offset + 3] = 255;
        }
    }
}