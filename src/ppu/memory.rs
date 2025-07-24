// SNES PPU Memory Components

// VRAM - Video RAM (64KB)
// Used for tiles, tilemaps, and Mode 7 data
pub struct Vram {
    data: Vec<u8>,
}

impl Vram {
    pub fn new() -> Self {
        Self {
            data: vec![0; 0x10000], // 64KB
        }
    }
    
    pub fn reset(&mut self) {
        self.data.fill(0);
    }
    
    pub fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }
    
    pub fn write(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }
    
    pub fn read16(&self, address: u16) -> u16 {
        let low = self.read(address) as u16;
        let high = self.read(address.wrapping_add(1)) as u16;
        (high << 8) | low
    }
    
    pub fn write16(&mut self, address: u16, value: u16) {
        self.write(address, (value & 0xFF) as u8);
        self.write(address.wrapping_add(1), (value >> 8) as u8);
    }
    
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

// CGRAM - Color Generator RAM (512 bytes)
// Stores the palette data (256 colors, 2 bytes per color)
pub struct Cgram {
    data: Vec<u8>,
}

impl Cgram {
    pub fn new() -> Self {
        Self {
            data: vec![0; 0x200], // 512 bytes
        }
    }
    
    pub fn reset(&mut self) {
        self.data.fill(0);
    }
    
    pub fn read(&self, address: u8) -> u8 {
        self.data[address as usize]
    }
    
    pub fn write(&mut self, address: u8, value: u8) {
        self.data[address as usize] = value;
    }
    
    pub fn read_color(&self, index: u8) -> u16 {
        let base = (index as usize) * 2;
        let low = self.data[base] as u16;
        let high = self.data[base + 1] as u16;
        (high << 8) | low
    }
    
    pub fn write_color(&mut self, index: u8, color: u16) {
        let base = (index as usize) * 2;
        self.data[base] = (color & 0xFF) as u8;
        self.data[base + 1] = (color >> 8) as u8;
    }
    
    // Convert SNES BGR555 color to RGB888
    pub fn color_to_rgb(&self, color: u16) -> (u8, u8, u8) {
        let r = ((color & 0x001F) << 3) as u8;
        let g = (((color & 0x03E0) >> 5) << 3) as u8;
        let b = (((color & 0x7C00) >> 10) << 3) as u8;
        (r, g, b)
    }
    
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

// OAM - Object Attribute Memory (544 bytes)
// Stores sprite attributes (128 sprites, 4 bytes each + 32 bytes for high table)
pub struct Oam {
    // Low table: 128 sprites × 4 bytes = 512 bytes
    low_table: Vec<u8>,
    // High table: 128 sprites × 2 bits = 32 bytes
    high_table: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteAttributes {
    pub x: i16,        // X position (-256 to 255)
    pub y: u8,         // Y position (0-239)
    pub tile: u16,     // Tile number (0-511 with name select)
    pub palette: u8,   // Palette number (0-7)
    pub priority: u8,  // Priority (0-3)
    pub h_flip: bool,  // Horizontal flip
    pub v_flip: bool,  // Vertical flip
    pub size: bool,    // Size select (0=small, 1=large)
}

impl Oam {
    pub fn new() -> Self {
        Self {
            low_table: vec![0; 512],
            high_table: vec![0; 32],
        }
    }
    
    pub fn reset(&mut self) {
        self.low_table.fill(0);
        self.high_table.fill(0);
    }
    
    pub fn read(&self, address: u16) -> u8 {
        if address < 512 {
            self.low_table[address as usize]
        } else {
            self.high_table[(address - 512) as usize]
        }
    }
    
    pub fn write(&mut self, address: u16, value: u8) {
        if address < 512 {
            self.low_table[address as usize] = value;
        } else {
            self.high_table[(address - 512) as usize] = value;
        }
    }
    
    pub fn get_sprite(&self, index: u8) -> SpriteAttributes {
        let base = (index as usize) * 4;
        
        // Read from low table
        let x_low = self.low_table[base] as i16;
        let y = self.low_table[base + 1];
        let tile_low = self.low_table[base + 2] as u16;
        let attrs = self.low_table[base + 3];
        
        // Read from high table
        let high_index = index / 4;
        let high_shift = (index % 4) * 2;
        let high_byte = self.high_table[high_index as usize];
        let high_bits = (high_byte >> high_shift) & 0x03;
        
        // Extract attributes
        let x_high = if high_bits & 0x01 != 0 { 256 } else { 0 };
        let x = x_low - x_high;
        
        let tile_high = if attrs & 0x01 != 0 { 256 } else { 0 };
        let tile = tile_low | tile_high;
        
        let palette = (attrs >> 1) & 0x07;
        let priority = (attrs >> 4) & 0x03;
        let h_flip = (attrs & 0x40) != 0;
        let v_flip = (attrs & 0x80) != 0;
        let size = (high_bits & 0x02) != 0;
        
        SpriteAttributes {
            x,
            y,
            tile,
            palette,
            priority,
            h_flip,
            v_flip,
            size,
        }
    }
    
    pub fn set_sprite(&mut self, index: u8, sprite: &SpriteAttributes) {
        let base = (index as usize) * 4;
        
        // Write to low table
        self.low_table[base] = (sprite.x & 0xFF) as u8;
        self.low_table[base + 1] = sprite.y;
        self.low_table[base + 2] = (sprite.tile & 0xFF) as u8;
        
        let mut attrs = 0u8;
        if sprite.tile > 255 { attrs |= 0x01; }
        attrs |= (sprite.palette & 0x07) << 1;
        attrs |= (sprite.priority & 0x03) << 4;
        if sprite.h_flip { attrs |= 0x40; }
        if sprite.v_flip { attrs |= 0x80; }
        self.low_table[base + 3] = attrs;
        
        // Write to high table
        let high_index = index / 4;
        let high_shift = (index % 4) * 2;
        let mut high_byte = self.high_table[high_index as usize];
        
        // Clear the 2 bits for this sprite
        high_byte &= !(0x03 << high_shift);
        
        // Set new bits
        let mut high_bits = 0u8;
        if sprite.x < 0 { high_bits |= 0x01; }
        if sprite.size { high_bits |= 0x02; }
        
        high_byte |= high_bits << high_shift;
        self.high_table[high_index as usize] = high_byte;
    }
    
    pub fn get_data(&self) -> &[u8] {
        // Return low table for simplicity - TODO: combine both tables
        &self.low_table
    }
}