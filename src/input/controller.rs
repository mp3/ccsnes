// SNES controller button mapping
pub const BUTTON_B: u16      = 0x8000;
pub const BUTTON_Y: u16      = 0x4000;
pub const BUTTON_SELECT: u16 = 0x2000;
pub const BUTTON_START: u16  = 0x1000;
pub const BUTTON_UP: u16     = 0x0800;
pub const BUTTON_DOWN: u16   = 0x0400;
pub const BUTTON_LEFT: u16   = 0x0200;
pub const BUTTON_RIGHT: u16  = 0x0100;
pub const BUTTON_A: u16      = 0x0080;
pub const BUTTON_X: u16      = 0x0040;
pub const BUTTON_L: u16      = 0x0020;
pub const BUTTON_R: u16      = 0x0010;

pub struct Controller {
    state: u16,         // Current button state
    shift_register: u16, // Shift register for serial reading
    strobe: bool,       // Strobe state
}

impl Controller {
    pub fn new() -> Self {
        Self {
            state: 0,
            shift_register: 0,
            strobe: false,
        }
    }

    pub fn set_state(&mut self, buttons: u16) {
        self.state = buttons;
        if !self.strobe {
            self.shift_register = self.state;
        }
    }

    pub fn strobe(&mut self, value: bool) {
        let was_strobing = self.strobe;
        self.strobe = value;
        
        if was_strobing && !value {
            // Falling edge of strobe - load shift register
            self.shift_register = self.state;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.strobe {
            // While strobing, always return button A state
            (self.state & BUTTON_A != 0) as u8
        } else {
            // Shift out one bit
            let bit = (self.shift_register & 0x8000) != 0;
            self.shift_register <<= 1;
            self.shift_register |= 1; // Pad with 1s
            bit as u8
        }
    }

    pub fn get_state(&self) -> u16 {
        self.state
    }

    pub fn is_pressed(&self, button: u16) -> bool {
        (self.state & button) != 0
    }
}