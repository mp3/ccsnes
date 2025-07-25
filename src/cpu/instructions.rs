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

#[derive(Debug, Clone, Copy)]
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
        
        // 0x40-0x4F
        0x40 => Some(InstructionInfo::new(RTI, Implied, 6)),
        0x41 => Some(InstructionInfo::new(EOR, DirectPageIndirectX, 6)),
        0x42 => Some(InstructionInfo::new(WDM, Immediate, 2)),
        0x43 => Some(InstructionInfo::new(EOR, StackRelative, 4)),
        0x44 => Some(InstructionInfo::new(MVP, BlockMove, 7)),
        0x45 => Some(InstructionInfo::new(EOR, DirectPage, 3)),
        0x46 => Some(InstructionInfo::new(LSR, DirectPage, 5)),
        0x47 => Some(InstructionInfo::new(EOR, DirectPageIndirectLong, 6)),
        0x48 => Some(InstructionInfo::new(PHA, Implied, 3)),
        0x49 => Some(InstructionInfo::new(EOR, Immediate, 2)),
        0x4A => Some(InstructionInfo::new(LSR, Accumulator, 2)),
        0x4B => Some(InstructionInfo::new(PHK, Implied, 3)),
        0x4C => Some(InstructionInfo::new(JMP, Absolute, 3)),
        0x4D => Some(InstructionInfo::new(EOR, Absolute, 4)),
        0x4E => Some(InstructionInfo::new(LSR, Absolute, 6)),
        0x4F => Some(InstructionInfo::new(EOR, AbsoluteLong, 5)),
        
        // 0x50-0x5F
        0x50 => Some(InstructionInfo::new(BVC, Relative, 2)),
        0x51 => Some(InstructionInfo::new(EOR, DirectPageIndirectY, 5)),
        0x52 => Some(InstructionInfo::new(EOR, DirectPageIndirect, 5)),
        0x53 => Some(InstructionInfo::new(EOR, StackRelativeIndirectY, 7)),
        0x54 => Some(InstructionInfo::new(MVN, BlockMove, 7)),
        0x55 => Some(InstructionInfo::new(EOR, DirectPageX, 4)),
        0x56 => Some(InstructionInfo::new(LSR, DirectPageX, 6)),
        0x57 => Some(InstructionInfo::new(EOR, DirectPageIndirectLongY, 6)),
        0x58 => Some(InstructionInfo::new(CLI, Implied, 2)),
        0x59 => Some(InstructionInfo::new(EOR, AbsoluteY, 4)),
        0x5A => Some(InstructionInfo::new(PHY, Implied, 3)),
        0x5B => Some(InstructionInfo::new(TCD, Implied, 2)),
        0x5C => Some(InstructionInfo::new(JML, AbsoluteLong, 4)),
        0x5D => Some(InstructionInfo::new(EOR, AbsoluteX, 4)),
        0x5E => Some(InstructionInfo::new(LSR, AbsoluteX, 7)),
        0x5F => Some(InstructionInfo::new(EOR, AbsoluteLongX, 5)),
        
        // 0x60-0x6F
        0x60 => Some(InstructionInfo::new(RTS, Implied, 6)),
        0x61 => Some(InstructionInfo::new(ADC, DirectPageIndirectX, 6)),
        0x62 => Some(InstructionInfo::new(PER, RelativeLong, 6)),
        0x63 => Some(InstructionInfo::new(ADC, StackRelative, 4)),
        0x64 => Some(InstructionInfo::new(STZ, DirectPage, 3)),
        0x65 => Some(InstructionInfo::new(ADC, DirectPage, 3)),
        0x66 => Some(InstructionInfo::new(ROR, DirectPage, 5)),
        0x67 => Some(InstructionInfo::new(ADC, DirectPageIndirectLong, 6)),
        0x68 => Some(InstructionInfo::new(PLA, Implied, 4)),
        0x69 => Some(InstructionInfo::new(ADC, Immediate, 2)),
        0x6A => Some(InstructionInfo::new(ROR, Accumulator, 2)),
        0x6B => Some(InstructionInfo::new(RTL, Implied, 6)),
        0x6C => Some(InstructionInfo::new(JMP, AbsoluteIndirect, 5)),
        0x6D => Some(InstructionInfo::new(ADC, Absolute, 4)),
        0x6E => Some(InstructionInfo::new(ROR, Absolute, 6)),
        0x6F => Some(InstructionInfo::new(ADC, AbsoluteLong, 5)),
        
        // 0x70-0x7F
        0x70 => Some(InstructionInfo::new(BVS, Relative, 2)),
        0x71 => Some(InstructionInfo::new(ADC, DirectPageIndirectY, 5)),
        0x72 => Some(InstructionInfo::new(ADC, DirectPageIndirect, 5)),
        0x73 => Some(InstructionInfo::new(ADC, StackRelativeIndirectY, 7)),
        0x74 => Some(InstructionInfo::new(STZ, DirectPageX, 4)),
        0x75 => Some(InstructionInfo::new(ADC, DirectPageX, 4)),
        0x76 => Some(InstructionInfo::new(ROR, DirectPageX, 6)),
        0x77 => Some(InstructionInfo::new(ADC, DirectPageIndirectLongY, 6)),
        0x78 => Some(InstructionInfo::new(SEI, Implied, 2)),
        0x79 => Some(InstructionInfo::new(ADC, AbsoluteY, 4)),
        0x7A => Some(InstructionInfo::new(PLY, Implied, 4)),
        0x7B => Some(InstructionInfo::new(TDC, Implied, 2)),
        0x7C => Some(InstructionInfo::new(JMP, AbsoluteIndirectX, 6)),
        0x7D => Some(InstructionInfo::new(ADC, AbsoluteX, 4)),
        0x7E => Some(InstructionInfo::new(ROR, AbsoluteX, 7)),
        0x7F => Some(InstructionInfo::new(ADC, AbsoluteLongX, 5)),
        
        // 0x80-0x8F
        0x80 => Some(InstructionInfo::new(BRA, Relative, 3)),
        0x81 => Some(InstructionInfo::new(STA, DirectPageIndirectX, 6)),
        0x82 => Some(InstructionInfo::new(BRL, RelativeLong, 4)),
        0x83 => Some(InstructionInfo::new(STA, StackRelative, 4)),
        0x84 => Some(InstructionInfo::new(STY, DirectPage, 3)),
        0x85 => Some(InstructionInfo::new(STA, DirectPage, 3)),
        0x86 => Some(InstructionInfo::new(STX, DirectPage, 3)),
        0x87 => Some(InstructionInfo::new(STA, DirectPageIndirectLong, 6)),
        0x88 => Some(InstructionInfo::new(DEY, Implied, 2)),
        0x89 => Some(InstructionInfo::new(BIT, Immediate, 2)),
        0x8A => Some(InstructionInfo::new(TXA, Implied, 2)),
        0x8B => Some(InstructionInfo::new(PHB, Implied, 3)),
        0x8C => Some(InstructionInfo::new(STY, Absolute, 4)),
        0x8D => Some(InstructionInfo::new(STA, Absolute, 4)),
        0x8E => Some(InstructionInfo::new(STX, Absolute, 4)),
        0x8F => Some(InstructionInfo::new(STA, AbsoluteLong, 5)),
        
        // 0x90-0x9F
        0x90 => Some(InstructionInfo::new(BCC, Relative, 2)),
        0x91 => Some(InstructionInfo::new(STA, DirectPageIndirectY, 6)),
        0x92 => Some(InstructionInfo::new(STA, DirectPageIndirect, 5)),
        0x93 => Some(InstructionInfo::new(STA, StackRelativeIndirectY, 7)),
        0x94 => Some(InstructionInfo::new(STY, DirectPageX, 4)),
        0x95 => Some(InstructionInfo::new(STA, DirectPageX, 4)),
        0x96 => Some(InstructionInfo::new(STX, DirectPageY, 4)),
        0x97 => Some(InstructionInfo::new(STA, DirectPageIndirectLongY, 6)),
        0x98 => Some(InstructionInfo::new(TYA, Implied, 2)),
        0x99 => Some(InstructionInfo::new(STA, AbsoluteY, 5)),
        0x9A => Some(InstructionInfo::new(TXS, Implied, 2)),
        0x9B => Some(InstructionInfo::new(TXY, Implied, 2)),
        0x9C => Some(InstructionInfo::new(STZ, Absolute, 4)),
        0x9D => Some(InstructionInfo::new(STA, AbsoluteX, 5)),
        0x9E => Some(InstructionInfo::new(STZ, AbsoluteX, 5)),
        0x9F => Some(InstructionInfo::new(STA, AbsoluteLongX, 5)),
        
        // 0xA0-0xAF
        0xA0 => Some(InstructionInfo::new(LDY, Immediate, 2)),
        0xA1 => Some(InstructionInfo::new(LDA, DirectPageIndirectX, 6)),
        0xA2 => Some(InstructionInfo::new(LDX, Immediate, 2)),
        0xA3 => Some(InstructionInfo::new(LDA, StackRelative, 4)),
        0xA4 => Some(InstructionInfo::new(LDY, DirectPage, 3)),
        0xA5 => Some(InstructionInfo::new(LDA, DirectPage, 3)),
        0xA6 => Some(InstructionInfo::new(LDX, DirectPage, 3)),
        0xA7 => Some(InstructionInfo::new(LDA, DirectPageIndirectLong, 6)),
        0xA8 => Some(InstructionInfo::new(TAY, Implied, 2)),
        0xA9 => Some(InstructionInfo::new(LDA, Immediate, 2)),
        0xAA => Some(InstructionInfo::new(TAX, Implied, 2)),
        0xAB => Some(InstructionInfo::new(PLB, Implied, 4)),
        0xAC => Some(InstructionInfo::new(LDY, Absolute, 4)),
        0xAD => Some(InstructionInfo::new(LDA, Absolute, 4)),
        0xAE => Some(InstructionInfo::new(LDX, Absolute, 4)),
        0xAF => Some(InstructionInfo::new(LDA, AbsoluteLong, 5)),
        
        // 0xB0-0xBF
        0xB0 => Some(InstructionInfo::new(BCS, Relative, 2)),
        0xB1 => Some(InstructionInfo::new(LDA, DirectPageIndirectY, 5)),
        0xB2 => Some(InstructionInfo::new(LDA, DirectPageIndirect, 5)),
        0xB3 => Some(InstructionInfo::new(LDA, StackRelativeIndirectY, 7)),
        0xB4 => Some(InstructionInfo::new(LDY, DirectPageX, 4)),
        0xB5 => Some(InstructionInfo::new(LDA, DirectPageX, 4)),
        0xB6 => Some(InstructionInfo::new(LDX, DirectPageY, 4)),
        0xB7 => Some(InstructionInfo::new(LDA, DirectPageIndirectLongY, 6)),
        0xB8 => Some(InstructionInfo::new(CLV, Implied, 2)),
        0xB9 => Some(InstructionInfo::new(LDA, AbsoluteY, 4)),
        0xBA => Some(InstructionInfo::new(TSX, Implied, 2)),
        0xBB => Some(InstructionInfo::new(TYX, Implied, 2)),
        0xBC => Some(InstructionInfo::new(LDY, AbsoluteX, 4)),
        0xBD => Some(InstructionInfo::new(LDA, AbsoluteX, 4)),
        0xBE => Some(InstructionInfo::new(LDX, AbsoluteY, 4)),
        0xBF => Some(InstructionInfo::new(LDA, AbsoluteLongX, 5)),
        
        // 0xC0-0xCF
        0xC0 => Some(InstructionInfo::new(CPY, Immediate, 2)),
        0xC1 => Some(InstructionInfo::new(CMP, DirectPageIndirectX, 6)),
        0xC2 => Some(InstructionInfo::new(REP, Immediate, 3)),
        0xC3 => Some(InstructionInfo::new(CMP, StackRelative, 4)),
        0xC4 => Some(InstructionInfo::new(CPY, DirectPage, 3)),
        0xC5 => Some(InstructionInfo::new(CMP, DirectPage, 3)),
        0xC6 => Some(InstructionInfo::new(DEC, DirectPage, 5)),
        0xC7 => Some(InstructionInfo::new(CMP, DirectPageIndirectLong, 6)),
        0xC8 => Some(InstructionInfo::new(INY, Implied, 2)),
        0xC9 => Some(InstructionInfo::new(CMP, Immediate, 2)),
        0xCA => Some(InstructionInfo::new(DEX, Implied, 2)),
        0xCB => Some(InstructionInfo::new(WAI, Implied, 3)),
        0xCC => Some(InstructionInfo::new(CPY, Absolute, 4)),
        0xCD => Some(InstructionInfo::new(CMP, Absolute, 4)),
        0xCE => Some(InstructionInfo::new(DEC, Absolute, 6)),
        0xCF => Some(InstructionInfo::new(CMP, AbsoluteLong, 5)),
        
        // 0xD0-0xDF
        0xD0 => Some(InstructionInfo::new(BNE, Relative, 2)),
        0xD1 => Some(InstructionInfo::new(CMP, DirectPageIndirectY, 5)),
        0xD2 => Some(InstructionInfo::new(CMP, DirectPageIndirect, 5)),
        0xD3 => Some(InstructionInfo::new(CMP, StackRelativeIndirectY, 7)),
        0xD4 => Some(InstructionInfo::new(PEI, DirectPage, 6)),
        0xD5 => Some(InstructionInfo::new(CMP, DirectPageX, 4)),
        0xD6 => Some(InstructionInfo::new(DEC, DirectPageX, 6)),
        0xD7 => Some(InstructionInfo::new(CMP, DirectPageIndirectLongY, 6)),
        0xD8 => Some(InstructionInfo::new(CLD, Implied, 2)),
        0xD9 => Some(InstructionInfo::new(CMP, AbsoluteY, 4)),
        0xDA => Some(InstructionInfo::new(PHX, Implied, 3)),
        0xDB => Some(InstructionInfo::new(STP, Implied, 3)),
        0xDC => Some(InstructionInfo::new(JML, AbsoluteIndirectLong, 6)),
        0xDD => Some(InstructionInfo::new(CMP, AbsoluteX, 4)),
        0xDE => Some(InstructionInfo::new(DEC, AbsoluteX, 7)),
        0xDF => Some(InstructionInfo::new(CMP, AbsoluteLongX, 5)),
        
        // 0xE0-0xEF
        0xE0 => Some(InstructionInfo::new(CPX, Immediate, 2)),
        0xE1 => Some(InstructionInfo::new(SBC, DirectPageIndirectX, 6)),
        0xE2 => Some(InstructionInfo::new(SEP, Immediate, 3)),
        0xE3 => Some(InstructionInfo::new(SBC, StackRelative, 4)),
        0xE4 => Some(InstructionInfo::new(CPX, DirectPage, 3)),
        0xE5 => Some(InstructionInfo::new(SBC, DirectPage, 3)),
        0xE6 => Some(InstructionInfo::new(INC, DirectPage, 5)),
        0xE7 => Some(InstructionInfo::new(SBC, DirectPageIndirectLong, 6)),
        0xE8 => Some(InstructionInfo::new(INX, Implied, 2)),
        0xE9 => Some(InstructionInfo::new(SBC, Immediate, 2)),
        0xEA => Some(InstructionInfo::new(NOP, Implied, 2)),
        0xEB => Some(InstructionInfo::new(XBA, Implied, 3)),
        0xEC => Some(InstructionInfo::new(CPX, Absolute, 4)),
        0xED => Some(InstructionInfo::new(SBC, Absolute, 4)),
        0xEE => Some(InstructionInfo::new(INC, Absolute, 6)),
        0xEF => Some(InstructionInfo::new(SBC, AbsoluteLong, 5)),
        
        // 0xF0-0xFF
        0xF0 => Some(InstructionInfo::new(BEQ, Relative, 2)),
        0xF1 => Some(InstructionInfo::new(SBC, DirectPageIndirectY, 5)),
        0xF2 => Some(InstructionInfo::new(SBC, DirectPageIndirect, 5)),
        0xF3 => Some(InstructionInfo::new(SBC, StackRelativeIndirectY, 7)),
        0xF4 => Some(InstructionInfo::new(PEA, Absolute, 5)),
        0xF5 => Some(InstructionInfo::new(SBC, DirectPageX, 4)),
        0xF6 => Some(InstructionInfo::new(INC, DirectPageX, 6)),
        0xF7 => Some(InstructionInfo::new(SBC, DirectPageIndirectLongY, 6)),
        0xF8 => Some(InstructionInfo::new(SED, Implied, 2)),
        0xF9 => Some(InstructionInfo::new(SBC, AbsoluteY, 4)),
        0xFA => Some(InstructionInfo::new(PLX, Implied, 4)),
        0xFB => Some(InstructionInfo::new(XCE, Implied, 2)),
        0xFC => Some(InstructionInfo::new(JSR, AbsoluteIndirectX, 8)),
        0xFD => Some(InstructionInfo::new(SBC, AbsoluteX, 4)),
        0xFE => Some(InstructionInfo::new(INC, AbsoluteX, 7)),
        0xFF => Some(InstructionInfo::new(SBC, AbsoluteLongX, 5)),
    }
}