use wasm_bindgen::prelude::*;
use web_sys::{console, HtmlCanvasElement, CanvasRenderingContext2d, ImageData};
use crate::emulator::Emulator;

#[wasm_bindgen]
pub struct SnesEmulator {
    emulator: Emulator,
    canvas_id: String,
    last_frame_time: f64,
}

#[wasm_bindgen]
impl SnesEmulator {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<SnesEmulator, JsValue> {
        console::log_1(&"Creating SNES emulator".into());
        
        let emulator = Emulator::new()
            .map_err(|e| JsValue::from_str(&format!("Failed to create emulator: {}", e)))?;
        
        Ok(SnesEmulator {
            emulator,
            canvas_id: canvas_id.to_string(),
            last_frame_time: 0.0,
        })
    }

    #[wasm_bindgen]
    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<(), JsValue> {
        console::log_1(&format!("Loading ROM ({} bytes)", rom_data.len()).into());
        
        self.emulator.load_rom(rom_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to load ROM: {}", e)))?;
        
        console::log_1(&"ROM loaded successfully".into());
        Ok(())
    }

    #[wasm_bindgen]
    pub fn step_frame(&mut self) -> Result<(), JsValue> {
        self.emulator.step_frame()
            .map_err(|e| JsValue::from_str(&format!("Frame step failed: {}", e)))?;
        
        // Render to canvas
        self.render_to_canvas()?;
        
        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_controller_state(&mut self, player: u8, buttons: u16) {
        self.emulator.set_controller_input(player, buttons);
    }

    #[wasm_bindgen]
    pub fn get_video_buffer(&self) -> Vec<u8> {
        self.emulator.get_video_buffer().to_vec()
    }

    #[wasm_bindgen]
    pub fn get_audio_samples(&mut self) -> Vec<f32> {
        self.emulator.get_audio_samples()
    }

    #[wasm_bindgen]
    pub fn is_running(&self) -> bool {
        self.emulator.is_running()
    }

    #[wasm_bindgen]
    pub fn pause(&mut self) {
        self.emulator.pause();
    }

    #[wasm_bindgen]
    pub fn resume(&mut self) {
        self.emulator.resume();
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) -> Result<(), JsValue> {
        self.emulator.reset()
            .map_err(|e| JsValue::from_str(&format!("Reset failed: {}", e)))?;
        Ok(())
    }

    fn render_to_canvas(&self) -> Result<(), JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document
            .get_element_by_id(&self.canvas_id)
            .ok_or_else(|| JsValue::from_str("Canvas not found"))?
            .dyn_into::<HtmlCanvasElement>()?;

        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get 2D context"))?
            .dyn_into::<CanvasRenderingContext2d>()?;

        // Get frame buffer from emulator
        let frame_buffer = self.emulator.get_video_buffer();
        
        // Create ImageData from the frame buffer
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(frame_buffer),
            256, // width
            224, // height
        )?;

        // Draw to canvas
        context.put_image_data(&image_data, 0.0, 0.0)?;

        Ok(())
    }
}

// Helper functions for JavaScript interop

#[wasm_bindgen]
pub fn log(s: &str) {
    console::log_1(&s.into());
}

// Input mapping utilities
#[wasm_bindgen]
pub struct InputMapper;

#[wasm_bindgen]
impl InputMapper {
    #[wasm_bindgen]
    pub fn keyboard_to_snes_buttons(key_code: &str) -> u16 {
        use crate::input::controller::*;
        
        match key_code {
            // Arrow keys
            "ArrowUp" => BUTTON_UP,
            "ArrowDown" => BUTTON_DOWN,
            "ArrowLeft" => BUTTON_LEFT,
            "ArrowRight" => BUTTON_RIGHT,
            
            // Action buttons (using common WASD + other keys)
            "KeyZ" | "KeyJ" => BUTTON_A,
            "KeyX" | "KeyK" => BUTTON_B,
            "KeyA" | "KeyU" => BUTTON_X,
            "KeyS" | "KeyI" => BUTTON_Y,
            
            // Shoulder buttons
            "KeyQ" | "KeyO" => BUTTON_L,
            "KeyW" | "KeyP" => BUTTON_R,
            
            // Start/Select
            "Enter" => BUTTON_START,
            "Space" => BUTTON_SELECT,
            
            _ => 0,
        }
    }

    #[wasm_bindgen]
    pub fn gamepad_to_snes_buttons(gamepad_buttons: &[f64]) -> u16 {
        use crate::input::controller::*;
        
        let mut buttons = 0u16;
        
        if gamepad_buttons.len() >= 16 {
            // Standard gamepad mapping
            if gamepad_buttons[0] > 0.5 { buttons |= BUTTON_A; }      // A
            if gamepad_buttons[1] > 0.5 { buttons |= BUTTON_B; }      // B
            if gamepad_buttons[2] > 0.5 { buttons |= BUTTON_X; }      // X
            if gamepad_buttons[3] > 0.5 { buttons |= BUTTON_Y; }      // Y
            if gamepad_buttons[4] > 0.5 { buttons |= BUTTON_L; }      // L1
            if gamepad_buttons[5] > 0.5 { buttons |= BUTTON_R; }      // R1
            if gamepad_buttons[8] > 0.5 { buttons |= BUTTON_SELECT; } // Select
            if gamepad_buttons[9] > 0.5 { buttons |= BUTTON_START; }  // Start
            if gamepad_buttons[12] > 0.5 { buttons |= BUTTON_UP; }    // D-pad Up
            if gamepad_buttons[13] > 0.5 { buttons |= BUTTON_DOWN; }  // D-pad Down
            if gamepad_buttons[14] > 0.5 { buttons |= BUTTON_LEFT; }  // D-pad Left
            if gamepad_buttons[15] > 0.5 { buttons |= BUTTON_RIGHT; } // D-pad Right
        }
        
        buttons
    }
}