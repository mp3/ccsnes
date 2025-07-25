// Optimized instruction decode table using static lookup
use crate::cpu::instructions::{Instruction, InstructionInfo};
use crate::cpu::addressing::AddressingMode;
use once_cell::sync::Lazy;

// Pre-computed instruction decode table for O(1) lookup
pub static DECODE_TABLE: Lazy<[Option<InstructionInfo>; 256]> = Lazy::new(|| {
    use Instruction::*;
    use AddressingMode::*;
    
    let mut table: [Option<InstructionInfo>; 256] = [None; 256];
    
    // Macro to simplify table initialization
    macro_rules! set_opcode {
        ($opcode:expr, $instr:expr, $mode:expr, $cycles:expr) => {
            table[$opcode] = Some(InstructionInfo::new($instr, $mode, $cycles));
        };
    }
    
    // 0x00-0x0F
    set_opcode!(0x00, BRK, Implied, 7);
    set_opcode!(0x01, ORA, DirectPageIndirectX, 6);
    set_opcode!(0x02, COP, Immediate, 7);
    set_opcode!(0x03, ORA, StackRelative, 4);
    set_opcode!(0x04, TSB, DirectPage, 5);
    set_opcode!(0x05, ORA, DirectPage, 3);
    set_opcode!(0x06, ASL, DirectPage, 5);
    set_opcode!(0x07, ORA, DirectPageIndirectLong, 6);
    set_opcode!(0x08, PHP, Implied, 3);
    set_opcode!(0x09, ORA, Immediate, 2);
    set_opcode!(0x0A, ASL, Accumulator, 2);
    set_opcode!(0x0B, PHD, Implied, 4);
    set_opcode!(0x0C, TSB, Absolute, 6);
    set_opcode!(0x0D, ORA, Absolute, 4);
    set_opcode!(0x0E, ASL, Absolute, 6);
    set_opcode!(0x0F, ORA, AbsoluteLong, 5);
    
    // 0x10-0x1F
    set_opcode!(0x10, BPL, Relative, 2);
    set_opcode!(0x11, ORA, DirectPageIndirectY, 5);
    set_opcode!(0x12, ORA, DirectPageIndirect, 5);
    set_opcode!(0x13, ORA, StackRelativeIndirectY, 7);
    set_opcode!(0x14, TRB, DirectPage, 5);
    set_opcode!(0x15, ORA, DirectPageX, 4);
    set_opcode!(0x16, ASL, DirectPageX, 6);
    set_opcode!(0x17, ORA, DirectPageIndirectLongY, 6);
    set_opcode!(0x18, CLC, Implied, 2);
    set_opcode!(0x19, ORA, AbsoluteY, 4);
    set_opcode!(0x1A, INC, Accumulator, 2);
    set_opcode!(0x1B, TCS, Implied, 2);
    set_opcode!(0x1C, TRB, Absolute, 6);
    set_opcode!(0x1D, ORA, AbsoluteX, 4);
    set_opcode!(0x1E, ASL, AbsoluteX, 7);
    set_opcode!(0x1F, ORA, AbsoluteLongX, 5);
    
    // 0x20-0x2F
    set_opcode!(0x20, JSR, Absolute, 6);
    set_opcode!(0x21, AND, DirectPageIndirectX, 6);
    set_opcode!(0x22, JSL, AbsoluteLong, 8);
    set_opcode!(0x23, AND, StackRelative, 4);
    set_opcode!(0x24, BIT, DirectPage, 3);
    set_opcode!(0x25, AND, DirectPage, 3);
    set_opcode!(0x26, ROL, DirectPage, 5);
    set_opcode!(0x27, AND, DirectPageIndirectLong, 6);
    set_opcode!(0x28, PLP, Implied, 4);
    set_opcode!(0x29, AND, Immediate, 2);
    set_opcode!(0x2A, ROL, Accumulator, 2);
    set_opcode!(0x2B, PLD, Implied, 5);
    set_opcode!(0x2C, BIT, Absolute, 4);
    set_opcode!(0x2D, AND, Absolute, 4);
    set_opcode!(0x2E, ROL, Absolute, 6);
    set_opcode!(0x2F, AND, AbsoluteLong, 5);
    
    // 0x30-0x3F
    set_opcode!(0x30, BMI, Relative, 2);
    set_opcode!(0x31, AND, DirectPageIndirectY, 5);
    set_opcode!(0x32, AND, DirectPageIndirect, 5);
    set_opcode!(0x33, AND, StackRelativeIndirectY, 7);
    set_opcode!(0x34, BIT, DirectPageX, 4);
    set_opcode!(0x35, AND, DirectPageX, 4);
    set_opcode!(0x36, ROL, DirectPageX, 6);
    set_opcode!(0x37, AND, DirectPageIndirectLongY, 6);
    set_opcode!(0x38, SEC, Implied, 2);
    set_opcode!(0x39, AND, AbsoluteY, 4);
    set_opcode!(0x3A, DEC, Accumulator, 2);
    set_opcode!(0x3B, TSC, Implied, 2);
    set_opcode!(0x3C, BIT, AbsoluteX, 4);
    set_opcode!(0x3D, AND, AbsoluteX, 4);
    set_opcode!(0x3E, ROL, AbsoluteX, 7);
    set_opcode!(0x3F, AND, AbsoluteLongX, 5);
    
    // 0x40-0x4F
    set_opcode!(0x40, RTI, Implied, 6);
    set_opcode!(0x41, EOR, DirectPageIndirectX, 6);
    set_opcode!(0x42, WDM, Immediate, 2);
    set_opcode!(0x43, EOR, StackRelative, 4);
    set_opcode!(0x44, MVP, BlockMove, 7);
    set_opcode!(0x45, EOR, DirectPage, 3);
    set_opcode!(0x46, LSR, DirectPage, 5);
    set_opcode!(0x47, EOR, DirectPageIndirectLong, 6);
    set_opcode!(0x48, PHA, Implied, 3);
    set_opcode!(0x49, EOR, Immediate, 2);
    set_opcode!(0x4A, LSR, Accumulator, 2);
    set_opcode!(0x4B, PHK, Implied, 3);
    set_opcode!(0x4C, JMP, Absolute, 3);
    set_opcode!(0x4D, EOR, Absolute, 4);
    set_opcode!(0x4E, LSR, Absolute, 6);
    set_opcode!(0x4F, EOR, AbsoluteLong, 5);
    
    // 0x50-0x5F
    set_opcode!(0x50, BVC, Relative, 2);
    set_opcode!(0x51, EOR, DirectPageIndirectY, 5);
    set_opcode!(0x52, EOR, DirectPageIndirect, 5);
    set_opcode!(0x53, EOR, StackRelativeIndirectY, 7);
    set_opcode!(0x54, MVN, BlockMove, 7);
    set_opcode!(0x55, EOR, DirectPageX, 4);
    set_opcode!(0x56, LSR, DirectPageX, 6);
    set_opcode!(0x57, EOR, DirectPageIndirectLongY, 6);
    set_opcode!(0x58, CLI, Implied, 2);
    set_opcode!(0x59, EOR, AbsoluteY, 4);
    set_opcode!(0x5A, PHY, Implied, 3);
    set_opcode!(0x5B, TCD, Implied, 2);
    set_opcode!(0x5C, JML, AbsoluteLong, 4);
    set_opcode!(0x5D, EOR, AbsoluteX, 4);
    set_opcode!(0x5E, LSR, AbsoluteX, 7);
    set_opcode!(0x5F, EOR, AbsoluteLongX, 5);
    
    // 0x60-0x6F
    set_opcode!(0x60, RTS, Implied, 6);
    set_opcode!(0x61, ADC, DirectPageIndirectX, 6);
    set_opcode!(0x62, PER, RelativeLong, 6);
    set_opcode!(0x63, ADC, StackRelative, 4);
    set_opcode!(0x64, STZ, DirectPage, 3);
    set_opcode!(0x65, ADC, DirectPage, 3);
    set_opcode!(0x66, ROR, DirectPage, 5);
    set_opcode!(0x67, ADC, DirectPageIndirectLong, 6);
    set_opcode!(0x68, PLA, Implied, 4);
    set_opcode!(0x69, ADC, Immediate, 2);
    set_opcode!(0x6A, ROR, Accumulator, 2);
    set_opcode!(0x6B, RTL, Implied, 6);
    set_opcode!(0x6C, JMP, AbsoluteIndirect, 5);
    set_opcode!(0x6D, ADC, Absolute, 4);
    set_opcode!(0x6E, ROR, Absolute, 6);
    set_opcode!(0x6F, ADC, AbsoluteLong, 5);
    
    // 0x70-0x7F
    set_opcode!(0x70, BVS, Relative, 2);
    set_opcode!(0x71, ADC, DirectPageIndirectY, 5);
    set_opcode!(0x72, ADC, DirectPageIndirect, 5);
    set_opcode!(0x73, ADC, StackRelativeIndirectY, 7);
    set_opcode!(0x74, STZ, DirectPageX, 4);
    set_opcode!(0x75, ADC, DirectPageX, 4);
    set_opcode!(0x76, ROR, DirectPageX, 6);
    set_opcode!(0x77, ADC, DirectPageIndirectLongY, 6);
    set_opcode!(0x78, SEI, Implied, 2);
    set_opcode!(0x79, ADC, AbsoluteY, 4);
    set_opcode!(0x7A, PLY, Implied, 4);
    set_opcode!(0x7B, TDC, Implied, 2);
    set_opcode!(0x7C, JMP, AbsoluteIndirectX, 6);
    set_opcode!(0x7D, ADC, AbsoluteX, 4);
    set_opcode!(0x7E, ROR, AbsoluteX, 7);
    set_opcode!(0x7F, ADC, AbsoluteLongX, 5);
    
    // 0x80-0x8F
    set_opcode!(0x80, BRA, Relative, 2);
    set_opcode!(0x81, STA, DirectPageIndirectX, 6);
    set_opcode!(0x82, BRL, RelativeLong, 4);
    set_opcode!(0x83, STA, StackRelative, 4);
    set_opcode!(0x84, STY, DirectPage, 3);
    set_opcode!(0x85, STA, DirectPage, 3);
    set_opcode!(0x86, STX, DirectPage, 3);
    set_opcode!(0x87, STA, DirectPageIndirectLong, 6);
    set_opcode!(0x88, DEY, Implied, 2);
    set_opcode!(0x89, BIT, Immediate, 2);
    set_opcode!(0x8A, TXA, Implied, 2);
    set_opcode!(0x8B, PHB, Implied, 3);
    set_opcode!(0x8C, STY, Absolute, 4);
    set_opcode!(0x8D, STA, Absolute, 4);
    set_opcode!(0x8E, STX, Absolute, 4);
    set_opcode!(0x8F, STA, AbsoluteLong, 5);
    
    // 0x90-0x9F
    set_opcode!(0x90, BCC, Relative, 2);
    set_opcode!(0x91, STA, DirectPageIndirectY, 6);
    set_opcode!(0x92, STA, DirectPageIndirect, 5);
    set_opcode!(0x93, STA, StackRelativeIndirectY, 7);
    set_opcode!(0x94, STY, DirectPageX, 4);
    set_opcode!(0x95, STA, DirectPageX, 4);
    set_opcode!(0x96, STX, DirectPageY, 4);
    set_opcode!(0x97, STA, DirectPageIndirectLongY, 6);
    set_opcode!(0x98, TYA, Implied, 2);
    set_opcode!(0x99, STA, AbsoluteY, 5);
    set_opcode!(0x9A, TXS, Implied, 2);
    set_opcode!(0x9B, TXY, Implied, 2);
    set_opcode!(0x9C, STZ, Absolute, 4);
    set_opcode!(0x9D, STA, AbsoluteX, 5);
    set_opcode!(0x9E, STZ, AbsoluteX, 5);
    set_opcode!(0x9F, STA, AbsoluteLongX, 5);
    
    // 0xA0-0xAF
    set_opcode!(0xA0, LDY, Immediate, 2);
    set_opcode!(0xA1, LDA, DirectPageIndirectX, 6);
    set_opcode!(0xA2, LDX, Immediate, 2);
    set_opcode!(0xA3, LDA, StackRelative, 4);
    set_opcode!(0xA4, LDY, DirectPage, 3);
    set_opcode!(0xA5, LDA, DirectPage, 3);
    set_opcode!(0xA6, LDX, DirectPage, 3);
    set_opcode!(0xA7, LDA, DirectPageIndirectLong, 6);
    set_opcode!(0xA8, TAY, Implied, 2);
    set_opcode!(0xA9, LDA, Immediate, 2);
    set_opcode!(0xAA, TAX, Implied, 2);
    set_opcode!(0xAB, PLB, Implied, 4);
    set_opcode!(0xAC, LDY, Absolute, 4);
    set_opcode!(0xAD, LDA, Absolute, 4);
    set_opcode!(0xAE, LDX, Absolute, 4);
    set_opcode!(0xAF, LDA, AbsoluteLong, 5);
    
    // 0xB0-0xBF
    set_opcode!(0xB0, BCS, Relative, 2);
    set_opcode!(0xB1, LDA, DirectPageIndirectY, 5);
    set_opcode!(0xB2, LDA, DirectPageIndirect, 5);
    set_opcode!(0xB3, LDA, StackRelativeIndirectY, 7);
    set_opcode!(0xB4, LDY, DirectPageX, 4);
    set_opcode!(0xB5, LDA, DirectPageX, 4);
    set_opcode!(0xB6, LDX, DirectPageY, 4);
    set_opcode!(0xB7, LDA, DirectPageIndirectLongY, 6);
    set_opcode!(0xB8, CLV, Implied, 2);
    set_opcode!(0xB9, LDA, AbsoluteY, 4);
    set_opcode!(0xBA, TSX, Implied, 2);
    set_opcode!(0xBB, TYX, Implied, 2);
    set_opcode!(0xBC, LDY, AbsoluteX, 4);
    set_opcode!(0xBD, LDA, AbsoluteX, 4);
    set_opcode!(0xBE, LDX, AbsoluteY, 4);
    set_opcode!(0xBF, LDA, AbsoluteLongX, 5);
    
    // 0xC0-0xCF
    set_opcode!(0xC0, CPY, Immediate, 2);
    set_opcode!(0xC1, CMP, DirectPageIndirectX, 6);
    set_opcode!(0xC2, REP, Immediate, 3);
    set_opcode!(0xC3, CMP, StackRelative, 4);
    set_opcode!(0xC4, CPY, DirectPage, 3);
    set_opcode!(0xC5, CMP, DirectPage, 3);
    set_opcode!(0xC6, DEC, DirectPage, 5);
    set_opcode!(0xC7, CMP, DirectPageIndirectLong, 6);
    set_opcode!(0xC8, INY, Implied, 2);
    set_opcode!(0xC9, CMP, Immediate, 2);
    set_opcode!(0xCA, DEX, Implied, 2);
    set_opcode!(0xCB, WAI, Implied, 3);
    set_opcode!(0xCC, CPY, Absolute, 4);
    set_opcode!(0xCD, CMP, Absolute, 4);
    set_opcode!(0xCE, DEC, Absolute, 6);
    set_opcode!(0xCF, CMP, AbsoluteLong, 5);
    
    // 0xD0-0xDF
    set_opcode!(0xD0, BNE, Relative, 2);
    set_opcode!(0xD1, CMP, DirectPageIndirectY, 5);
    set_opcode!(0xD2, CMP, DirectPageIndirect, 5);
    set_opcode!(0xD3, CMP, StackRelativeIndirectY, 7);
    set_opcode!(0xD4, PEI, DirectPageIndirect, 6);
    set_opcode!(0xD5, CMP, DirectPageX, 4);
    set_opcode!(0xD6, DEC, DirectPageX, 6);
    set_opcode!(0xD7, CMP, DirectPageIndirectLongY, 6);
    set_opcode!(0xD8, CLD, Implied, 2);
    set_opcode!(0xD9, CMP, AbsoluteY, 4);
    set_opcode!(0xDA, PHX, Implied, 3);
    set_opcode!(0xDB, STP, Implied, 3);
    set_opcode!(0xDC, JML, AbsoluteIndirect, 6);
    set_opcode!(0xDD, CMP, AbsoluteX, 4);
    set_opcode!(0xDE, DEC, AbsoluteX, 7);
    set_opcode!(0xDF, CMP, AbsoluteLongX, 5);
    
    // 0xE0-0xEF
    set_opcode!(0xE0, CPX, Immediate, 2);
    set_opcode!(0xE1, SBC, DirectPageIndirectX, 6);
    set_opcode!(0xE2, SEP, Immediate, 3);
    set_opcode!(0xE3, SBC, StackRelative, 4);
    set_opcode!(0xE4, CPX, DirectPage, 3);
    set_opcode!(0xE5, SBC, DirectPage, 3);
    set_opcode!(0xE6, INC, DirectPage, 5);
    set_opcode!(0xE7, SBC, DirectPageIndirectLong, 6);
    set_opcode!(0xE8, INX, Implied, 2);
    set_opcode!(0xE9, SBC, Immediate, 2);
    set_opcode!(0xEA, NOP, Implied, 2);
    set_opcode!(0xEB, XBA, Implied, 3);
    set_opcode!(0xEC, CPX, Absolute, 4);
    set_opcode!(0xED, SBC, Absolute, 4);
    set_opcode!(0xEE, INC, Absolute, 6);
    set_opcode!(0xEF, SBC, AbsoluteLong, 5);
    
    // 0xF0-0xFF
    set_opcode!(0xF0, BEQ, Relative, 2);
    set_opcode!(0xF1, SBC, DirectPageIndirectY, 5);
    set_opcode!(0xF2, SBC, DirectPageIndirect, 5);
    set_opcode!(0xF3, SBC, StackRelativeIndirectY, 7);
    set_opcode!(0xF4, PEA, Absolute, 5);
    set_opcode!(0xF5, SBC, DirectPageX, 4);
    set_opcode!(0xF6, INC, DirectPageX, 6);
    set_opcode!(0xF7, SBC, DirectPageIndirectLongY, 6);
    set_opcode!(0xF8, SED, Implied, 2);
    set_opcode!(0xF9, SBC, AbsoluteY, 4);
    set_opcode!(0xFA, PLX, Implied, 4);
    set_opcode!(0xFB, XCE, Implied, 2);
    set_opcode!(0xFC, JSR, AbsoluteIndirectX, 8);
    set_opcode!(0xFD, SBC, AbsoluteX, 4);
    set_opcode!(0xFE, INC, AbsoluteX, 7);
    set_opcode!(0xFF, SBC, AbsoluteLongX, 5);
    
    table
});

// Optimized decode function using static table
#[inline(always)]
pub fn decode_opcode_fast(opcode: u8) -> Option<InstructionInfo> {
    DECODE_TABLE[opcode as usize].clone()
}