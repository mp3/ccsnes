use crate::memory::Bus;
use crate::ppu::registers::PpuRegisters;
use crate::ppu::renderer::Renderer;
use log::trace;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 224;
const FRAMEBUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT * 4; // RGBA

// PPU timing constants
const DOTS_PER_SCANLINE: u32 = 341;
const SCANLINES_PER_FRAME: u16 = 262;
const VBLANK_START_SCANLINE: u16 = 225;

pub struct Ppu {
    // PPU state
    pub registers: PpuRegisters,
    _renderer: Renderer,
    
    // Timing
    dot: u32,           // Current dot (0-340)
    scanline: u16,      // Current scanline (0-261)
    frame: u64,         // Frame counter
    
    // Frame buffer
    frame_buffer: Vec<u8>,
    
    // Interrupt flags
    nmi_pending: bool,
    irq_pending: bool,
    
    // H/V counters for latching
    h_counter: u16,
    v_counter: u16,
    latch_h: bool,
    latch_v: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            registers: PpuRegisters::new(),
            _renderer: Renderer::new(),
            dot: 0,
            scanline: 0,
            frame: 0,
            frame_buffer: vec![0; FRAMEBUFFER_SIZE],
            nmi_pending: false,
            irq_pending: false,
            h_counter: 0,
            v_counter: 0,
            latch_h: false,
            latch_v: false,
        }
    }

    pub fn reset(&mut self) {
        self.registers = PpuRegisters::new();
        self.dot = 0;
        self.scanline = 0;
        self.frame = 0;
        self.nmi_pending = false;
        self.irq_pending = false;
        self.h_counter = 0;
        self.v_counter = 0;
        self.latch_h = false;
        self.latch_v = false;
        
        // Clear frame buffer to black
        for pixel in self.frame_buffer.chunks_mut(4) {
            pixel[0] = 0;   // R
            pixel[1] = 0;   // G
            pixel[2] = 0;   // B
            pixel[3] = 255; // A
        }
    }

    pub fn step(&mut self, bus: &mut Bus) {
        self.dot += 1;

        // Update H/V counters
        self.h_counter = self.dot as u16;
        self.v_counter = self.scanline;

        // Check for H-Blank (dot 274)
        if self.dot == 274 {
            // H-Blank processing
        }

        // End of scanline
        if self.dot >= DOTS_PER_SCANLINE {
            self.dot = 0;
            self.scanline += 1;
            
            // Check if we're in visible range
            if self.scanline < VBLANK_START_SCANLINE {
                self.render_scanline(bus);
            }
            
            // V-Blank start
            if self.scanline == VBLANK_START_SCANLINE {
                self.enter_vblank();
            }
            
            // End of frame
            if self.scanline >= SCANLINES_PER_FRAME {
                self.scanline = 0;
                self.frame += 1;
                self.exit_vblank();
            }
        }
    }

    fn render_scanline(&mut self, _bus: &mut Bus) {
        // Skip rendering if screen is blanked
        if self.registers.is_screen_blanked() {
            return;
        }
        
        let y = self.scanline as usize;
        if y >= SCREEN_HEIGHT {
            return;
        }
        
        // Get current BG mode
        let bg_mode = self.registers.get_bg_mode();
        let main_screen = self.registers.get_main_screen_layers();
        
        // Simple test pattern for now
        // TODO: Implement actual background and sprite rendering
        for x in 0..SCREEN_WIDTH {
            let pixel_offset = (y * SCREEN_WIDTH + x) * 4;
            
            // Generate a test pattern based on BG mode
            let (r, g, b) = match bg_mode {
                0 => {
                    // Mode 0: 4 backgrounds, 4 colors each
                    let color = ((x / 8) + (y / 8)) & 0x03;
                    match color {
                        0 => (64, 64, 64),
                        1 => (128, 0, 0),
                        2 => (0, 128, 0),
                        3 => (0, 0, 128),
                        _ => (0, 0, 0),
                    }
                }
                1 => {
                    // Mode 1: 3 backgrounds
                    if main_screen & 0x01 != 0 {
                        // BG1 enabled - draw gradient
                        (x as u8, y as u8, 128)
                    } else {
                        (0, 0, 0)
                    }
                }
                _ => {
                    // Other modes - simple pattern
                    let checker = ((x / 16) + (y / 16)) & 1;
                    if checker == 1 {
                        (192, 192, 192)
                    } else {
                        (64, 64, 64)
                    }
                }
            };
            
            // Apply brightness
            let brightness = self.registers.get_brightness();
            let factor = brightness as f32 / 15.0;
            
            self.frame_buffer[pixel_offset] = (r as f32 * factor) as u8;
            self.frame_buffer[pixel_offset + 1] = (g as f32 * factor) as u8;
            self.frame_buffer[pixel_offset + 2] = (b as f32 * factor) as u8;
            self.frame_buffer[pixel_offset + 3] = 255;
        }
    }

    fn enter_vblank(&mut self) {
        trace!("PPU: Entering V-Blank at frame {}", self.frame);
        
        // Set V-Blank flag and trigger NMI if enabled
        if !self.registers.is_screen_blanked() {
            self.nmi_pending = true;
        }
    }

    fn exit_vblank(&mut self) {
        trace!("PPU: Exiting V-Blank");
        // V-Blank period is over
    }

    pub fn get_frame_buffer(&self) -> &[u8] {
        &self.frame_buffer
    }

    pub fn nmi_pending(&mut self) -> bool {
        if self.nmi_pending {
            self.nmi_pending = false;
            true
        } else {
            false
        }
    }

    pub fn irq_pending(&mut self) -> bool {
        if self.irq_pending {
            self.irq_pending = false;
            true
        } else {
            false
        }
    }

    // PPU register access
    pub fn read_register(&mut self, address: u16) -> u8 {
        let value = self.registers.read(address);
        
        // Special handling for counter latch register
        if address == 0x2137 {
            // Latch H/V counters on read
            if self.latch_h {
                // Return latched H counter value
            }
            if self.latch_v {
                // Return latched V counter value
            }
        }
        
        value
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        self.registers.write(address, value);
        
        // Handle VRAM writes
        match address {
            0x2118 => {
                // VRAM data write (low byte)
                self.write_vram_low(value);
            }
            0x2119 => {
                // VRAM data write (high byte)
                self.write_vram_high(value);
            }
            0x2122 => {
                // CGRAM data write
                self.write_cgram(value);
            }
            0x2104 => {
                // OAM data write
                self.write_oam(value);
            }
            _ => {}
        }
    }

    fn write_vram_low(&mut self, value: u8) {
        let address = self.registers.get_vram_address();
        // TODO: Actually write to VRAM
        trace!("VRAM write low: ${:04X} = ${:02X}", address, value);
        
        // Auto-increment based on VMAIN setting
        if (self.registers.vmain & 0x80) == 0 {
            self.auto_increment_vram();
        }
    }

    fn write_vram_high(&mut self, value: u8) {
        let address = self.registers.get_vram_address();
        // TODO: Actually write to VRAM
        trace!("VRAM write high: ${:04X} = ${:02X}", address, value);
        
        // Auto-increment based on VMAIN setting
        if (self.registers.vmain & 0x80) != 0 {
            self.auto_increment_vram();
        }
    }

    fn auto_increment_vram(&mut self) {
        let increment = match self.registers.vmain & 0x03 {
            0 => 1,    // Increment by 1
            1 => 32,   // Increment by 32
            2 => 128,  // Increment by 128
            3 => 128,  // Increment by 128
            _ => 1,
        };
        
        let new_address = (self.registers.get_vram_address() + increment) & 0x7FFF;
        self.registers.set_vram_address(new_address);
    }

    fn write_cgram(&mut self, value: u8) {
        // TODO: Implement CGRAM writes
        trace!("CGRAM write: ${:02X} = ${:02X}", self.registers.cgadd, value);
        
        // Auto-increment CGRAM address
        self.registers.cgadd = self.registers.cgadd.wrapping_add(1);
    }

    fn write_oam(&mut self, value: u8) {
        let address = self.registers.get_oam_address();
        // TODO: Actually write to OAM
        trace!("OAM write: ${:04X} = ${:02X}", address, value);
        
        // Auto-increment OAM address
        let new_address = (address + 1) & 0x3FF;
        self.registers.oamaddl = (new_address & 0xFF) as u8;
        self.registers.oamaddh = ((new_address >> 8) & 0x01) as u8 | (self.registers.oamaddh & 0x80);
    }

    pub fn get_current_scanline(&self) -> u16 {
        self.scanline
    }

    pub fn get_current_dot(&self) -> u32 {
        self.dot
    }

    pub fn is_in_vblank(&self) -> bool {
        self.scanline >= VBLANK_START_SCANLINE
    }

    pub fn get_frame_count(&self) -> u64 {
        self.frame
    }
}