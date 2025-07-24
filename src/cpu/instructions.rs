use crate::cpu::addressing::AddressingMode;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    // Load/Store Instructions
    LDA,    // Load Accumulator
    LDX,    // Load X Register
    LDY,    // Load Y Register
    STA,    // Store Accumulator
    STX,    // Store X Register
    STY,    // Store Y Register
    STZ,    // Store Zero
    
    // Transfer Instructions
    TAX,    // Transfer A to X
    TAY,    // Transfer A to Y
    TXA,    // Transfer X to A
    TYA,    // Transfer Y to A
    TSX,    // Transfer Stack to X
    TXS,    // Transfer X to Stack
    TXY,    // Transfer X to Y
    TYX,    // Transfer Y to X
    TCD,    // Transfer C to Direct Page
    TDC,    // Transfer Direct Page to C
    TCS,    // Transfer C to Stack
    TSC,    // Transfer Stack to C
    
    // Stack Instructions
    PHA,    // Push Accumulator
    PHP,    // Push Processor Status
    PLA,    // Pull Accumulator
    PLP,    // Pull Processor Status
    PHX,    // Push X
    PHY,    // Push Y
    PLX,    // Pull X
    PLY,    // Pull Y
    PHB,    // Push Data Bank
    PLB,    // Pull Data Bank
    PHD,    // Push Direct Page
    PLD,    // Pull Direct Page
    PHK,    // Push Program Bank
    
    // Arithmetic Instructions
    ADC,    // Add with Carry
    SBC,    // Subtract with Carry
    INC,    // Increment
    INX,    // Increment X
    INY,    // Increment Y
    DEC,    // Decrement
    DEX,    // Decrement X
    DEY,    // Decrement Y
    
    // Logic Instructions
    AND,    // Logical AND
    ORA,    // Logical OR
    EOR,    // Exclusive OR
    BIT,    // Bit Test
    TSB,    // Test and Set Bits
    TRB,    // Test and Reset Bits
    
    // Shift/Rotate Instructions
    ASL,    // Arithmetic Shift Left
    LSR,    // Logical Shift Right
    ROL,    // Rotate Left
    ROR,    // Rotate Right
    
    // Compare Instructions
    CMP,    // Compare Accumulator
    CPX,    // Compare X
    CPY,    // Compare Y
    
    // Branch Instructions
    BCC,    // Branch if Carry Clear
    BCS,    // Branch if Carry Set
    BEQ,    // Branch if Equal (Zero)
    BNE,    // Branch if Not Equal
    BMI,    // Branch if Minus
    BPL,    // Branch if Plus
    BVC,    // Branch if Overflow Clear
    BVS,    // Branch if Overflow Set
    BRA,    // Branch Always
    BRL,    // Branch Long
    
    // Jump Instructions
    JMP,    // Jump
    JML,    // Jump Long
    JSR,    // Jump to Subroutine
    JSL,    // Jump to Subroutine Long
    RTS,    // Return from Subroutine
    RTL,    // Return from Subroutine Long
    RTI,    // Return from Interrupt
    
    // Flag Instructions
    CLC,    // Clear Carry
    CLD,    // Clear Decimal
    CLI,    // Clear Interrupt Disable
    CLV,    // Clear Overflow
    SEC,    // Set Carry
    SED,    // Set Decimal
    SEI,    // Set Interrupt Disable
    REP,    // Reset Processor Status Bits
    SEP,    // Set Processor Status Bits
    
    // Miscellaneous Instructions
    NOP,    // No Operation
    WDM,    // William D. Mensch Jr. (Reserved)
    XBA,    // Exchange B and A
    XCE,    // Exchange Carry and Emulation
    STP,    // Stop the Clock
    WAI,    // Wait for Interrupt
    BRK,    // Break
    COP,    // Co-processor
    
    // Block Move Instructions
    MVN,    // Move Block Negative
    MVP,    // Move Block Positive
    
    // New 65816 Instructions
    PEA,    // Push Effective Absolute Address
    PEI,    // Push Effective Indirect Address
    PER,    // Push Effective Relative Address
}

#[derive(Debug, Clone)]
pub struct InstructionInfo {
    pub instruction: Instruction,
    pub addressing_mode: AddressingMode,
    pub base_cycles: u8,
}

impl InstructionInfo {
    pub fn new(instruction: Instruction, addressing_mode: AddressingMode, base_cycles: u8) -> Self {
        Self {
            instruction,
            addressing_mode,
            base_cycles,
        }
    }
}

// Opcode decoding table (256 entries)
pub fn decode_opcode(opcode: u8) -> Option<InstructionInfo> {
    use Instruction::*;
    use AddressingMode::*;
    
    match opcode {
        // 0x00-0x0F
        0x00 => Some(InstructionInfo::new(BRK, Implied, 7)),
        0x01 => Some(InstructionInfo::new(ORA, DirectPageIndirectX, 6)),
        0x02 => Some(InstructionInfo::new(COP, Immediate, 7)),
        0x03 => Some(InstructionInfo::new(ORA, StackRelative, 4)),
        0x04 => Some(InstructionInfo::new(TSB, DirectPage, 5)),
        0x05 => Some(InstructionInfo::new(ORA, DirectPage, 3)),
        0x06 => Some(InstructionInfo::new(ASL, DirectPage, 5)),
        0x07 => Some(InstructionInfo::new(ORA, DirectPageIndirectLong, 6)),
        0x08 => Some(InstructionInfo::new(PHP, Implied, 3)),
        0x09 => Some(InstructionInfo::new(ORA, Immediate, 2)),
        0x0A => Some(InstructionInfo::new(ASL, Accumulator, 2)),
        0x0B => Some(InstructionInfo::new(PHD, Implied, 4)),
        0x0C => Some(InstructionInfo::new(TSB, Absolute, 6)),
        0x0D => Some(InstructionInfo::new(ORA, Absolute, 4)),
        0x0E => Some(InstructionInfo::new(ASL, Absolute, 6)),
        0x0F => Some(InstructionInfo::new(ORA, AbsoluteLong, 5)),
        
        // 0x10-0x1F
        0x10 => Some(InstructionInfo::new(BPL, Relative, 2)),
        0x11 => Some(InstructionInfo::new(ORA, DirectPageIndirectY, 5)),
        0x12 => Some(InstructionInfo::new(ORA, DirectPageIndirect, 5)),
        0x13 => Some(InstructionInfo::new(ORA, StackRelativeIndirectY, 7)),
        0x14 => Some(InstructionInfo::new(TRB, DirectPage, 5)),
        0x15 => Some(InstructionInfo::new(ORA, DirectPageX, 4)),
        0x16 => Some(InstructionInfo::new(ASL, DirectPageX, 6)),
        0x17 => Some(InstructionInfo::new(ORA, DirectPageIndirectLongY, 6)),
        0x18 => Some(InstructionInfo::new(CLC, Implied, 2)),
        0x19 => Some(InstructionInfo::new(ORA, AbsoluteY, 4)),
        0x1A => Some(InstructionInfo::new(INC, Accumulator, 2)),
        0x1B => Some(InstructionInfo::new(TCS, Implied, 2)),
        0x1C => Some(InstructionInfo::new(TRB, Absolute, 6)),
        0x1D => Some(InstructionInfo::new(ORA, AbsoluteX, 4)),
        0x1E => Some(InstructionInfo::new(ASL, AbsoluteX, 7)),
        0x1F => Some(InstructionInfo::new(ORA, AbsoluteLongX, 5)),
        
        // 0x20-0x2F
        0x20 => Some(InstructionInfo::new(JSR, Absolute, 6)),
        0x21 => Some(InstructionInfo::new(AND, DirectPageIndirectX, 6)),
        0x22 => Some(InstructionInfo::new(JSL, AbsoluteLong, 8)),
        0x23 => Some(InstructionInfo::new(AND, StackRelative, 4)),
        0x24 => Some(InstructionInfo::new(BIT, DirectPage, 3)),
        0x25 => Some(InstructionInfo::new(AND, DirectPage, 3)),
        0x26 => Some(InstructionInfo::new(ROL, DirectPage, 5)),
        0x27 => Some(InstructionInfo::new(AND, DirectPageIndirectLong, 6)),
        0x28 => Some(InstructionInfo::new(PLP, Implied, 4)),
        0x29 => Some(InstructionInfo::new(AND, Immediate, 2)),
        0x2A => Some(InstructionInfo::new(ROL, Accumulator, 2)),
        0x2B => Some(InstructionInfo::new(PLD, Implied, 5)),
        0x2C => Some(InstructionInfo::new(BIT, Absolute, 4)),
        0x2D => Some(InstructionInfo::new(AND, Absolute, 4)),
        0x2E => Some(InstructionInfo::new(ROL, Absolute, 6)),
        0x2F => Some(InstructionInfo::new(AND, AbsoluteLong, 5)),
        
        // 0x30-0x3F
        0x30 => Some(InstructionInfo::new(BMI, Relative, 2)),
        0x31 => Some(InstructionInfo::new(AND, DirectPageIndirectY, 5)),
        0x32 => Some(InstructionInfo::new(AND, DirectPageIndirect, 5)),
        0x33 => Some(InstructionInfo::new(AND, StackRelativeIndirectY, 7)),
        0x34 => Some(InstructionInfo::new(BIT, DirectPageX, 4)),
        0x35 => Some(InstructionInfo::new(AND, DirectPageX, 4)),
        0x36 => Some(InstructionInfo::new(ROL, DirectPageX, 6)),
        0x37 => Some(InstructionInfo::new(AND, DirectPageIndirectLongY, 6)),
        0x38 => Some(InstructionInfo::new(SEC, Implied, 2)),
        0x39 => Some(InstructionInfo::new(AND, AbsoluteY, 4)),
        0x3A => Some(InstructionInfo::new(DEC, Accumulator, 2)),
        0x3B => Some(InstructionInfo::new(TSC, Implied, 2)),
        0x3C => Some(InstructionInfo::new(BIT, AbsoluteX, 4)),
        0x3D => Some(InstructionInfo::new(AND, AbsoluteX, 4)),
        0x3E => Some(InstructionInfo::new(ROL, AbsoluteX, 7)),
        0x3F => Some(InstructionInfo::new(AND, AbsoluteLongX, 5)),
        
        // TODO: Implement remaining opcodes 0x40-0xFF
        // For now, return None for unimplemented opcodes
        _ => None,
    }
}