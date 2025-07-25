use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, HtmlCanvasElement, ImageData, KeyboardEvent};
use std::cell::RefCell;
use std::rc::Rc;

use crate::emulator::Emulator;
use crate::input::controller::{
    BUTTON_A, BUTTON_B, BUTTON_X, BUTTON_Y,
    BUTTON_L, BUTTON_R, BUTTON_START, BUTTON_SELECT,
    BUTTON_UP, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT
};

#[wasm_bindgen]
pub struct WasmEmulator {
    emulator: Rc<RefCell<Emulator>>,
    ctx: web_sys::CanvasRenderingContext2d,
    audio_ctx: Option<web_sys::AudioContext>,
    frame_buffer: Vec<u8>,
    controller_state: u16,
}

#[wasm_bindgen]
impl WasmEmulator {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<WasmEmulator, JsValue> {
        // Set panic hook for better error messages
        console_error_panic_hook::set_once();
        
        // Get canvas element
        let document = web_sys::window()
            .ok_or("No window")?
            .document()
            .ok_or("No document")?;
            
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or("Canvas not found")?
            .dyn_into::<HtmlCanvasElement>()?;
            
        // Get 2D context
        let ctx = canvas
            .get_context("2d")?
            .ok_or("Failed to get 2D context")?
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
            
        // Set canvas size
        canvas.set_width(256);
        canvas.set_height(224);
        
        // Create emulator
        let emulator = Emulator::new()
            .map_err(|e| JsValue::from_str(&format!("Failed to create emulator: {}", e)))?;
        let emulator = Rc::new(RefCell::new(emulator));
        
        // Try to create audio context (might fail due to browser restrictions)
        let audio_ctx = web_sys::AudioContext::new().ok();
        
        Ok(WasmEmulator {
            emulator,
            ctx,
            audio_ctx,
            frame_buffer: vec![0; 256 * 224 * 4],
            controller_state: 0,
        })
    }
    
    #[wasm_bindgen]
    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<String, JsValue> {
        self.emulator.borrow_mut()
            .load_rom(rom_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to load ROM: {}", e)))?;
            
        let title = self.emulator.borrow().get_rom_info()
            .map(|info| info.title.clone())
            .unwrap_or_else(|| "Unknown".to_string());
            
        console::log_1(&format!("Loaded ROM: {}", title).into());
        Ok(title)
    }
    
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        let _ = self.emulator.borrow_mut().reset();
        console::log_1(&"Emulator reset".into());
    }
    
    #[wasm_bindgen]
    pub fn run_frame(&mut self) -> Result<(), JsValue> {
        // Run one frame
        self.emulator.borrow_mut().step_frame()
            .map_err(|e| JsValue::from_str(&format!("Emulation error: {}", e)))?;
        
        // Get frame buffer and render
        self.render_frame()?;
        
        // Process audio if available
        if let Some(ref audio_ctx) = self.audio_ctx {
            self.process_audio(audio_ctx)?;
        }
        
        Ok(())
    }
    
    #[wasm_bindgen]
    pub fn handle_key_down(&mut self, event: &KeyboardEvent) {
        let button = match event.key().as_str() {
            "ArrowUp" => Some(BUTTON_UP),
            "ArrowDown" => Some(BUTTON_DOWN),
            "ArrowLeft" => Some(BUTTON_LEFT),
            "ArrowRight" => Some(BUTTON_RIGHT),
            "z" | "Z" => Some(BUTTON_A),
            "x" | "X" => Some(BUTTON_B),
            "a" | "A" => Some(BUTTON_X),
            "s" | "S" => Some(BUTTON_Y),
            "q" | "Q" => Some(BUTTON_L),
            "w" | "W" => Some(BUTTON_R),
            "Enter" => Some(BUTTON_START),
            "Shift" => Some(BUTTON_SELECT),
            _ => None,
        };
        
        if let Some(button) = button {
            self.controller_state |= button;
            self.emulator.borrow_mut().set_controller_input(0, self.controller_state);
        }
    }
    
    #[wasm_bindgen]
    pub fn handle_key_up(&mut self, event: &KeyboardEvent) {
        let button = match event.key().as_str() {
            "ArrowUp" => Some(BUTTON_UP),
            "ArrowDown" => Some(BUTTON_DOWN),
            "ArrowLeft" => Some(BUTTON_LEFT),
            "ArrowRight" => Some(BUTTON_RIGHT),
            "z" | "Z" => Some(BUTTON_A),
            "x" | "X" => Some(BUTTON_B),
            "a" | "A" => Some(BUTTON_X),
            "s" | "S" => Some(BUTTON_Y),
            "q" | "Q" => Some(BUTTON_L),
            "w" | "W" => Some(BUTTON_R),
            "Enter" => Some(BUTTON_START),
            "Shift" => Some(BUTTON_SELECT),
            _ => None,
        };
        
        if let Some(button) = button {
            self.controller_state &= !button;
            self.emulator.borrow_mut().set_controller_input(0, self.controller_state);
        }
    }
    
    #[wasm_bindgen]
    pub fn save_state(&self) -> Result<Vec<u8>, JsValue> {
        use crate::savestate::SaveState;
        
        let state = self.emulator.borrow()
            .save_state()
            .map_err(|e| JsValue::from_str(&format!("Failed to save state: {}", e)))?;
            
        SaveState::to_bytes(&state)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize state: {}", e)))
    }
    
    #[wasm_bindgen]
    pub fn load_state(&mut self, state_data: &[u8]) -> Result<(), JsValue> {
        use crate::savestate::SaveState;
        
        let state = SaveState::from_bytes(state_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize state: {}", e)))?;
            
        self.emulator.borrow_mut()
            .load_state(&state)
            .map_err(|e| JsValue::from_str(&format!("Failed to load state: {}", e)))
    }
    
    #[wasm_bindgen]
    pub fn get_fps(&self) -> f64 {
        60.0 // TODO: Implement actual FPS calculation
    }
    
    fn render_frame(&mut self) -> Result<(), JsValue> {
        let emulator = self.emulator.borrow();
        let frame = emulator.get_frame_buffer();
        
        // Convert RGB565 to RGBA8888
        for (i, chunk) in frame.chunks_exact(2).enumerate() {
            if i >= 256 * 224 {
                break;
            }
            
            let pixel = u16::from_le_bytes([chunk[0], chunk[1]]);
            let r = ((pixel >> 11) & 0x1F) as u8;
            let g = ((pixel >> 5) & 0x3F) as u8;
            let b = (pixel & 0x1F) as u8;
            
            // Expand to 8-bit
            let idx = i * 4;
            self.frame_buffer[idx] = (r << 3) | (r >> 2);
            self.frame_buffer[idx + 1] = (g << 2) | (g >> 4);
            self.frame_buffer[idx + 2] = (b << 3) | (b >> 2);
            self.frame_buffer[idx + 3] = 255;
        }
        
        // Create ImageData
        let image_data = ImageData::new_with_u8_clamped_array(
            wasm_bindgen::Clamped(&self.frame_buffer),
            256,
        )?;
        
        // Draw to canvas
        self.ctx.put_image_data(&image_data, 0.0, 0.0)?;
        
        Ok(())
    }
    
    fn process_audio(&self, _audio_ctx: &web_sys::AudioContext) -> Result<(), JsValue> {
        // TODO: Implement audio processing
        // For now, just return Ok
        Ok(())
    }
}

// Module initialization
#[wasm_bindgen(start)]
pub fn main() {
    console::log_1(&"CCSNES WASM module loaded".into());
}