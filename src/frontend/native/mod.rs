pub mod video;
pub mod audio;

use crate::emulator::Emulator;
use crate::{Result, EmulatorError};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct NativeFrontend {
    scale: u32,
    debug: bool,
}

impl NativeFrontend {
    pub fn new(scale: u32, debug: bool) -> Result<Self> {
        Ok(Self { scale, debug })
    }

    pub fn run(&mut self, mut emulator: Emulator) -> Result<()> {
        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new()
            .with_title("CCSNES - Super Nintendo Emulator")
            .with_inner_size(winit::dpi::LogicalSize::new(
                256 * self.scale,
                224 * self.scale,
            ))
            .with_resizable(false)
            .build(&event_loop)
            .map_err(|e| EmulatorError::VideoError(format!("Failed to create window: {}", e)))?;

        // TODO: Initialize video and audio systems
        // let mut video = video::VideoRenderer::new(&window, self.scale)?;
        // let mut audio = audio::AudioPlayer::new()?;

        event_loop.run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => elwt.exit(),
                
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { event, .. },
                    ..
                } => {
                    // TODO: Handle keyboard input
                    println!("Key event: {:?}", event);
                }
                
                Event::AboutToWait => {
                    // Run one frame of emulation
                    if let Err(e) = emulator.step_frame() {
                        eprintln!("Emulation error: {}", e);
                        elwt.exit();
                    }
                    
                    // TODO: Render video frame
                    // video.render(emulator.get_video_buffer());
                    
                    // TODO: Play audio samples
                    // let samples = emulator.get_audio_samples();
                    // audio.queue_samples(&samples);
                    
                    window.request_redraw();
                }
                
                Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                    // TODO: Present the rendered frame
                }
                
                _ => {}
            }
        }).map_err(|e| EmulatorError::VideoError(format!("Event loop error: {:?}", e)))?;
        Ok(())
    }
}