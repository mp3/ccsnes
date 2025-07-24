pub mod spc700;
pub mod dsp;
mod spc700_instructions;

use self::spc700::Spc700;
use self::dsp::Dsp;

pub struct Apu {
    spc700: Spc700,
    dsp: Dsp,
    audio_buffer: Vec<f32>,
    dsp_address: u8,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            spc700: Spc700::new(),
            dsp: Dsp::new(),
            audio_buffer: Vec::new(),
            dsp_address: 0,
        }
    }

    pub fn reset(&mut self) {
        self.spc700.reset();
        self.dsp.reset();
        self.audio_buffer.clear();
        self.dsp_address = 0;
    }

    pub fn step(&mut self) {
        // Connect SPC700 to DSP through I/O ports
        self.connect_dsp();
        
        // Execute one SPC700 instruction
        self.spc700.step();
        
        // Generate audio samples (32kHz output rate)
        // The APU runs at 1.024 MHz, so we generate a sample every 32 cycles
        if self.spc700.cycles % 32 == 0 {
            let sample = self.dsp.step();
            self.audio_buffer.push(sample);
            
            // Keep buffer from growing too large
            if self.audio_buffer.len() > 4096 {
                self.audio_buffer.drain(0..2048);
            }
        }
    }
    
    fn connect_dsp(&mut self) {
        // Handle DSP register access through SPC700 I/O ports
        let dsp_addr_write = self.spc700.read8(0x00F2);
        let dsp_data_write = self.spc700.read8(0x00F3);
        
        // Update DSP address
        if dsp_addr_write != self.dsp_address {
            self.dsp_address = dsp_addr_write;
        }
        
        // Handle DSP data write
        if self.spc700.read8(0x00F3) != dsp_data_write {
            self.dsp.write_register(self.dsp_address, dsp_data_write);
        }
        
        // Handle DSP data read
        let dsp_data = self.dsp.read_register(self.dsp_address);
        self.spc700.write8(0x00F3, dsp_data);
    }

    pub fn get_audio_samples(&mut self) -> Vec<f32> {
        let samples = self.audio_buffer.clone();
        self.audio_buffer.clear();
        samples
    }
    
    // Communication ports with main CPU
    pub fn read_port(&self, port: usize) -> u8 {
        self.spc700.read_port(port)
    }
    
    pub fn write_port(&mut self, port: usize, value: u8) {
        self.spc700.write_port(port, value)
    }
}