// Execution trace for debugging
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufWriter, Write};
use crate::cpu::instructions::InstructionInfo;

const DEFAULT_TRACE_SIZE: usize = 10000;

#[derive(Debug, Clone)]
pub struct Tracer {
    // Circular buffer of trace entries
    entries: VecDeque<TraceEntry>,
    
    // Maximum number of entries to keep
    max_entries: usize,
    
    // Enable/disable tracing
    enabled: bool,
    
    // Trace to file
    file_writer: Option<BufWriter<File>>,
    
    // Filter settings
    filter: TraceFilter,
    
    // Statistics
    total_traced: u64,
}

#[derive(Debug, Clone)]
pub struct TraceEntry {
    // CPU state
    pub pc: u32,
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub s: u16,
    pub p: u8,
    pub db: u8,
    
    // Instruction info
    pub opcode: u8,
    pub instruction: Option<InstructionInfo>,
    pub operand: u32,
    
    // Timing
    pub cycle: u64,
    pub scanline: u16,
    pub dot: u32,
    
    // Memory access
    pub memory_reads: Vec<(u32, u8)>,
    pub memory_writes: Vec<(u32, u8)>,
}

#[derive(Debug, Clone)]
pub struct TraceFilter {
    // PC range filter
    pub pc_min: Option<u32>,
    pub pc_max: Option<u32>,
    
    // Bank filter
    pub banks: Option<Vec<u8>>,
    
    // Instruction filter
    pub instructions: Option<Vec<String>>,
    
    // Only trace on specific conditions
    pub only_branches: bool,
    pub only_interrupts: bool,
    pub only_memory_access: bool,
}

impl Tracer {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(DEFAULT_TRACE_SIZE),
            max_entries: DEFAULT_TRACE_SIZE,
            enabled: false,
            file_writer: None,
            filter: TraceFilter::default(),
            total_traced: 0,
        }
    }
    
    // Enable/disable tracing
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            log::info!("CPU tracing enabled");
        } else {
            log::info!("CPU tracing disabled");
        }
    }
    
    // Set maximum trace entries
    pub fn set_max_entries(&mut self, max: usize) {
        self.max_entries = max;
        self.entries.reserve(max);
    }
    
    // Start tracing to file
    pub fn start_file_trace(&mut self, path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        self.file_writer = Some(BufWriter::new(file));
        log::info!("Started trace to file: {}", path);
        Ok(())
    }
    
    // Stop file tracing
    pub fn stop_file_trace(&mut self) {
        if let Some(mut writer) = self.file_writer.take() {
            let _ = writer.flush();
            log::info!("Stopped file trace");
        }
    }
    
    // Add trace entry
    pub fn trace(&mut self, entry: TraceEntry) {
        if !self.enabled {
            return;
        }
        
        // Apply filters
        if !self.should_trace(&entry) {
            return;
        }
        
        // Write to file if enabled
        if let Some(ref mut writer) = self.file_writer {
            let _ = writeln!(writer, "{}", self.format_entry(&entry));
        }
        
        // Add to circular buffer
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
        
        self.total_traced += 1;
    }
    
    // Check if entry should be traced based on filters
    fn should_trace(&self, entry: &TraceEntry) -> bool {
        // PC range filter
        if let Some(min) = self.filter.pc_min {
            if entry.pc < min {
                return false;
            }
        }
        if let Some(max) = self.filter.pc_max {
            if entry.pc > max {
                return false;
            }
        }
        
        // Bank filter
        if let Some(ref banks) = self.filter.banks {
            let bank = (entry.pc >> 16) as u8;
            if !banks.contains(&bank) {
                return false;
            }
        }
        
        // Instruction filter
        if let Some(ref instructions) = self.filter.instructions {
            if let Some(ref info) = entry.instruction {
                let instr_name = format!("{:?}", info.instruction);
                if !instructions.contains(&instr_name) {
                    return false;
                }
            }
        }
        
        // Conditional filters
        if self.filter.only_branches {
            // Check if instruction is a branch
            if let Some(ref info) = entry.instruction {
                let instr_name = format!("{:?}", info.instruction);
                if !instr_name.starts_with('B') && !["JMP", "JSR", "JSL", "RTS", "RTL", "RTI"]
                    .contains(&instr_name.as_str()) {
                    return false;
                }
            }
        }
        
        if self.filter.only_memory_access {
            if entry.memory_reads.is_empty() && entry.memory_writes.is_empty() {
                return false;
            }
        }
        
        true
    }
    
    // Format trace entry for display
    fn format_entry(&self, entry: &TraceEntry) -> String {
        let mut result = format!(
            "[{:08}] ${:06X}: {:02X} ",
            entry.cycle,
            entry.pc,
            entry.opcode
        );
        
        // Add instruction mnemonic
        if let Some(ref info) = entry.instruction {
            result.push_str(&format!("{:<4} ", format!("{:?}", info.instruction)));
        } else {
            result.push_str("???  ");
        }
        
        // Add operand
        result.push_str(&format!("${:06X} ", entry.operand));
        
        // Add registers
        result.push_str(&format!(
            "A:{:04X} X:{:04X} Y:{:04X} S:{:04X} P:{:02X} DB:{:02X}",
            entry.a, entry.x, entry.y, entry.s, entry.p, entry.db
        ));
        
        // Add memory access
        if !entry.memory_reads.is_empty() {
            result.push_str(" R:");
            for (addr, val) in &entry.memory_reads {
                result.push_str(&format!("${:06X}={:02X} ", addr, val));
            }
        }
        
        if !entry.memory_writes.is_empty() {
            result.push_str(" W:");
            for (addr, val) in &entry.memory_writes {
                result.push_str(&format!("${:06X}={:02X} ", addr, val));
            }
        }
        
        // Add timing info
        result.push_str(&format!(" (Line:{} Dot:{})", entry.scanline, entry.dot));
        
        result
    }
    
    // Get recent trace entries
    pub fn get_recent(&self, count: usize) -> Vec<&TraceEntry> {
        let start = self.entries.len().saturating_sub(count);
        self.entries.range(start..).collect()
    }
    
    // Search trace for pattern
    pub fn search(&self, pattern: &str) -> Vec<&TraceEntry> {
        self.entries.iter()
            .filter(|entry| {
                let formatted = self.format_entry(entry);
                formatted.contains(pattern)
            })
            .collect()
    }
    
    // Clear trace buffer
    pub fn clear(&mut self) {
        self.entries.clear();
        log::info!("Trace buffer cleared");
    }
    
    // Get trace statistics
    pub fn get_stats(&self) -> TraceStats {
        let mut instruction_counts = std::collections::HashMap::new();
        let mut bank_counts = std::collections::HashMap::new();
        
        for entry in &self.entries {
            // Count instructions
            if let Some(ref info) = entry.instruction {
                let instr_name = format!("{:?}", info.instruction);
                *instruction_counts.entry(instr_name).or_insert(0) += 1;
            }
            
            // Count banks
            let bank = (entry.pc >> 16) as u8;
            *bank_counts.entry(bank).or_insert(0) += 1;
        }
        
        TraceStats {
            total_entries: self.entries.len(),
            total_traced: self.total_traced,
            instruction_counts,
            bank_counts,
        }
    }
}

impl Default for TraceFilter {
    fn default() -> Self {
        Self {
            pc_min: None,
            pc_max: None,
            banks: None,
            instructions: None,
            only_branches: false,
            only_interrupts: false,
            only_memory_access: false,
        }
    }
}

pub struct TraceStats {
    pub total_entries: usize,
    pub total_traced: u64,
    pub instruction_counts: std::collections::HashMap<String, u32>,
    pub bank_counts: std::collections::HashMap<u8, u32>,
}