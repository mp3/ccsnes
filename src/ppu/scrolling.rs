use super::registers::PpuRegisters;

/// Handles PPU scrolling and window functionality
pub struct ScrollingEngine {
    // BG scroll positions (written to during HBlank/VBlank)
    bg1_hscroll: u16,
    bg1_vscroll: u16,
    bg2_hscroll: u16,
    bg2_vscroll: u16,
    bg3_hscroll: u16,
    bg3_vscroll: u16,
    bg4_hscroll: u16,
    bg4_vscroll: u16,
    
    // Mode 7 scroll positions
    m7_hscroll: u16,
    m7_vscroll: u16,
    
    // Window positions
    window1_left: u8,
    window1_right: u8,
    window2_left: u8,
    window2_right: u8,
    
    // Window masks for BGs and OBJ
    window_mask_bg: [u8; 4],
    window_mask_obj: u8,
    window_mask_color: u8,
    
    // Window logic operations
    window_logic_bg: [u8; 4],
    window_logic_obj: u8,
    window_logic_color: u8,
    
    // Main/sub screen designation
    main_screen_designation: u8,
    sub_screen_designation: u8,
    
    // Color math control
    color_math_control: u8,
    fixed_color: u16,
    
    // Internal state
    prev_write: u8,
    write_toggle: bool,
}

impl ScrollingEngine {
    pub fn new() -> Self {
        Self {
            bg1_hscroll: 0,
            bg1_vscroll: 0,
            bg2_hscroll: 0,
            bg2_vscroll: 0,
            bg3_hscroll: 0,
            bg3_vscroll: 0,
            bg4_hscroll: 0,
            bg4_vscroll: 0,
            m7_hscroll: 0,
            m7_vscroll: 0,
            window1_left: 0,
            window1_right: 0,
            window2_left: 0,
            window2_right: 0,
            window_mask_bg: [0; 4],
            window_mask_obj: 0,
            window_mask_color: 0,
            window_logic_bg: [0; 4],
            window_logic_obj: 0,
            window_logic_color: 0,
            main_screen_designation: 0,
            sub_screen_designation: 0,
            color_math_control: 0,
            fixed_color: 0,
            prev_write: 0,
            write_toggle: false,
        }
    }
    
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            // BG scroll registers
            0x210D => {
                // BG1HOFS - BG1 Horizontal Scroll
                self.bg1_hscroll = (self.bg1_hscroll & 0xFF00) | (value as u16);
                self.bg1_hscroll |= ((self.prev_write & 0x03) as u16) << 8;
                self.prev_write = value;
            }
            0x210E => {
                // BG1VOFS - BG1 Vertical Scroll
                self.bg1_vscroll = (self.bg1_vscroll & 0xFF00) | (value as u16);
                self.bg1_vscroll |= ((self.prev_write & 0x03) as u16) << 8;
                self.prev_write = value;
            }
            0x210F => {
                // BG2HOFS - BG2 Horizontal Scroll
                self.bg2_hscroll = (self.bg2_hscroll & 0xFF00) | (value as u16);
                self.bg2_hscroll |= ((self.prev_write & 0x03) as u16) << 8;
                self.prev_write = value;
            }
            0x2110 => {
                // BG2VOFS - BG2 Vertical Scroll
                self.bg2_vscroll = (self.bg2_vscroll & 0xFF00) | (value as u16);
                self.bg2_vscroll |= ((self.prev_write & 0x03) as u16) << 8;
                self.prev_write = value;
            }
            0x2111 => {
                // BG3HOFS - BG3 Horizontal Scroll
                self.bg3_hscroll = (self.bg3_hscroll & 0xFF00) | (value as u16);
                self.bg3_hscroll |= ((self.prev_write & 0x03) as u16) << 8;
                self.prev_write = value;
            }
            0x2112 => {
                // BG3VOFS - BG3 Vertical Scroll
                self.bg3_vscroll = (self.bg3_vscroll & 0xFF00) | (value as u16);
                self.bg3_vscroll |= ((self.prev_write & 0x03) as u16) << 8;
                self.prev_write = value;
            }
            0x2113 => {
                // BG4HOFS - BG4 Horizontal Scroll
                self.bg4_hscroll = (self.bg4_hscroll & 0xFF00) | (value as u16);
                self.bg4_hscroll |= ((self.prev_write & 0x03) as u16) << 8;
                self.prev_write = value;
            }
            0x2114 => {
                // BG4VOFS - BG4 Vertical Scroll
                self.bg4_vscroll = (self.bg4_vscroll & 0xFF00) | (value as u16);
                self.bg4_vscroll |= ((self.prev_write & 0x01) as u16) << 8;
                self.prev_write = value;
            }
            
            // Mode 7 scroll registers
            0x211F => {
                // M7HOFS - Mode 7 Horizontal Scroll
                self.m7_hscroll = (self.m7_hscroll & 0xFF00) | (value as u16);
                self.m7_hscroll |= ((self.prev_write & 0x1F) as u16) << 8;
                self.prev_write = value;
            }
            0x2120 => {
                // M7VOFS - Mode 7 Vertical Scroll
                self.m7_vscroll = (self.m7_vscroll & 0xFF00) | (value as u16);
                self.m7_vscroll |= ((self.prev_write & 0x1F) as u16) << 8;
                self.prev_write = value;
            }
            
            // Window position registers
            0x2126 => {
                // WH0 - Window 1 Left Position
                self.window1_left = value;
            }
            0x2127 => {
                // WH1 - Window 1 Right Position
                self.window1_right = value;
            }
            0x2128 => {
                // WH2 - Window 2 Left Position
                self.window2_left = value;
            }
            0x2129 => {
                // WH3 - Window 2 Right Position
                self.window2_right = value;
            }
            
            // Window mask settings
            0x212A => {
                // WBGLOG - Window BG Logic
                self.window_logic_bg[0] = value & 0x03;
                self.window_logic_bg[1] = (value >> 2) & 0x03;
                self.window_logic_bg[2] = (value >> 4) & 0x03;
                self.window_logic_bg[3] = (value >> 6) & 0x03;
            }
            0x212B => {
                // WOBJLOG - Window OBJ/Color Logic
                self.window_logic_obj = value & 0x03;
                self.window_logic_color = (value >> 2) & 0x03;
            }
            0x212C => {
                // TM - Main Screen Designation
                self.main_screen_designation = value & 0x1F;
            }
            0x212D => {
                // TS - Sub Screen Designation
                self.sub_screen_designation = value & 0x1F;
            }
            0x212E => {
                // TMW - Window Mask Main Screen
                self.window_mask_bg[0] = value & 0x01;
                self.window_mask_bg[1] = (value >> 1) & 0x01;
                self.window_mask_bg[2] = (value >> 2) & 0x01;
                self.window_mask_bg[3] = (value >> 3) & 0x01;
                self.window_mask_obj = (value >> 4) & 0x01;
            }
            0x212F => {
                // TSW - Window Mask Sub Screen
                // Similar to TMW but for sub screen
            }
            0x2130 => {
                // CGWSEL - Color Math Control A
                self.color_math_control = value;
            }
            0x2131 => {
                // CGADSUB - Color Math Control B
                // Color math designation
            }
            0x2132 => {
                // COLDATA - Fixed Color Data
                if value & 0x20 != 0 {
                    // Red
                    self.fixed_color = (self.fixed_color & 0xFFE0) | ((value & 0x1F) as u16);
                }
                if value & 0x40 != 0 {
                    // Green
                    self.fixed_color = (self.fixed_color & 0xFC1F) | (((value & 0x1F) as u16) << 5);
                }
                if value & 0x80 != 0 {
                    // Blue
                    self.fixed_color = (self.fixed_color & 0x03FF) | (((value & 0x1F) as u16) << 10);
                }
            }
            _ => {}
        }
    }
    
    pub fn get_bg_scroll(&self, bg_num: u8) -> (u16, u16) {
        match bg_num {
            1 => (self.bg1_hscroll, self.bg1_vscroll),
            2 => (self.bg2_hscroll, self.bg2_vscroll),
            3 => (self.bg3_hscroll, self.bg3_vscroll),
            4 => (self.bg4_hscroll, self.bg4_vscroll),
            _ => (0, 0),
        }
    }
    
    pub fn get_mode7_scroll(&self) -> (u16, u16) {
        (self.m7_hscroll, self.m7_vscroll)
    }
    
    pub fn is_in_window(&self, x: u16, window_num: u8) -> bool {
        match window_num {
            1 => x >= self.window1_left as u16 && x <= self.window1_right as u16,
            2 => x >= self.window2_left as u16 && x <= self.window2_right as u16,
            _ => false,
        }
    }
    
    pub fn apply_window_logic(&self, bg_num: u8, x: u16) -> bool {
        let window_mask = self.window_mask_bg[bg_num as usize - 1];
        if window_mask == 0 {
            return true; // No window masking
        }
        
        let in_window1 = self.is_in_window(x, 1);
        let in_window2 = self.is_in_window(x, 2);
        let logic = self.window_logic_bg[bg_num as usize - 1];
        
        match logic {
            0 => in_window1 || in_window2,    // OR
            1 => in_window1 && in_window2,    // AND
            2 => in_window1 != in_window2,    // XOR
            3 => !(in_window1 != in_window2), // XNOR
            _ => true,
        }
    }
    
    pub fn is_bg_on_main_screen(&self, bg_num: u8) -> bool {
        (self.main_screen_designation & (1 << (bg_num - 1))) != 0
    }
    
    pub fn is_bg_on_sub_screen(&self, bg_num: u8) -> bool {
        (self.sub_screen_designation & (1 << (bg_num - 1))) != 0
    }
    
    pub fn is_obj_on_main_screen(&self) -> bool {
        (self.main_screen_designation & 0x10) != 0
    }
    
    pub fn is_obj_on_sub_screen(&self) -> bool {
        (self.sub_screen_designation & 0x10) != 0
    }
}