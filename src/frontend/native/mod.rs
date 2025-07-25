pub mod video;
pub mod audio;

use crate::emulator::Emulator;
use crate::{Result, EmulatorError};
use winit::{
    event::{Event, WindowEvent, KeyEvent, ElementState},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{PhysicalKey, KeyCode},
    window::WindowBuilder,
};
use std::time::{Instant, Duration};
use pollster::FutureExt;

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
        
        // Initialize video and audio systems
        let mut video = video::VideoRenderer::new(&window, self.scale).block_on()?;
        let mut audio = audio::AudioPlayer::new()?;
        
        // Frame timing
        let mut last_frame = Instant::now();
        let frame_duration = Duration::from_secs_f64(1.0 / 60.0);
        let mut fps_counter = 0;
        let mut fps_timer = Instant::now();
        
        // Controller state
        let mut controller_state = 0u16;
        
        event_loop.run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    
                    WindowEvent::Resized(_) => {
                        // Surface is recreated each frame, so no need to handle resize
                    }
                    
                    WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(keycode), state, .. }, .. } => {
                        // Map keyboard to SNES controller
                        let button = match keycode {
                            KeyCode::KeyZ => Some(0x80),    // A
                            KeyCode::KeyX => Some(0x8000),  // B  
                            KeyCode::KeyA => Some(0x40),    // X
                            KeyCode::KeyS => Some(0x4000),  // Y
                            KeyCode::KeyQ => Some(0x20),    // L
                            KeyCode::KeyW => Some(0x10),    // R
                            KeyCode::Enter => Some(0x1000), // Start
                            KeyCode::ShiftRight => Some(0x2000), // Select
                            KeyCode::ArrowUp => Some(0x800),     // Up
                            KeyCode::ArrowDown => Some(0x400),   // Down
                            KeyCode::ArrowLeft => Some(0x200),   // Left
                            KeyCode::ArrowRight => Some(0x100),  // Right
                            _ => None,
                        };
                        
                        if let Some(button) = button {
                            match state {
                                ElementState::Pressed => controller_state |= button,
                                ElementState::Released => controller_state &= !button,
                            }
                            emulator.set_controller_input(0, controller_state);
                        }
                    }
                    
                    WindowEvent::RedrawRequested => {
                        // Present the rendered frame
                        if let Err(e) = video.render(&window) {
                            eprintln!("Render error: {}", e);
                        }
                    }
                    
                    _ => {}
                },
                
                Event::AboutToWait => {
                    // Check if enough time has passed for next frame
                    let now = Instant::now();
                    if now.duration_since(last_frame) >= frame_duration {
                        last_frame = now;
                        
                        // Run one frame of emulation
                        if let Err(e) = emulator.step_frame() {
                            eprintln!("Emulation error: {}", e);
                            elwt.exit();
                            return;
                        }
                        
                        // Update video with frame buffer
                        video.update_frame(emulator.get_video_buffer());
                        
                        // Queue audio samples
                        let samples = emulator.get_audio_samples();
                        if !samples.is_empty() {
                            audio.queue_samples(&samples);
                        }
                        
                        // Request redraw
                        window.request_redraw();
                        
                        // FPS counter
                        fps_counter += 1;
                        if fps_timer.elapsed() >= Duration::from_secs(1) {
                            if self.debug {
                                println!("FPS: {}", fps_counter);
                            }
                            fps_counter = 0;
                            fps_timer = Instant::now();
                        }
                    }
                }
                
                _ => {}
            }
        }).map_err(|e| EmulatorError::VideoError(format!("Event loop error: {:?}", e)))?;
        Ok(())
    }
}