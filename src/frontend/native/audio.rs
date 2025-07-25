use crate::{Result, EmulatorError};
use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Stream};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

const SAMPLE_RATE: u32 = 32000;
const BUFFER_SIZE: usize = 2048;

pub struct AudioPlayer {
    stream: Stream,
    sample_buffer: Arc<Mutex<VecDeque<f32>>>,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        
        let device = host.default_output_device()
            .ok_or_else(|| EmulatorError::AudioError("No output device available".to_string()))?;
        
        let config = device.default_output_config()
            .map_err(|e| EmulatorError::AudioError(format!("Failed to get default config: {}", e)))?;
        
        let sample_buffer = Arc::new(Mutex::new(VecDeque::with_capacity(BUFFER_SIZE * 4)));
        let buffer_clone = Arc::clone(&sample_buffer);
        
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => Self::build_stream::<f32>(&device, &config.into(), buffer_clone),
            cpal::SampleFormat::I16 => Self::build_stream::<i16>(&device, &config.into(), buffer_clone),
            cpal::SampleFormat::U16 => Self::build_stream::<u16>(&device, &config.into(), buffer_clone),
            sample_format => return Err(EmulatorError::AudioError(format!("Unsupported sample format: {:?}", sample_format))),
        }?;
        
        stream.play()
            .map_err(|e| EmulatorError::AudioError(format!("Failed to play stream: {}", e)))?;
        
        Ok(Self {
            stream,
            sample_buffer,
        })
    }
    
    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        buffer: Arc<Mutex<VecDeque<f32>>>,
    ) -> Result<Stream>
    where
        T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
    {
        let channels = config.channels as usize;
        
        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                let mut buffer = buffer.lock().unwrap();
                
                for frame in data.chunks_mut(channels) {
                    if buffer.len() >= channels {
                        // We have stereo samples
                        for (i, sample) in frame.iter_mut().enumerate() {
                            if i < channels && !buffer.is_empty() {
                                let value = buffer.pop_front().unwrap_or(0.0);
                                *sample = T::from_sample(value);
                            } else {
                                *sample = T::from_sample(0.0);
                            }
                        }
                    } else {
                        // Not enough samples, output silence
                        for sample in frame.iter_mut() {
                            *sample = T::from_sample(0.0);
                        }
                    }
                }
            },
            move |err| {
                eprintln!("Audio stream error: {}", err);
            },
            None,
        ).map_err(|e| EmulatorError::AudioError(format!("Failed to build output stream: {}", e)))?;
        
        Ok(stream)
    }
    
    pub fn queue_samples(&mut self, samples: &[f32]) {
        let mut buffer = self.sample_buffer.lock().unwrap();
        
        // Don't let the buffer grow too large
        let max_size = BUFFER_SIZE * 8;
        if buffer.len() + samples.len() > max_size {
            // Drop old samples if buffer is getting too full
            let to_drop = (buffer.len() + samples.len()) - max_size;
            for _ in 0..to_drop {
                buffer.pop_front();
            }
        }
        
        // Queue new samples
        for &sample in samples {
            buffer.push_back(sample);
        }
    }
    
    pub fn clear_buffer(&mut self) {
        let mut buffer = self.sample_buffer.lock().unwrap();
        buffer.clear();
    }
    
    pub fn get_buffer_size(&self) -> usize {
        self.sample_buffer.lock().unwrap().len()
    }
}