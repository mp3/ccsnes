pub mod controller;

pub use controller::Controller;

pub struct Input {
    controller1: Controller,
    controller2: Controller,
}

impl Input {
    pub fn new() -> Self {
        Self {
            controller1: Controller::new(),
            controller2: Controller::new(),
        }
    }

    pub fn set_controller_state(&mut self, player: u8, buttons: u16) {
        match player {
            0 => self.controller1.set_state(buttons),
            1 => self.controller2.set_state(buttons),
            _ => {}
        }
    }

    pub fn read_controller(&mut self, player: u8) -> u8 {
        match player {
            0 => self.controller1.read(),
            1 => self.controller2.read(),
            _ => 0,
        }
    }
    
    pub fn strobe_controllers(&mut self, value: bool) {
        self.controller1.strobe(value);
        self.controller2.strobe(value);
    }
}