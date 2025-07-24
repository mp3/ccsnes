use std::fmt;

#[derive(Debug, Clone)]
pub struct CpuRegisters {
    // 16-bit accumulator (A)
    pub a: u16,
    // 16-bit index registers (X, Y)
    pub x: u16,
    pub y: u16,
    // 16-bit stack pointer (S)
    pub s: u16,
    // 24-bit program counter (PC) - stored as u32 but only uses 24 bits
    pub pc: u32,
    // 8-bit processor status (P)
    pub p: u8,
    // 8-bit data bank register (DB)
    pub db: u8,
    // 16-bit direct page register (D)
    pub d: u16,
    
    // Emulation mode flag (not a real register, but important for 65C816)
    pub emulation_mode: bool,
    
    // CPU state flags
    pub halt: bool,
    pub waiting_for_interrupt: bool,
}

// Processor status flags (P register)
pub const FLAG_CARRY: u8        = 0x01; // C - Carry
pub const FLAG_ZERO: u8         = 0x02; // Z - Zero
pub const FLAG_IRQ_DISABLE: u8  = 0x04; // I - IRQ Disable
pub const FLAG_DECIMAL: u8      = 0x08; // D - Decimal mode
pub const FLAG_INDEX_WIDTH: u8  = 0x10; // X - Index register width (0=16bit, 1=8bit)
pub const FLAG_MEMORY_WIDTH: u8 = 0x20; // M - Memory/Accumulator width (0=16bit, 1=8bit)
pub const FLAG_OVERFLOW: u8     = 0x40; // V - Overflow
pub const FLAG_NEGATIVE: u8     = 0x80; // N - Negative

impl CpuRegisters {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            s: 0x01FF,         // Stack starts at $01FF
            pc: 0,
            p: 0x34,           // Default: X=1, M=1, I=1 (8-bit mode, IRQ disabled)
            db: 0,
            d: 0,
            emulation_mode: true, // Start in 6502 emulation mode
            halt: false,
            waiting_for_interrupt: false,
        }
    }

    // Processor status flag operations
    pub fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.p |= flag;
        } else {
            self.p &= !flag;
        }
    }

    pub fn get_flag(&self, flag: u8) -> bool {
        (self.p & flag) != 0
    }

    // Individual flag getters/setters
    pub fn carry(&self) -> bool { self.get_flag(FLAG_CARRY) }
    pub fn zero(&self) -> bool { self.get_flag(FLAG_ZERO) }
    pub fn irq_disable(&self) -> bool { self.get_flag(FLAG_IRQ_DISABLE) }
    pub fn decimal(&self) -> bool { self.get_flag(FLAG_DECIMAL) }
    pub fn index_width(&self) -> bool { self.get_flag(FLAG_INDEX_WIDTH) }
    pub fn memory_width(&self) -> bool { self.get_flag(FLAG_MEMORY_WIDTH) }
    pub fn overflow(&self) -> bool { self.get_flag(FLAG_OVERFLOW) }
    pub fn negative(&self) -> bool { self.get_flag(FLAG_NEGATIVE) }

    pub fn set_carry(&mut self, value: bool) { self.set_flag(FLAG_CARRY, value); }
    pub fn set_zero(&mut self, value: bool) { self.set_flag(FLAG_ZERO, value); }
    pub fn set_irq_disable(&mut self, value: bool) { self.set_flag(FLAG_IRQ_DISABLE, value); }
    pub fn set_decimal(&mut self, value: bool) { self.set_flag(FLAG_DECIMAL, value); }
    pub fn set_index_width(&mut self, value: bool) { self.set_flag(FLAG_INDEX_WIDTH, value); }
    pub fn set_memory_width(&mut self, value: bool) { self.set_flag(FLAG_MEMORY_WIDTH, value); }
    pub fn set_overflow(&mut self, value: bool) { self.set_flag(FLAG_OVERFLOW, value); }
    pub fn set_negative(&mut self, value: bool) { self.set_flag(FLAG_NEGATIVE, value); }

    // Width-dependent register access
    pub fn get_a(&self) -> u16 {
        if self.memory_width() {
            // 8-bit mode: return only low byte
            self.a & 0xFF
        } else {
            // 16-bit mode: return full 16 bits
            self.a
        }
    }

    pub fn set_a(&mut self, value: u16) {
        if self.memory_width() {
            // 8-bit mode: preserve high byte, set only low byte
            self.a = (self.a & 0xFF00) | (value & 0xFF);
        } else {
            // 16-bit mode: set full 16 bits
            self.a = value;
        }
    }

    pub fn get_x(&self) -> u16 {
        if self.index_width() {
            // 8-bit mode: return only low byte
            self.x & 0xFF
        } else {
            // 16-bit mode: return full 16 bits
            self.x
        }
    }

    pub fn set_x(&mut self, value: u16) {
        if self.index_width() {
            // 8-bit mode: preserve high byte, set only low byte
            self.x = (self.x & 0xFF00) | (value & 0xFF);
        } else {
            // 16-bit mode: set full 16 bits
            self.x = value;
        }
    }

    pub fn get_y(&self) -> u16 {
        if self.index_width() {
            // 8-bit mode: return only low byte
            self.y & 0xFF
        } else {
            // 16-bit mode: return full 16 bits
            self.y
        }
    }

    pub fn set_y(&mut self, value: u16) {
        if self.index_width() {
            // 8-bit mode: preserve high byte, set only low byte
            self.y = (self.y & 0xFF00) | (value & 0xFF);
        } else {
            // 16-bit mode: set full 16 bits
            self.y = value;
        }
    }

    // Update flags based on value
    pub fn update_nz_flags(&mut self, value: u16) {
        if self.memory_width() {
            // 8-bit mode
            let val = (value & 0xFF) as u8;
            self.set_zero(val == 0);
            self.set_negative((val & 0x80) != 0);
        } else {
            // 16-bit mode
            self.set_zero(value == 0);
            self.set_negative((value & 0x8000) != 0);
        }
    }

    pub fn update_nz_flags_index(&mut self, value: u16) {
        if self.index_width() {
            // 8-bit mode
            let val = (value & 0xFF) as u8;
            self.set_zero(val == 0);
            self.set_negative((val & 0x80) != 0);
        } else {
            // 16-bit mode
            self.set_zero(value == 0);
            self.set_negative((value & 0x8000) != 0);
        }
    }

    // Stack operations
    pub fn push_8(&mut self, bus: &mut crate::memory::Bus, value: u8) {
        bus.write8(self.s as u32, value);
        self.s = self.s.wrapping_sub(1);
        if self.emulation_mode {
            // In emulation mode, stack wraps within page 1
            self.s = (self.s & 0xFF) | 0x0100;
        }
    }

    pub fn pop_8(&mut self, bus: &mut crate::memory::Bus) -> u8 {
        self.s = self.s.wrapping_add(1);
        if self.emulation_mode {
            // In emulation mode, stack wraps within page 1
            self.s = (self.s & 0xFF) | 0x0100;
        }
        bus.read8(self.s as u32)
    }

    pub fn push_16(&mut self, bus: &mut crate::memory::Bus, value: u16) {
        self.push_8(bus, (value >> 8) as u8);   // High byte first
        self.push_8(bus, (value & 0xFF) as u8); // Low byte second
    }

    pub fn pop_16(&mut self, bus: &mut crate::memory::Bus) -> u16 {
        let low = self.pop_8(bus) as u16;       // Low byte first
        let high = self.pop_8(bus) as u16;      // High byte second
        (high << 8) | low
    }

    // Program counter operations
    pub fn get_pc_bank(&self) -> u8 {
        ((self.pc >> 16) & 0xFF) as u8
    }

    pub fn get_pc_offset(&self) -> u16 {
        (self.pc & 0xFFFF) as u16
    }

    pub fn set_pc(&mut self, bank: u8, offset: u16) {
        self.pc = ((bank as u32) << 16) | (offset as u32);
    }

    pub fn increment_pc(&mut self, amount: u32) {
        self.pc = (self.pc + amount) & 0xFFFFFF; // Keep within 24-bit space
    }

    // Mode switching
    pub fn enter_native_mode(&mut self) {
        self.emulation_mode = false;
        // Set stack to 16-bit (if it was in page 1)
        if (self.s & 0xFF00) == 0x0100 {
            self.s = self.s | 0x0000; // Stack can now be anywhere
        }
    }

    pub fn enter_emulation_mode(&mut self) {
        self.emulation_mode = true;
        // Force stack into page 1
        self.s = (self.s & 0xFF) | 0x0100;
        // Set flags for 6502 compatibility
        self.set_memory_width(true);  // 8-bit accumulator
        self.set_index_width(true);   // 8-bit index registers
    }

    // Get effective address width for current addressing mode
    pub fn get_memory_width(&self) -> u8 {
        if self.memory_width() { 8 } else { 16 }
    }

    pub fn get_index_width(&self) -> u8 {
        if self.index_width() { 8 } else { 16 }
    }
}

impl fmt::Display for CpuRegisters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A:{:04X} X:{:04X} Y:{:04X} S:{:04X} PC:{:06X} P:{:02X} DB:{:02X} D:{:04X}",
               self.a, self.x, self.y, self.s, self.pc, self.p, self.db, self.d)?;
        
        let flags = format!("{}{}{}{}{}{}{}{}",
            if self.negative() { "N" } else { "n" },
            if self.overflow() { "V" } else { "v" },
            if self.memory_width() { "M" } else { "m" },
            if self.index_width() { "X" } else { "x" },
            if self.decimal() { "D" } else { "d" },
            if self.irq_disable() { "I" } else { "i" },
            if self.zero() { "Z" } else { "z" },
            if self.carry() { "C" } else { "c" }
        );
        
        write!(f, " [{}] {}", flags, if self.emulation_mode { "EMU" } else { "NAT" })
    }
}