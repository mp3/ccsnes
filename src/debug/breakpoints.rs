// Breakpoint management for debugging
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct BreakpointManager {
    // PC breakpoints (execution)
    pc_breakpoints: HashSet<u32>,
    
    // Memory read breakpoints
    read_breakpoints: HashSet<u32>,
    
    // Memory write breakpoints
    write_breakpoints: HashSet<u32>,
    
    // Conditional breakpoints
    conditional_breakpoints: Vec<ConditionalBreakpoint>,
    
    // Breakpoint hit counts
    hit_counts: std::collections::HashMap<u32, u32>,
    
    // Enable/disable state
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ConditionalBreakpoint {
    pub address: u32,
    pub condition: BreakpointCondition,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum BreakpointCondition {
    // Break when register equals value
    RegisterEquals { register: Register, value: u16 },
    
    // Break when register is in range
    RegisterInRange { register: Register, min: u16, max: u16 },
    
    // Break when memory equals value
    MemoryEquals { address: u32, value: u8 },
    
    // Break after N hits
    HitCount { count: u32 },
    
    // Combination of conditions
    And(Box<BreakpointCondition>, Box<BreakpointCondition>),
    Or(Box<BreakpointCondition>, Box<BreakpointCondition>),
    Not(Box<BreakpointCondition>),
}

#[derive(Debug, Clone, Copy)]
pub enum Register {
    A,
    X,
    Y,
    S,
    D,
    DB,
    P,
}

impl BreakpointManager {
    pub fn new() -> Self {
        Self {
            pc_breakpoints: HashSet::new(),
            read_breakpoints: HashSet::new(),
            write_breakpoints: HashSet::new(),
            conditional_breakpoints: Vec::new(),
            hit_counts: std::collections::HashMap::new(),
            enabled: true,
        }
    }
    
    // Enable/disable all breakpoints
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    // Add PC breakpoint
    pub fn add_pc_breakpoint(&mut self, address: u32) {
        self.pc_breakpoints.insert(address);
        log::debug!("Added PC breakpoint at ${:06X}", address);
    }
    
    // Remove PC breakpoint
    pub fn remove_pc_breakpoint(&mut self, address: u32) -> bool {
        let removed = self.pc_breakpoints.remove(&address);
        if removed {
            log::debug!("Removed PC breakpoint at ${:06X}", address);
        }
        removed
    }
    
    // Add read breakpoint
    pub fn add_read_breakpoint(&mut self, address: u32) {
        self.read_breakpoints.insert(address);
        log::debug!("Added read breakpoint at ${:06X}", address);
    }
    
    // Remove read breakpoint
    pub fn remove_read_breakpoint(&mut self, address: u32) -> bool {
        self.read_breakpoints.remove(&address)
    }
    
    // Add write breakpoint
    pub fn add_write_breakpoint(&mut self, address: u32) {
        self.write_breakpoints.insert(address);
        log::debug!("Added write breakpoint at ${:06X}", address);
    }
    
    // Remove write breakpoint
    pub fn remove_write_breakpoint(&mut self, address: u32) -> bool {
        self.write_breakpoints.remove(&address)
    }
    
    // Add conditional breakpoint
    pub fn add_conditional_breakpoint(&mut self, address: u32, condition: BreakpointCondition) {
        self.conditional_breakpoints.push(ConditionalBreakpoint {
            address,
            condition,
            enabled: true,
        });
        log::debug!("Added conditional breakpoint at ${:06X}", address);
    }
    
    // Check if PC breakpoint should trigger
    pub fn check_breakpoint(&self, pc: u32) -> bool {
        if !self.enabled {
            return false;
        }
        
        // Update hit count
        if self.pc_breakpoints.contains(&pc) {
            let count = self.hit_counts.get(&pc).unwrap_or(&0) + 1;
            // Note: In a mutable context, we would update hit_counts here
        }
        
        self.pc_breakpoints.contains(&pc)
    }
    
    // Check if read breakpoint should trigger
    pub fn check_read_breakpoint(&self, address: u32) -> bool {
        self.enabled && self.read_breakpoints.contains(&address)
    }
    
    // Check if write breakpoint should trigger
    pub fn check_write_breakpoint(&self, address: u32) -> bool {
        self.enabled && self.write_breakpoints.contains(&address)
    }
    
    // Check conditional breakpoints
    pub fn check_conditional_breakpoints(&self, pc: u32, cpu_state: &CpuState) -> bool {
        if !self.enabled {
            return false;
        }
        
        for bp in &self.conditional_breakpoints {
            if bp.enabled && bp.address == pc {
                if self.evaluate_condition(&bp.condition, cpu_state) {
                    return true;
                }
            }
        }
        
        false
    }
    
    // Evaluate breakpoint condition
    fn evaluate_condition(&self, condition: &BreakpointCondition, cpu_state: &CpuState) -> bool {
        match condition {
            BreakpointCondition::RegisterEquals { register, value } => {
                let reg_value = self.get_register_value(register, cpu_state);
                reg_value == *value
            }
            
            BreakpointCondition::RegisterInRange { register, min, max } => {
                let reg_value = self.get_register_value(register, cpu_state);
                reg_value >= *min && reg_value <= *max
            }
            
            BreakpointCondition::MemoryEquals { address, value } => {
                // Note: This would need access to memory bus in real implementation
                false
            }
            
            BreakpointCondition::HitCount { count } => {
                // Note: This would need mutable access to hit counts
                false
            }
            
            BreakpointCondition::And(a, b) => {
                self.evaluate_condition(a, cpu_state) && self.evaluate_condition(b, cpu_state)
            }
            
            BreakpointCondition::Or(a, b) => {
                self.evaluate_condition(a, cpu_state) || self.evaluate_condition(b, cpu_state)
            }
            
            BreakpointCondition::Not(a) => {
                !self.evaluate_condition(a, cpu_state)
            }
        }
    }
    
    fn get_register_value(&self, register: &Register, cpu_state: &CpuState) -> u16 {
        match register {
            Register::A => cpu_state.a,
            Register::X => cpu_state.x,
            Register::Y => cpu_state.y,
            Register::S => cpu_state.s,
            Register::D => cpu_state.d,
            Register::DB => cpu_state.db as u16,
            Register::P => cpu_state.p as u16,
        }
    }
    
    // Clear all breakpoints
    pub fn clear_all(&mut self) {
        self.pc_breakpoints.clear();
        self.read_breakpoints.clear();
        self.write_breakpoints.clear();
        self.conditional_breakpoints.clear();
        self.hit_counts.clear();
        log::debug!("Cleared all breakpoints");
    }
    
    // Get breakpoint statistics
    pub fn get_stats(&self) -> BreakpointStats {
        BreakpointStats {
            pc_count: self.pc_breakpoints.len(),
            read_count: self.read_breakpoints.len(),
            write_count: self.write_breakpoints.len(),
            conditional_count: self.conditional_breakpoints.len(),
            total_hits: self.hit_counts.values().sum(),
        }
    }
}

// CPU state for condition evaluation
pub struct CpuState {
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub s: u16,
    pub d: u16,
    pub db: u8,
    pub p: u8,
}

pub struct BreakpointStats {
    pub pc_count: usize,
    pub read_count: usize,
    pub write_count: usize,
    pub conditional_count: usize,
    pub total_hits: u32,
}