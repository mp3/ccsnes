// TODO: Implement 65C816 CPU registers

pub struct CpuRegisters {
    // 16-bit accumulator (A)
    pub a: u16,
    // 16-bit index registers (X, Y)
    pub x: u16,
    pub y: u16,
    // 16-bit stack pointer (S)
    pub s: u16,
    // 24-bit program counter (PC)
    pub pc: u32,
    // 8-bit processor status (P)
    pub p: u8,
    // 8-bit data bank register (DB)
    pub db: u8,
    // 16-bit direct page register (D)
    pub d: u16,
}

impl CpuRegisters {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            s: 0x01FF,
            pc: 0,
            p: 0x34, // Default processor status
            db: 0,
            d: 0,
        }
    }
}