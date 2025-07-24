use crate::memory::Bus;
use crate::ppu::registers::PpuRegisters;
use crate::ppu::renderer::Renderer;
use crate::ppu::memory::{Vram, Cgram, Oam};
use crate::ppu::backgrounds::BackgroundRenderer;
use crate::ppu::sprites::SpriteRenderer;
use crate::ppu::scrolling::ScrollingEngine;
use crate::ppu::mode7::Mode7Renderer;
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
    bg_renderer: BackgroundRenderer,
    sprite_renderer: SpriteRenderer,
    scrolling: ScrollingEngine,
    mode7: Mode7Renderer,
    
    // Memory
    vram: Vram,
    cgram: Cgram,
    oam: Oam,
    
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
    
    // VRAM write buffer (for 16-bit writes)
    vram_latch: u8,
    vram_first_write: bool,
    
    // Temporary scanline buffer for compositing
    scanline_buffer: Vec<u8>,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            registers: PpuRegisters::new(),
            _renderer: Renderer::new(),
            bg_renderer: BackgroundRenderer::new(),
            sprite_renderer: SpriteRenderer::new(),
            scrolling: ScrollingEngine::new(),
            mode7: Mode7Renderer::new(),
            vram: Vram::new(),
            cgram: Cgram::new(),
            oam: Oam::new(),
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
            vram_latch: 0,
            vram_first_write: true,
            scanline_buffer: vec![0; 256 * 4],
        }
    }

    pub fn reset(&mut self) {
        self.registers = PpuRegisters::new();
        self.vram.reset();
        self.cgram.reset();
        self.oam.reset();
        self.dot = 0;
        self.scanline = 0;
        self.frame = 0;
        self.nmi_pending = false;
        self.irq_pending = false;
        self.h_counter = 0;
        self.v_counter = 0;
        self.latch_h = false;
        self.latch_v = false;
        self.vram_latch = 0;
        self.vram_first_write = true;
        
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
        
        // Check if we're in Mode 7
        let bg_mode = self.registers.get_bg_mode();
        
        if bg_mode == 7 {
            // Mode 7 rendering
            self.mode7.render_scanline(
                &self.vram,
                &self.cgram,
                &self.registers,
                self.scanline,
                &mut self.scanline_buffer,
            );
            
            // Check for Mode 7 EXTBG (BG2)
            if self.mode7.is_extbg_enabled(&self.registers) {
                let mut extbg_buffer = vec![0u8; SCREEN_WIDTH * 4];
                self.mode7.render_extbg_scanline(
                    &self.vram,
                    &self.cgram,
                    &self.registers,
                    self.scanline,
                    &mut extbg_buffer,
                );
                
                // Composite EXTBG onto main buffer
                for x in 0..SCREEN_WIDTH {
                    let offset = x * 4;
                    if extbg_buffer[offset + 3] != 0 {
                        self.scanline_buffer[offset] = extbg_buffer[offset];
                        self.scanline_buffer[offset + 1] = extbg_buffer[offset + 1];
                        self.scanline_buffer[offset + 2] = extbg_buffer[offset + 2];
                        self.scanline_buffer[offset + 3] = extbg_buffer[offset + 3];
                    }
                }
            }
        } else {
            // Normal background rendering
            let bg_buffer = self.bg_renderer.render_scanline(
                &self.vram,
                &self.cgram,
                &self.registers,
                self.scanline,
            );
            
            // Copy background to scanline buffer
            self.scanline_buffer.copy_from_slice(bg_buffer);
        }
        
        // Render sprites on top
        let main_screen = self.registers.get_main_screen_layers();
        if (main_screen & 0x10) != 0 { // Check if sprites are enabled on main screen
            self.sprite_renderer.render_scanline(
                &self.vram,
                &self.cgram,
                &self.oam,
                &self.registers,
                self.scanline,
                &mut self.scanline_buffer,
            );
        }
        
        // Copy final scanline to frame buffer with brightness adjustment
        let frame_offset = y * SCREEN_WIDTH * 4;
        let brightness = self.registers.get_brightness();
        let factor = brightness as f32 / 15.0;
        
        for x in 0..SCREEN_WIDTH {
            let src_offset = x * 4;
            let dst_offset = frame_offset + src_offset;
            
            self.frame_buffer[dst_offset] = (self.scanline_buffer[src_offset] as f32 * factor) as u8;
            self.frame_buffer[dst_offset + 1] = (self.scanline_buffer[src_offset + 1] as f32 * factor) as u8;
            self.frame_buffer[dst_offset + 2] = (self.scanline_buffer[src_offset + 2] as f32 * factor) as u8;
            self.frame_buffer[dst_offset + 3] = self.scanline_buffer[src_offset + 3];
        }
        
        // TODO: Implement proper layer priority compositing
        // TODO: Implement sub-screen and color math
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
        match address {
            // VRAM data read
            0x2139 => {
                let vram_addr = self.registers.get_vram_address();
                let value = if self.vram_first_write {
                    // Read low byte
                    self.vram.read(vram_addr)
                } else {
                    // Read high byte
                    self.vram.read(vram_addr.wrapping_add(1))
                };
                
                // Toggle first write flag
                self.vram_first_write = !self.vram_first_write;
                
                // Auto-increment based on VMAIN setting
                if (!self.vram_first_write && (self.registers.vmain & 0x80) == 0) ||
                   (self.vram_first_write && (self.registers.vmain & 0x80) != 0) {
                    self.auto_increment_vram();
                }
                
                value
            }
            
            // CGRAM data read
            0x213B => {
                let value = self.cgram.read(self.registers.cgadd);
                self.registers.cgadd = self.registers.cgadd.wrapping_add(1);
                value
            }
            
            // OAM data read
            0x2138 => {
                let address = self.registers.get_oam_address();
                let value = self.oam.read(address);
                
                // Auto-increment OAM address
                let new_address = (address + 1) & 0x3FF;
                self.registers.oamaddl = (new_address & 0xFF) as u8;
                self.registers.oamaddh = ((new_address >> 8) & 0x01) as u8 | (self.registers.oamaddh & 0x80);
                
                value
            }
            
            // Counter latch register
            0x2137 => {
                // Latch H/V counters on read
                self.latch_h = true;
                self.latch_v = true;
                self.registers.read(address)
            }
            
            // H counter data
            0x213C => {
                if self.latch_h {
                    self.latch_h = false;
                    (self.h_counter & 0xFF) as u8
                } else {
                    0
                }
            }
            0x213D => {
                if self.latch_h {
                    self.latch_h = false;
                    ((self.h_counter >> 8) & 0x01) as u8
                } else {
                    0
                }
            }
            
            // V counter data
            0x213E => {
                if self.latch_v {
                    self.latch_v = false;
                    (self.v_counter & 0xFF) as u8
                } else {
                    0
                }
            }
            0x213F => {
                if self.latch_v {
                    self.latch_v = false;
                    ((self.v_counter >> 8) & 0x01) as u8
                } else {
                    0
                }
            }
            
            // Default register read
            _ => self.registers.read(address),
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        self.registers.write(address, value);
        
        // Forward to scrolling engine for scroll/window registers
        self.scrolling.write_register(address, value);
        
        // Forward to Mode 7 renderer for Mode 7 registers
        self.mode7.write_register(address, value);
        
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
        
        if self.vram_first_write {
            // First write - store in latch
            self.vram_latch = value;
            self.vram_first_write = false;
        } else {
            // Second write - write both bytes
            self.vram.write16(address, (value as u16) << 8 | self.vram_latch as u16);
            self.vram_first_write = true;
            
            // Auto-increment based on VMAIN setting
            if (self.registers.vmain & 0x80) == 0 {
                self.auto_increment_vram();
            }
        }
        
        trace!("VRAM write low: ${:04X} = ${:02X}", address, value);
    }

    fn write_vram_high(&mut self, value: u8) {
        let address = self.registers.get_vram_address();
        
        if self.vram_first_write {
            // First write - store in latch
            self.vram_latch = value;
            self.vram_first_write = false;
        } else {
            // Second write - write both bytes
            self.vram.write16(address, (self.vram_latch as u16) << 8 | value as u16);
            self.vram_first_write = true;
            
            // Auto-increment based on VMAIN setting
            if (self.registers.vmain & 0x80) != 0 {
                self.auto_increment_vram();
            }
        }
        
        trace!("VRAM write high: ${:04X} = ${:02X}", address, value);
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
        self.cgram.write(self.registers.cgadd, value);
        trace!("CGRAM write: ${:02X} = ${:02X}", self.registers.cgadd, value);
        
        // Auto-increment CGRAM address
        self.registers.cgadd = self.registers.cgadd.wrapping_add(1);
    }

    fn write_oam(&mut self, value: u8) {
        let address = self.registers.get_oam_address();
        self.oam.write(address, value);
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