use std::fmt;

#[derive(Debug, Clone)]
pub struct PpuRegisters {
    // Display control registers
    pub inidisp: u8,    // $2100 - Screen display (brightness and blanking)
    pub obsel: u8,      // $2101 - Object size and character address
    pub oamaddl: u8,    // $2102 - OAM address (low)
    pub oamaddh: u8,    // $2103 - OAM address (high)
    pub oamdata: u8,    // $2104 - OAM data write
    
    // Background control
    pub bgmode: u8,     // $2105 - BG mode and character size
    pub mosaic: u8,     // $2106 - Mosaic settings
    pub bg1sc: u8,      // $2107 - BG1 screen address
    pub bg2sc: u8,      // $2108 - BG2 screen address
    pub bg3sc: u8,      // $2109 - BG3 screen address
    pub bg4sc: u8,      // $210A - BG4 screen address
    pub bg12nba: u8,    // $210B - BG1/2 character data area
    pub bg34nba: u8,    // $210C - BG3/4 character data area
    
    // Background scroll registers
    pub bg1hofs: u16,   // $210D - BG1 horizontal scroll
    pub bg1vofs: u16,   // $210E - BG1 vertical scroll
    pub bg2hofs: u16,   // $210F - BG2 horizontal scroll
    pub bg2vofs: u16,   // $2110 - BG2 vertical scroll
    pub bg3hofs: u16,   // $2111 - BG3 horizontal scroll
    pub bg3vofs: u16,   // $2112 - BG3 vertical scroll
    pub bg4hofs: u16,   // $2113 - BG4 horizontal scroll
    pub bg4vofs: u16,   // $2114 - BG4 vertical scroll
    
    // VRAM control
    pub vmain: u8,      // $2115 - Video port control
    pub vmaddl: u8,     // $2116 - VRAM address (low)
    pub vmaddh: u8,     // $2117 - VRAM address (high)
    pub vmdatal: u8,    // $2118 - VRAM data write (low)
    pub vmdatah: u8,    // $2119 - VRAM data write (high)
    
    // Mode 7 registers
    pub m7sel: u8,      // $211A - Mode 7 settings
    pub m7a: i16,       // $211B - Mode 7 matrix parameter A
    pub m7b: i16,       // $211C - Mode 7 matrix parameter B
    pub m7c: i16,       // $211D - Mode 7 matrix parameter C
    pub m7d: i16,       // $211E - Mode 7 matrix parameter D
    pub m7x: i16,       // $211F - Mode 7 center X
    pub m7y: i16,       // $2120 - Mode 7 center Y
    
    // Color math registers
    pub cgadd: u8,      // $2121 - CGRAM address
    pub cgdata: u8,     // $2122 - CGRAM data write
    pub w12sel: u8,     // $2123 - Window mask settings BG1/2
    pub w34sel: u8,     // $2124 - Window mask settings BG3/4
    pub wobjsel: u8,    // $2125 - Window mask settings OBJ/Color
    pub wh0: u8,        // $2126 - Window 1 left position
    pub wh1: u8,        // $2127 - Window 1 right position
    pub wh2: u8,        // $2128 - Window 2 left position
    pub wh3: u8,        // $2129 - Window 2 right position
    pub wbglog: u8,     // $212A - Window mask logic BG1-4
    pub wobjlog: u8,    // $212B - Window mask logic OBJ/Color
    pub tm: u8,         // $212C - Main screen designation
    pub ts: u8,         // $212D - Sub screen designation
    pub tmw: u8,        // $212E - Main screen window mask
    pub tsw: u8,        // $212F - Sub screen window mask
    pub cgwsel: u8,     // $2130 - Color addition select
    pub cgadsub: u8,    // $2131 - Color math designation
    pub coldata: u8,    // $2132 - Fixed color data
    pub setini: u8,     // $2133 - Screen mode/video select
    
    // Internal state for write-twice registers
    pub ppu1_latch: bool,
    pub ppu2_latch: bool,
    pub cgram_latch: bool,
    pub cgram_data_latch: u8,
}

impl PpuRegisters {
    pub fn new() -> Self {
        Self {
            // Initialize all registers to their power-on states
            inidisp: 0x80,     // Screen blanked on power-on
            obsel: 0,
            oamaddl: 0,
            oamaddh: 0,
            oamdata: 0,
            bgmode: 0,
            mosaic: 0,
            bg1sc: 0,
            bg2sc: 0,
            bg3sc: 0,
            bg4sc: 0,
            bg12nba: 0,
            bg34nba: 0,
            bg1hofs: 0,
            bg1vofs: 0,
            bg2hofs: 0,
            bg2vofs: 0,
            bg3hofs: 0,
            bg3vofs: 0,
            bg4hofs: 0,
            bg4vofs: 0,
            vmain: 0,
            vmaddl: 0,
            vmaddh: 0,
            vmdatal: 0,
            vmdatah: 0,
            m7sel: 0,
            m7a: 0,
            m7b: 0,
            m7c: 0,
            m7d: 0,
            m7x: 0,
            m7y: 0,
            cgadd: 0,
            cgdata: 0,
            w12sel: 0,
            w34sel: 0,
            wobjsel: 0,
            wh0: 0,
            wh1: 0,
            wh2: 0,
            wh3: 0,
            wbglog: 0,
            wobjlog: 0,
            tm: 0,
            ts: 0,
            tmw: 0,
            tsw: 0,
            cgwsel: 0,
            cgadsub: 0,
            coldata: 0,
            setini: 0,
            
            ppu1_latch: false,
            ppu2_latch: false,
            cgram_latch: false,
            cgram_data_latch: 0,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x2100 => self.inidisp = value,
            0x2101 => self.obsel = value,
            0x2102 => self.oamaddl = value,
            0x2103 => self.oamaddh = value & 0x81, // Only bits 0 and 7 are used
            0x2104 => self.oamdata = value,
            0x2105 => self.bgmode = value,
            0x2106 => self.mosaic = value,
            0x2107 => self.bg1sc = value,
            0x2108 => self.bg2sc = value,
            0x2109 => self.bg3sc = value,
            0x210A => self.bg4sc = value,
            0x210B => self.bg12nba = value,
            0x210C => self.bg34nba = value,
            
            // BG scroll registers (write twice)
            0x210D => {
                if self.ppu1_latch {
                    self.bg1hofs = ((value as u16) << 8) | (self.bg1hofs & 0xFF);
                } else {
                    self.bg1hofs = (self.bg1hofs & 0xFF00) | (value as u16);
                }
                self.ppu1_latch = !self.ppu1_latch;
            }
            0x210E => {
                if self.ppu1_latch {
                    self.bg1vofs = ((value as u16) << 8) | (self.bg1vofs & 0xFF);
                } else {
                    self.bg1vofs = (self.bg1vofs & 0xFF00) | (value as u16);
                }
                self.ppu1_latch = !self.ppu1_latch;
            }
            0x210F => {
                if self.ppu1_latch {
                    self.bg2hofs = ((value as u16) << 8) | (self.bg2hofs & 0xFF);
                } else {
                    self.bg2hofs = (self.bg2hofs & 0xFF00) | (value as u16);
                }
                self.ppu1_latch = !self.ppu1_latch;
            }
            0x2110 => {
                if self.ppu1_latch {
                    self.bg2vofs = ((value as u16) << 8) | (self.bg2vofs & 0xFF);
                } else {
                    self.bg2vofs = (self.bg2vofs & 0xFF00) | (value as u16);
                }
                self.ppu1_latch = !self.ppu1_latch;
            }
            0x2111 => {
                if self.ppu1_latch {
                    self.bg3hofs = ((value as u16) << 8) | (self.bg3hofs & 0xFF);
                } else {
                    self.bg3hofs = (self.bg3hofs & 0xFF00) | (value as u16);
                }
                self.ppu1_latch = !self.ppu1_latch;
            }
            0x2112 => {
                if self.ppu1_latch {
                    self.bg3vofs = ((value as u16) << 8) | (self.bg3vofs & 0xFF);
                } else {
                    self.bg3vofs = (self.bg3vofs & 0xFF00) | (value as u16);
                }
                self.ppu1_latch = !self.ppu1_latch;
            }
            0x2113 => {
                if self.ppu1_latch {
                    self.bg4hofs = ((value as u16) << 8) | (self.bg4hofs & 0xFF);
                } else {
                    self.bg4hofs = (self.bg4hofs & 0xFF00) | (value as u16);
                }
                self.ppu1_latch = !self.ppu1_latch;
            }
            0x2114 => {
                if self.ppu1_latch {
                    self.bg4vofs = ((value as u16) << 8) | (self.bg4vofs & 0xFF);
                } else {
                    self.bg4vofs = (self.bg4vofs & 0xFF00) | (value as u16);
                }
                self.ppu1_latch = !self.ppu1_latch;
            }
            
            // VRAM control
            0x2115 => self.vmain = value,
            0x2116 => self.vmaddl = value,
            0x2117 => self.vmaddh = value & 0x7F, // Bit 7 is unused
            0x2118 => self.vmdatal = value,
            0x2119 => self.vmdatah = value,
            
            // Mode 7 registers (write twice)
            0x211A => self.m7sel = value,
            0x211B => {
                if self.ppu1_latch {
                    self.m7a = ((value as i16) << 8) | (self.m7a & 0xFF);
                } else {
                    self.m7a = ((self.m7a as u16 & 0xFF00) as i16) | (value as i16);
                }
                self.ppu1_latch = !self.ppu1_latch;
            }
            0x211C => {
                if self.ppu1_latch {
                    self.m7b = ((value as i16) << 8) | (self.m7b & 0xFF);
                } else {
                    self.m7b = ((self.m7b as u16 & 0xFF00) as i16) | (value as i16);
                }
                self.ppu1_latch = !self.ppu1_latch;
            }
            0x211D => {
                if self.ppu1_latch {
                    self.m7c = ((value as i16) << 8) | (self.m7c & 0xFF);
                } else {
                    self.m7c = ((self.m7c as u16 & 0xFF00) as i16) | (value as i16);
                }
                self.ppu1_latch = !self.ppu1_latch;
            }
            0x211E => {
                if self.ppu1_latch {
                    self.m7d = ((value as i16) << 8) | (self.m7d & 0xFF);
                } else {
                    self.m7d = ((self.m7d as u16 & 0xFF00) as i16) | (value as i16);
                }
                self.ppu1_latch = !self.ppu1_latch;
            }
            0x211F => {
                if self.ppu1_latch {
                    self.m7x = ((value as i16) << 8) | (self.m7x & 0xFF);
                } else {
                    self.m7x = ((self.m7x as u16 & 0xFF00) as i16) | (value as i16);
                }
                self.ppu1_latch = !self.ppu1_latch;
            }
            0x2120 => {
                if self.ppu1_latch {
                    self.m7y = ((value as i16) << 8) | (self.m7y & 0xFF);
                } else {
                    self.m7y = ((self.m7y as u16 & 0xFF00) as i16) | (value as i16);
                }
                self.ppu1_latch = !self.ppu1_latch;
            }
            
            // Color math registers
            0x2121 => {
                self.cgadd = value;
                self.cgram_latch = false;
            }
            0x2122 => self.cgdata = value,
            0x2123 => self.w12sel = value,
            0x2124 => self.w34sel = value,
            0x2125 => self.wobjsel = value,
            0x2126 => self.wh0 = value,
            0x2127 => self.wh1 = value,
            0x2128 => self.wh2 = value,
            0x2129 => self.wh3 = value,
            0x212A => self.wbglog = value,
            0x212B => self.wobjlog = value,
            0x212C => self.tm = value,
            0x212D => self.ts = value,
            0x212E => self.tmw = value,
            0x212F => self.tsw = value,
            0x2130 => self.cgwsel = value,
            0x2131 => self.cgadsub = value,
            0x2132 => self.coldata = value,
            0x2133 => self.setini = value,
            
            _ => {} // Other addresses are read-only or unused
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            // Most PPU registers are write-only
            // Only a few registers can be read
            0x2134..=0x2136 => 0, // Multiplication result (implemented later)
            0x2137 => 0,          // Software latch
            0x2138 => 0,          // OAM data read (implemented later)
            0x2139 => 0,          // VRAM data read low (implemented later)
            0x213A => 0,          // VRAM data read high (implemented later)
            0x213B => 0,          // CGRAM data read (implemented later)
            0x213C => 0,          // H counter (implemented later)
            0x213D => 0,          // V counter (implemented later)
            0x213E => 1,          // PPU status flag (PPU1)
            0x213F => 2,          // PPU status flag (PPU2)
            _ => 0,               // Open bus
        }
    }

    // Helper methods
    pub fn is_screen_blanked(&self) -> bool {
        (self.inidisp & 0x80) != 0
    }

    pub fn get_brightness(&self) -> u8 {
        self.inidisp & 0x0F
    }

    pub fn get_bg_mode(&self) -> u8 {
        self.bgmode & 0x07
    }

    pub fn get_bg_character_size(&self, bg: u8) -> bool {
        // True = 16x16, False = 8x8
        match bg {
            1 => (self.bgmode & 0x10) != 0,
            2 => (self.bgmode & 0x20) != 0,
            3 => (self.bgmode & 0x40) != 0,
            4 => (self.bgmode & 0x80) != 0,
            _ => false,
        }
    }

    pub fn get_vram_address(&self) -> u16 {
        ((self.vmaddh as u16) << 8) | (self.vmaddl as u16)
    }

    pub fn set_vram_address(&mut self, address: u16) {
        self.vmaddl = (address & 0xFF) as u8;
        self.vmaddh = ((address >> 8) & 0x7F) as u8;
    }

    pub fn get_oam_address(&self) -> u16 {
        ((self.oamaddh as u16) << 8) | (self.oamaddl as u16)
    }

    pub fn get_main_screen_layers(&self) -> u8 {
        self.tm
    }

    pub fn get_sub_screen_layers(&self) -> u8 {
        self.ts
    }
}

impl fmt::Display for PpuRegisters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PPU: Mode {} ", self.get_bg_mode())?;
        write!(f, "Screen: {}", if self.is_screen_blanked() { "Blanked" } else { "Active" })?;
        write!(f, " Brightness: {}", self.get_brightness())?;
        write!(f, " Main: ${:02X} Sub: ${:02X}", self.tm, self.ts)
    }
}