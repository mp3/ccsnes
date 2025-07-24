use crate::memory::Bus;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 224;
const FRAMEBUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT * 4; // RGBA

pub struct Ppu {
    cycle: u32,
    scanline: u16,
    frame_buffer: Vec<u8>,
    nmi_pending: bool,
    irq_pending: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            scanline: 0,
            frame_buffer: vec![0; FRAMEBUFFER_SIZE],
            nmi_pending: false,
            irq_pending: false,
        }
    }

    pub fn reset(&mut self) {
        self.cycle = 0;
        self.scanline = 0;
        self.nmi_pending = false;
        self.irq_pending = false;
        
        // Clear frame buffer to black
        for pixel in self.frame_buffer.chunks_mut(4) {
            pixel[0] = 0; // R
            pixel[1] = 0; // G
            pixel[2] = 0; // B
            pixel[3] = 255; // A
        }
    }

    pub fn step(&mut self, _bus: &mut Bus) {
        self.cycle += 1;

        // NTSC timing: 341 cycles per scanline
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;

            // NTSC: 262 scanlines per frame
            if self.scanline >= 262 {
                self.scanline = 0;
                // Frame complete
            }

            // V-Blank starts at scanline 225
            if self.scanline == 225 {
                self.nmi_pending = true;
            }
        }

        // TODO: Implement actual PPU rendering logic
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
}