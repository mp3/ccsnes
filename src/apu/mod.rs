pub mod spc700;
pub mod dsp;

use self::spc700::Spc700;
use self::dsp::Dsp;

pub struct Apu {
    spc700: Spc700,
    dsp: Dsp,
    audio_buffer: Vec<f32>,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            spc700: Spc700::new(),
            dsp: Dsp::new(),
            audio_buffer: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.spc700.reset();
        self.dsp.reset();
        self.audio_buffer.clear();
    }

    pub fn step(&mut self) {
        // TODO: Implement APU step logic
        self.spc700.step();
        
        // Generate audio samples
        let sample = self.dsp.step();
        self.audio_buffer.push(sample);
        
        // Keep buffer from growing too large
        if self.audio_buffer.len() > 4096 {
            self.audio_buffer.drain(0..2048);
        }
    }

    pub fn get_audio_samples(&mut self) -> Vec<f32> {
        let samples = self.audio_buffer.clone();
        self.audio_buffer.clear();
        samples
    }
}