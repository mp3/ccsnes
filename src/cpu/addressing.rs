use crate::memory::Bus;
use crate::cpu::registers::CpuRegisters;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddressingMode {
    // Basic modes
    Implied,                    // IMP - No operand
    Accumulator,               // ACC - Operates on accumulator
    Immediate,                 // IMM - #$nn or #$nnnn
    
    // Zero page / Direct page modes  
    DirectPage,                // DP  - $nn (Direct Page)
    DirectPageX,               // DPX - $nn,X
    DirectPageY,               // DPY - $nn,Y
    DirectPageIndirect,        // DPI - ($nn)
    DirectPageIndirectX,       // DIX - ($nn,X)
    DirectPageIndirectY,       // DIY - ($nn),Y
    DirectPageIndirectLong,    // DIL - [$nn]
    DirectPageIndirectLongY,   // DILY - [$nn],Y
    
    // Absolute modes
    Absolute,                  // ABS - $nnnn
    AbsoluteX,                 // ABX - $nnnn,X
    AbsoluteY,                 // ABY - $nnnn,Y
    AbsoluteIndirect,          // ABI - ($nnnn)
    AbsoluteIndirectX,         // ABIX - ($nnnn,X)
    AbsoluteLong,              // ABL - $nnnnnn
    AbsoluteLongX,             // ABLX - $nnnnnn,X
    AbsoluteIndirectLong,      // ABIL - [$nnnn]
    
    // Stack modes
    StackRelative,             // SR  - $nn,S
    StackRelativeIndirectY,    // SRIY - ($nn,S),Y
    
    // Block move modes
    BlockMove,                 // BLOCK - $nn,$mm (for MVP/MVN)
    
    // Relative modes
    Relative,                  // REL - $nn (8-bit relative)
    RelativeLong,              // RELL - $nnnn (16-bit relative)
}

#[derive(Debug, Clone)]
pub struct AddressingResult {
    pub address: u32,
    pub value: u16,
    pub cycles: u32,
    pub crossed_page: bool,
}

impl AddressingMode {
    pub fn resolve(&self, cpu: &mut CpuRegisters, bus: &mut Bus) -> AddressingResult {
        match self {
            AddressingMode::Implied => {
                AddressingResult {
                    address: 0,
                    value: 0,
                    cycles: 0,
                    crossed_page: false,
                }
            }

            AddressingMode::Accumulator => {
                AddressingResult {
                    address: 0,
                    value: cpu.get_a(),
                    cycles: 0,
                    crossed_page: false,
                }
            }

            AddressingMode::Immediate => {
                let pc = cpu.pc;
                let value = if cpu.memory_width() {
                    // 8-bit immediate
                    let val = bus.read8(pc) as u16;
                    cpu.increment_pc(1);
                    val
                } else {
                    // 16-bit immediate
                    let val = bus.read16(pc);
                    cpu.increment_pc(2);
                    val
                };
                
                AddressingResult {
                    address: pc,
                    value,
                    cycles: 0,
                    crossed_page: false,
                }
            }

            AddressingMode::DirectPage => {
                let offset = bus.read8(cpu.pc) as u16;
                cpu.increment_pc(1);
                let address = (cpu.d + offset) as u32;
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                let cycles = if cpu.d & 0xFF != 0 { 1 } else { 0 }; // +1 cycle if D is not page-aligned
                
                AddressingResult {
                    address,
                    value,
                    cycles,
                    crossed_page: false,
                }
            }

            AddressingMode::DirectPageX => {
                let offset = bus.read8(cpu.pc) as u16;
                cpu.increment_pc(1);
                let address = (cpu.d + offset + cpu.get_x()) as u32;
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                let cycles = if cpu.d & 0xFF != 0 { 1 } else { 0 }; // +1 cycle if D is not page-aligned
                
                AddressingResult {
                    address,
                    value,
                    cycles,
                    crossed_page: false,
                }
            }

            AddressingMode::DirectPageY => {
                let offset = bus.read8(cpu.pc) as u16;
                cpu.increment_pc(1);
                let address = (cpu.d + offset + cpu.get_y()) as u32;
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                let cycles = if cpu.d & 0xFF != 0 { 1 } else { 0 }; // +1 cycle if D is not page-aligned
                
                AddressingResult {
                    address,
                    value,
                    cycles,
                    crossed_page: false,
                }
            }

            AddressingMode::DirectPageIndirect => {
                let offset = bus.read8(cpu.pc) as u16;
                cpu.increment_pc(1);
                let pointer_addr = (cpu.d + offset) as u32;
                let address = bus.read16(pointer_addr) as u32 | ((cpu.db as u32) << 16);
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                let cycles = if cpu.d & 0xFF != 0 { 1 } else { 0 }; // +1 cycle if D is not page-aligned
                
                AddressingResult {
                    address,
                    value,
                    cycles,
                    crossed_page: false,
                }
            }

            AddressingMode::DirectPageIndirectX => {
                let offset = bus.read8(cpu.pc) as u16;
                cpu.increment_pc(1);
                let pointer_addr = (cpu.d + offset + cpu.get_x()) as u32;
                let address = bus.read16(pointer_addr) as u32 | ((cpu.db as u32) << 16);
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                let cycles = if cpu.d & 0xFF != 0 { 1 } else { 0 }; // +1 cycle if D is not page-aligned
                
                AddressingResult {
                    address,
                    value,
                    cycles,
                    crossed_page: false,
                }
            }

            AddressingMode::DirectPageIndirectY => {
                let offset = bus.read8(cpu.pc) as u16;
                cpu.increment_pc(1);
                let pointer_addr = (cpu.d + offset) as u32;
                let base_address = bus.read16(pointer_addr) as u32 | ((cpu.db as u32) << 16);
                let address = base_address + cpu.get_y() as u32;
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                let crossed_page = (base_address & 0xFF00) != (address & 0xFF00);
                let mut cycles = if cpu.d & 0xFF != 0 { 1 } else { 0 }; // +1 cycle if D is not page-aligned
                if crossed_page { cycles += 1; } // +1 cycle if page crossed
                
                AddressingResult {
                    address,
                    value,
                    cycles,
                    crossed_page,
                }
            }

            AddressingMode::DirectPageIndirectLong => {
                let offset = bus.read8(cpu.pc) as u16;
                cpu.increment_pc(1);
                let pointer_addr = (cpu.d + offset) as u32;
                let address = bus.read24(pointer_addr);
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                let cycles = if cpu.d & 0xFF != 0 { 1 } else { 0 }; // +1 cycle if D is not page-aligned
                
                AddressingResult {
                    address,
                    value,
                    cycles,
                    crossed_page: false,
                }
            }

            AddressingMode::DirectPageIndirectLongY => {
                let offset = bus.read8(cpu.pc) as u16;
                cpu.increment_pc(1);
                let pointer_addr = (cpu.d + offset) as u32;
                let base_address = bus.read24(pointer_addr);
                let address = base_address + cpu.get_y() as u32;
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                let cycles = if cpu.d & 0xFF != 0 { 1 } else { 0 }; // +1 cycle if D is not page-aligned
                
                AddressingResult {
                    address,
                    value,
                    cycles,
                    crossed_page: false,
                }
            }

            AddressingMode::Absolute => {
                let address = bus.read16(cpu.pc) as u32 | ((cpu.db as u32) << 16);
                cpu.increment_pc(2);
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                AddressingResult {
                    address,
                    value,
                    cycles: 0,
                    crossed_page: false,
                }
            }

            AddressingMode::AbsoluteX => {
                let base_address = bus.read16(cpu.pc) as u32 | ((cpu.db as u32) << 16);
                cpu.increment_pc(2);
                let address = base_address + cpu.get_x() as u32;
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                let crossed_page = (base_address & 0xFF00) != (address & 0xFF00);
                let cycles = if crossed_page { 1 } else { 0 }; // +1 cycle if page crossed
                
                AddressingResult {
                    address,
                    value,
                    cycles,
                    crossed_page,
                }
            }

            AddressingMode::AbsoluteY => {
                let base_address = bus.read16(cpu.pc) as u32 | ((cpu.db as u32) << 16);
                cpu.increment_pc(2);
                let address = base_address + cpu.get_y() as u32;
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                let crossed_page = (base_address & 0xFF00) != (address & 0xFF00);
                let cycles = if crossed_page { 1 } else { 0 }; // +1 cycle if page crossed
                
                AddressingResult {
                    address,
                    value,
                    cycles,
                    crossed_page,
                }
            }

            AddressingMode::AbsoluteIndirect => {
                let pointer_addr = bus.read16(cpu.pc) as u32;
                cpu.increment_pc(2);
                let address = bus.read16(pointer_addr) as u32;
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                AddressingResult {
                    address,
                    value,
                    cycles: 0,
                    crossed_page: false,
                }
            }

            AddressingMode::AbsoluteIndirectX => {
                let base_pointer = bus.read16(cpu.pc) as u32;
                cpu.increment_pc(2);
                let pointer_addr = base_pointer + cpu.get_x() as u32;
                let address = bus.read16(pointer_addr) as u32 | ((cpu.get_pc_bank() as u32) << 16);
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                AddressingResult {
                    address,
                    value,
                    cycles: 0,
                    crossed_page: false,
                }
            }

            AddressingMode::AbsoluteLong => {
                let address = bus.read24(cpu.pc);
                cpu.increment_pc(3);
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                AddressingResult {
                    address,
                    value,
                    cycles: 0,
                    crossed_page: false,
                }
            }

            AddressingMode::AbsoluteLongX => {
                let base_address = bus.read24(cpu.pc);
                cpu.increment_pc(3);
                let address = base_address + cpu.get_x() as u32;
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                AddressingResult {
                    address,
                    value,
                    cycles: 0,
                    crossed_page: false,
                }
            }

            AddressingMode::AbsoluteIndirectLong => {
                let pointer_addr = bus.read16(cpu.pc) as u32;
                cpu.increment_pc(2);
                let address = bus.read24(pointer_addr);
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                AddressingResult {
                    address,
                    value,
                    cycles: 0,
                    crossed_page: false,
                }
            }

            AddressingMode::StackRelative => {
                let offset = bus.read8(cpu.pc) as u16;
                cpu.increment_pc(1);
                let address = (cpu.s + offset) as u32;
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                AddressingResult {
                    address,
                    value,
                    cycles: 0,
                    crossed_page: false,
                }
            }

            AddressingMode::StackRelativeIndirectY => {
                let offset = bus.read8(cpu.pc) as u16;
                cpu.increment_pc(1);
                let pointer_addr = (cpu.s + offset) as u32;
                let base_address = bus.read16(pointer_addr) as u32 | ((cpu.db as u32) << 16);
                let address = base_address + cpu.get_y() as u32;
                let value = if cpu.memory_width() {
                    bus.read8(address) as u16
                } else {
                    bus.read16(address)
                };
                
                AddressingResult {
                    address,
                    value,
                    cycles: 0,
                    crossed_page: false,
                }
            }

            AddressingMode::BlockMove => {
                // For MVP/MVN instructions - operands are source and destination banks
                let src_bank = bus.read8(cpu.pc) as u16;
                let dst_bank = bus.read8(cpu.pc + 1) as u16;
                cpu.increment_pc(2);
                
                AddressingResult {
                    address: ((dst_bank as u32) << 16) | (src_bank as u32),
                    value: src_bank | (dst_bank << 8),
                    cycles: 0,
                    crossed_page: false,
                }
            }

            AddressingMode::Relative => {
                let offset = bus.read8(cpu.pc) as i8 as i32;
                cpu.increment_pc(1);
                let address = (cpu.pc as i32 + offset) as u32 & 0xFFFFFF;
                
                AddressingResult {
                    address,
                    value: 0,
                    cycles: 0,
                    crossed_page: (cpu.pc & 0xFF00) != (address & 0xFF00),
                }
            }

            AddressingMode::RelativeLong => {
                let offset = bus.read16(cpu.pc) as i16 as i32;
                cpu.increment_pc(2);
                let address = (cpu.pc as i32 + offset) as u32 & 0xFFFFFF;
                
                AddressingResult {
                    address,
                    value: 0,
                    cycles: 0,
                    crossed_page: false,
                }
            }
        }
    }

    pub fn write_result(&self, cpu: &mut CpuRegisters, bus: &mut Bus, result: &AddressingResult, value: u16) {
        match self {
            AddressingMode::Accumulator => {
                cpu.set_a(value);
            }
            
            _ => {
                if cpu.memory_width() {
                    bus.write8(result.address, (value & 0xFF) as u8);
                } else {
                    bus.write16(result.address, value);
                }
            }
        }
    }

    pub fn get_operand_size(&self, cpu: &CpuRegisters) -> u8 {
        match self {
            AddressingMode::Implied | AddressingMode::Accumulator => 0,
            AddressingMode::Immediate => {
                if cpu.memory_width() { 1 } else { 2 }
            }
            AddressingMode::DirectPage | AddressingMode::DirectPageX | AddressingMode::DirectPageY |
            AddressingMode::DirectPageIndirect | AddressingMode::DirectPageIndirectX |
            AddressingMode::DirectPageIndirectY | AddressingMode::DirectPageIndirectLong |
            AddressingMode::DirectPageIndirectLongY | AddressingMode::StackRelative |
            AddressingMode::StackRelativeIndirectY | AddressingMode::Relative => 1,
            
            AddressingMode::Absolute | AddressingMode::AbsoluteX | AddressingMode::AbsoluteY |
            AddressingMode::AbsoluteIndirect | AddressingMode::AbsoluteIndirectX |
            AddressingMode::AbsoluteIndirectLong | AddressingMode::RelativeLong => 2,
            
            AddressingMode::BlockMove => 2,
            AddressingMode::AbsoluteLong | AddressingMode::AbsoluteLongX => 3,
        }
    }
}