// Enhanced debugging features for the SNES emulator
use crate::cpu::Cpu;
use crate::memory::Bus;
use crate::ppu::Ppu;
use std::collections::VecDeque;
use std::fmt::Write;

pub mod breakpoints;
pub mod trace;
pub mod profiler;

pub use breakpoints::BreakpointManager;
pub use trace::Tracer;
pub use profiler::Profiler;

// Debugger state
pub struct Debugger {
    // Breakpoint management
    pub breakpoints: BreakpointManager,
    
    // Execution trace
    pub tracer: Tracer,
    
    // Performance profiler
    pub profiler: Profiler,
    
    // Debugger state
    pub enabled: bool,
    pub single_step: bool,
    pub break_on_next: bool,
    
    // Watch variables
    watches: Vec<Watch>,
    
    // Command history
    command_history: VecDeque<String>,
}

#[derive(Debug, Clone)]
pub struct Watch {
    pub name: String,
    pub address: u32,
    pub size: WatchSize,
    pub format: WatchFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum WatchSize {
    Byte,
    Word,
    Long,
}

#[derive(Debug, Clone, Copy)]
pub enum WatchFormat {
    Hex,
    Decimal,
    Binary,
    Ascii,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            breakpoints: BreakpointManager::new(),
            tracer: Tracer::new(),
            profiler: Profiler::new(),
            enabled: false,
            single_step: false,
            break_on_next: false,
            watches: Vec::new(),
            command_history: VecDeque::with_capacity(100),
        }
    }
    
    // Check if we should break execution
    pub fn should_break(&self, cpu: &Cpu) -> bool {
        if !self.enabled {
            return false;
        }
        
        if self.single_step || self.break_on_next {
            return true;
        }
        
        self.breakpoints.check_breakpoint(cpu.registers.pc)
    }
    
    // Handle debugger break
    pub fn handle_break(&mut self, cpu: &Cpu, bus: &Bus) -> DebuggerAction {
        if !self.enabled {
            return DebuggerAction::Continue;
        }
        
        // Reset break flags
        self.break_on_next = false;
        
        // Print current state
        println!("\n=== DEBUGGER BREAK ===");
        println!("PC: ${:06X}", cpu.registers.pc);
        println!("Registers: {}", cpu.registers);
        
        // Print watches
        if !self.watches.is_empty() {
            println!("\nWatches:");
            for watch in &self.watches {
                let value = self.read_watch(bus, watch);
                println!("  {}: {}", watch.name, value);
            }
        }
        
        // Return action (in a real implementation, this would wait for user input)
        DebuggerAction::Continue
    }
    
    // Add a watch
    pub fn add_watch(&mut self, name: String, address: u32, size: WatchSize, format: WatchFormat) {
        self.watches.push(Watch {
            name,
            address,
            size,
            format,
        });
    }
    
    // Remove a watch
    pub fn remove_watch(&mut self, name: &str) {
        self.watches.retain(|w| w.name != name);
    }
    
    // Read watch value
    fn read_watch(&self, bus: &Bus, watch: &Watch) -> String {
        let value = match watch.size {
            WatchSize::Byte => bus.read8(watch.address) as u32,
            WatchSize::Word => bus.read16(watch.address) as u32,
            WatchSize::Long => {
                let low = bus.read16(watch.address) as u32;
                let high = bus.read8(watch.address + 2) as u32;
                (high << 16) | low
            }
        };
        
        match watch.format {
            WatchFormat::Hex => match watch.size {
                WatchSize::Byte => format!("${:02X}", value),
                WatchSize::Word => format!("${:04X}", value),
                WatchSize::Long => format!("${:06X}", value),
            },
            WatchFormat::Decimal => format!("{}", value),
            WatchFormat::Binary => format!("{:b}", value),
            WatchFormat::Ascii => {
                if watch.size == WatchSize::Byte && value < 128 {
                    format!("'{}'", value as u8 as char)
                } else {
                    format!("${:02X}", value)
                }
            }
        }
    }
    
    // Disassemble at address
    pub fn disassemble(&self, bus: &Bus, address: u32, count: usize) -> String {
        let mut result = String::new();
        let mut addr = address;
        
        for _ in 0..count {
            let opcode = bus.read8(addr);
            writeln!(&mut result, "${:06X}: {:02X}  ; TODO: Disassemble", addr, opcode).unwrap();
            addr += 1; // Simplified - real implementation would handle instruction length
        }
        
        result
    }
    
    // Memory dump
    pub fn memory_dump(&self, bus: &Bus, address: u32, length: usize) -> String {
        let mut result = String::new();
        
        for offset in (0..length).step_by(16) {
            write!(&mut result, "${:06X}: ", address + offset as u32).unwrap();
            
            // Hex bytes
            for i in 0..16 {
                if offset + i < length {
                    let byte = bus.read8(address + (offset + i) as u32);
                    write!(&mut result, "{:02X} ", byte).unwrap();
                } else {
                    write!(&mut result, "   ").unwrap();
                }
            }
            
            write!(&mut result, " ").unwrap();
            
            // ASCII representation
            for i in 0..16 {
                if offset + i < length {
                    let byte = bus.read8(address + (offset + i) as u32);
                    let ch = if byte >= 0x20 && byte < 0x7F {
                        byte as char
                    } else {
                        '.'
                    };
                    write!(&mut result, "{}", ch).unwrap();
                }
            }
            
            writeln!(&mut result).unwrap();
        }
        
        result
    }
    
    // Search memory
    pub fn search_memory(&self, bus: &Bus, pattern: &[u8], start: u32, end: u32) -> Vec<u32> {
        let mut matches = Vec::new();
        
        if pattern.is_empty() {
            return matches;
        }
        
        for addr in start..=end.saturating_sub(pattern.len() as u32 - 1) {
            let mut found = true;
            for (i, &byte) in pattern.iter().enumerate() {
                if bus.read8(addr + i as u32) != byte {
                    found = false;
                    break;
                }
            }
            
            if found {
                matches.push(addr);
            }
        }
        
        matches
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DebuggerAction {
    Continue,
    Step,
    StepOver,
    StepOut,
    Break,
    Reset,
    Quit,
}

// Helper for formatting debug output
pub struct DebugFormatter;

impl DebugFormatter {
    pub fn format_cpu_state(cpu: &Cpu) -> String {
        format!(
            "A:{:04X} X:{:04X} Y:{:04X} S:{:04X} D:{:04X} DB:{:02X} PC:{:06X} P:{:02X} [{}]",
            cpu.registers.a,
            cpu.registers.x,
            cpu.registers.y,
            cpu.registers.s,
            cpu.registers.d,
            cpu.registers.db,
            cpu.registers.pc,
            cpu.registers.p,
            Self::format_flags(cpu.registers.p)
        )
    }
    
    fn format_flags(p: u8) -> String {
        format!(
            "{}{}{}{}{}{}{}{}",
            if p & 0x80 != 0 { 'N' } else { 'n' },
            if p & 0x40 != 0 { 'V' } else { 'v' },
            if p & 0x20 != 0 { 'M' } else { 'm' },
            if p & 0x10 != 0 { 'X' } else { 'x' },
            if p & 0x08 != 0 { 'D' } else { 'd' },
            if p & 0x04 != 0 { 'I' } else { 'i' },
            if p & 0x02 != 0 { 'Z' } else { 'z' },
            if p & 0x01 != 0 { 'C' } else { 'c' }
        )
    }
    
    pub fn format_ppu_state(ppu: &Ppu) -> String {
        format!(
            "Scanline: {} Dot: {} Frame: {} VBlank: {}",
            ppu.get_current_scanline(),
            ppu.get_current_dot(),
            ppu.get_frame_count(),
            ppu.is_in_vblank()
        )
    }
}