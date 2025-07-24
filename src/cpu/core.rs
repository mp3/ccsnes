use crate::memory::Bus;
use crate::Result;
use crate::cpu::registers::CpuRegisters;
use crate::cpu::instructions::decode_opcode;
use crate::cpu::execute::execute_instruction;
use crate::savestate::CpuState;

pub struct Cpu {
    pub registers: CpuRegisters,
    pub cycles: u64,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: CpuRegisters::new(),
            cycles: 0,
        }
    }

    pub fn reset(&mut self, bus: &mut Bus) -> Result<()> {
        // Read reset vector from $FFFC-$FFFD
        let reset_vector = bus.read16(0xFFFC);
        
        // Initialize registers
        self.registers = CpuRegisters::new();
        self.registers.pc = reset_vector as u32;
        self.registers.s = 0x01FF;  // Stack pointer
        self.registers.p = 0x34;    // IRQ disabled, 8-bit mode
        self.registers.emulation_mode = true; // Start in emulation mode
        
        self.cycles = 0;
        self.registers.halt = false;
        self.registers.waiting_for_interrupt = false;
        
        log::info!("CPU Reset - PC: ${:04X}, S: ${:04X}", reset_vector, self.registers.s);
        
        Ok(())
    }

    pub fn step(&mut self, bus: &mut Bus) -> Result<u32> {
        if self.registers.halt || self.registers.waiting_for_interrupt {
            // CPU is halted, just consume 1 cycle
            self.cycles += 1;
            return Ok(1);
        }
        
        // Fetch opcode
        let opcode = bus.read8(self.registers.pc);
        self.registers.increment_pc(1);
        
        // Decode instruction
        if let Some(info) = decode_opcode(opcode) {
            // Execute instruction
            let cycles = execute_instruction(&mut self.registers, bus, &info)?;
            self.cycles += cycles as u64;
            
            Ok(cycles)
        } else {
            // Unknown opcode - treat as NOP
            log::warn!("Unknown opcode ${:02X} at PC ${:06X}", opcode, self.registers.pc - 1);
            self.cycles += 2;
            Ok(2)
        }
    }

    pub fn trigger_nmi(&mut self, bus: &mut Bus) -> Result<()> {
        if self.registers.waiting_for_interrupt {
            self.registers.waiting_for_interrupt = false;
        }
        
        // Push PC (24-bit in native mode, 16-bit in emulation mode)
        if !self.registers.emulation_mode {
            // Native mode: push 24-bit PC
            self.registers.push_8(bus, self.registers.get_pc_bank());
        }
        self.registers.push_16(bus, self.registers.get_pc_offset());
        
        // Push processor status
        self.registers.push_8(bus, self.registers.p);
        
        // Set interrupt disable flag
        self.registers.set_irq_disable(true);
        
        // Jump to NMI vector
        let nmi_vector = if self.registers.emulation_mode {
            bus.read16(0xFFFA) // Emulation mode NMI vector
        } else {
            bus.read16(0xFFEA) // Native mode NMI vector
        };
        
        self.registers.pc = nmi_vector as u32;
        
        // NMI takes 7-8 cycles
        self.cycles += 8;
        
        Ok(())
    }

    pub fn trigger_irq(&mut self, bus: &mut Bus) -> Result<()> {
        // IRQ is ignored if interrupt disable flag is set
        if self.registers.irq_disable() {
            return Ok(());
        }
        
        if self.registers.waiting_for_interrupt {
            self.registers.waiting_for_interrupt = false;
        }
        
        // Push PC (24-bit in native mode, 16-bit in emulation mode)
        if !self.registers.emulation_mode {
            // Native mode: push 24-bit PC
            self.registers.push_8(bus, self.registers.get_pc_bank());
        }
        self.registers.push_16(bus, self.registers.get_pc_offset());
        
        // Push processor status
        self.registers.push_8(bus, self.registers.p);
        
        // Set interrupt disable flag
        self.registers.set_irq_disable(true);
        
        // Jump to IRQ vector
        let irq_vector = if self.registers.emulation_mode {
            bus.read16(0xFFFE) // Emulation mode IRQ vector
        } else {
            bus.read16(0xFFEE) // Native mode IRQ vector
        };
        
        self.registers.pc = irq_vector as u32;
        
        // IRQ takes 7-8 cycles
        self.cycles += 8;
        
        Ok(())
    }
    
    pub fn get_registers(&self) -> &CpuRegisters {
        &self.registers
    }
    
    pub fn get_registers_mut(&mut self) -> &mut CpuRegisters {
        &mut self.registers
    }
    
    pub fn get_cycles(&self) -> u64 {
        self.cycles
    }
    
    // Save state functionality
    pub fn save_state(&self) -> CpuState {
        CpuState {
            a: self.registers.a,
            x: self.registers.x,
            y: self.registers.y,
            s: self.registers.s,
            d: self.registers.d,
            db: self.registers.db,
            pb: self.registers.get_pc_bank(),
            pc: self.registers.get_pc_offset(),
            p: self.registers.p,
            emulation_mode: self.registers.emulation_mode,
            stopped: self.registers.halt,
            waiting_for_interrupt: self.registers.waiting_for_interrupt,
            nmi_pending: false, // TODO: Track NMI state
            irq_pending: false, // TODO: Track IRQ state
        }
    }
    
    pub fn load_state(&mut self, state: &CpuState) {
        self.registers.a = state.a;
        self.registers.x = state.x;
        self.registers.y = state.y;
        self.registers.s = state.s;
        self.registers.d = state.d;
        self.registers.db = state.db;
        self.registers.set_pc(state.pb, state.pc);
        self.registers.p = state.p;
        self.registers.emulation_mode = state.emulation_mode;
        self.registers.halt = state.stopped;
        self.registers.waiting_for_interrupt = state.waiting_for_interrupt;
    }
}