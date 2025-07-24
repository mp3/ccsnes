use crate::cpu::instructions::{Instruction, InstructionInfo};
use crate::cpu::addressing::AddressingMode;
use crate::cpu::registers::CpuRegisters;
use crate::memory::Bus;
use crate::Result;

pub fn execute_instruction(
    cpu: &mut CpuRegisters,
    bus: &mut Bus,
    info: &InstructionInfo,
) -> Result<u32> {
    let addressing_result = info.addressing_mode.resolve(cpu, bus);
    let mut cycles = info.base_cycles as u32 + addressing_result.cycles;
    
    match info.instruction {
        // Load/Store Instructions
        Instruction::LDA => {
            let value = addressing_result.value;
            cpu.set_a(value);
            cpu.update_nz_flags(value);
        }
        
        Instruction::LDX => {
            let value = addressing_result.value;
            cpu.set_x(value);
            cpu.update_nz_flags_index(value);
        }
        
        Instruction::LDY => {
            let value = addressing_result.value;
            cpu.set_y(value);
            cpu.update_nz_flags_index(value);
        }
        
        Instruction::STA => {
            let value = cpu.get_a();
            info.addressing_mode.write_result(cpu, bus, &addressing_result, value);
        }
        
        Instruction::STX => {
            let value = cpu.get_x();
            info.addressing_mode.write_result(cpu, bus, &addressing_result, value);
        }
        
        Instruction::STY => {
            let value = cpu.get_y();
            info.addressing_mode.write_result(cpu, bus, &addressing_result, value);
        }
        
        Instruction::STZ => {
            info.addressing_mode.write_result(cpu, bus, &addressing_result, 0);
        }
        
        // Transfer Instructions
        Instruction::TAX => {
            let value = cpu.get_a();
            cpu.set_x(value);
            cpu.update_nz_flags_index(value);
        }
        
        Instruction::TAY => {
            let value = cpu.get_a();
            cpu.set_y(value);
            cpu.update_nz_flags_index(value);
        }
        
        Instruction::TXA => {
            let value = cpu.get_x();
            cpu.set_a(value);
            cpu.update_nz_flags(value);
        }
        
        Instruction::TYA => {
            let value = cpu.get_y();
            cpu.set_a(value);
            cpu.update_nz_flags(value);
        }
        
        Instruction::TSX => {
            let value = cpu.s;
            cpu.set_x(value);
            cpu.update_nz_flags_index(value);
        }
        
        Instruction::TXS => {
            cpu.s = cpu.get_x();
            // TXS doesn't affect flags
        }
        
        // Stack Instructions
        Instruction::PHA => {
            let value = cpu.get_a();
            if cpu.memory_width() {
                cpu.push_8(bus, value as u8);
            } else {
                cpu.push_16(bus, value);
            }
        }
        
        Instruction::PLA => {
            let value = if cpu.memory_width() {
                cpu.pop_8(bus) as u16
            } else {
                cpu.pop_16(bus)
            };
            cpu.set_a(value);
            cpu.update_nz_flags(value);
        }
        
        Instruction::PHP => {
            cpu.push_8(bus, cpu.p);
        }
        
        Instruction::PLP => {
            cpu.p = cpu.pop_8(bus);
            // In emulation mode, M and X flags are always set
            if cpu.emulation_mode {
                cpu.set_memory_width(true);
                cpu.set_index_width(true);
            }
        }
        
        Instruction::PHX => {
            let value = cpu.get_x();
            if cpu.index_width() {
                cpu.push_8(bus, value as u8);
            } else {
                cpu.push_16(bus, value);
            }
        }
        
        Instruction::PLX => {
            let value = if cpu.index_width() {
                cpu.pop_8(bus) as u16
            } else {
                cpu.pop_16(bus)
            };
            cpu.set_x(value);
            cpu.update_nz_flags_index(value);
        }
        
        Instruction::PHY => {
            let value = cpu.get_y();
            if cpu.index_width() {
                cpu.push_8(bus, value as u8);
            } else {
                cpu.push_16(bus, value);
            }
        }
        
        Instruction::PLY => {
            let value = if cpu.index_width() {
                cpu.pop_8(bus) as u16
            } else {
                cpu.pop_16(bus)
            };
            cpu.set_y(value);
            cpu.update_nz_flags_index(value);
        }
        
        // Arithmetic Instructions
        Instruction::ADC => {
            let a = cpu.get_a();
            let value = addressing_result.value;
            let carry = if cpu.carry() { 1 } else { 0 };
            
            if cpu.memory_width() {
                // 8-bit mode
                let result = (a as u8).wrapping_add(value as u8).wrapping_add(carry as u8);
                let signed_a = a as i8;
                let signed_val = value as i8;
                let signed_result = signed_a.wrapping_add(signed_val).wrapping_add(carry as i8);
                
                cpu.set_carry(result as u16 != (a & 0xFF) + (value & 0xFF) + carry);
                cpu.set_overflow((signed_a >= 0 && signed_val >= 0 && signed_result < 0) ||
                                (signed_a < 0 && signed_val < 0 && signed_result >= 0));
                cpu.set_a(result as u16);
                cpu.update_nz_flags(result as u16);
            } else {
                // 16-bit mode
                let result = a.wrapping_add(value).wrapping_add(carry);
                let signed_a = a as i16;
                let signed_val = value as i16;
                let signed_result = signed_a.wrapping_add(signed_val).wrapping_add(carry as i16);
                
                cpu.set_carry((a as u32 + value as u32 + carry as u32) > 0xFFFF);
                cpu.set_overflow((signed_a >= 0 && signed_val >= 0 && signed_result < 0) ||
                                (signed_a < 0 && signed_val < 0 && signed_result >= 0));
                cpu.set_a(result);
                cpu.update_nz_flags(result);
            }
        }
        
        Instruction::SBC => {
            let a = cpu.get_a();
            let value = addressing_result.value;
            let borrow = if cpu.carry() { 0 } else { 1 };
            
            if cpu.memory_width() {
                // 8-bit mode
                let result = (a as u8).wrapping_sub(value as u8).wrapping_sub(borrow as u8);
                let signed_a = a as i8;
                let signed_val = value as i8;
                let signed_result = signed_a.wrapping_sub(signed_val).wrapping_sub(borrow as i8);
                
                cpu.set_carry((a & 0xFF) >= ((value & 0xFF) + borrow));
                cpu.set_overflow((signed_a >= 0 && signed_val < 0 && signed_result < 0) ||
                                (signed_a < 0 && signed_val >= 0 && signed_result >= 0));
                cpu.set_a(result as u16);
                cpu.update_nz_flags(result as u16);
            } else {
                // 16-bit mode
                let result = a.wrapping_sub(value).wrapping_sub(borrow);
                let signed_a = a as i16;
                let signed_val = value as i16;
                let signed_result = signed_a.wrapping_sub(signed_val).wrapping_sub(borrow as i16);
                
                cpu.set_carry(a >= (value + borrow));
                cpu.set_overflow((signed_a >= 0 && signed_val < 0 && signed_result < 0) ||
                                (signed_a < 0 && signed_val >= 0 && signed_result >= 0));
                cpu.set_a(result);
                cpu.update_nz_flags(result);
            }
        }
        
        Instruction::INC => {
            let value = addressing_result.value.wrapping_add(1);
            if info.addressing_mode == AddressingMode::Accumulator {
                cpu.set_a(value);
                cpu.update_nz_flags(value);
            } else {
                info.addressing_mode.write_result(cpu, bus, &addressing_result, value);
                cpu.update_nz_flags(value);
            }
        }
        
        Instruction::INX => {
            let value = cpu.get_x().wrapping_add(1);
            cpu.set_x(value);
            cpu.update_nz_flags_index(value);
        }
        
        Instruction::INY => {
            let value = cpu.get_y().wrapping_add(1);
            cpu.set_y(value);
            cpu.update_nz_flags_index(value);
        }
        
        Instruction::DEC => {
            let value = addressing_result.value.wrapping_sub(1);
            if info.addressing_mode == AddressingMode::Accumulator {
                cpu.set_a(value);
                cpu.update_nz_flags(value);
            } else {
                info.addressing_mode.write_result(cpu, bus, &addressing_result, value);
                cpu.update_nz_flags(value);
            }
        }
        
        Instruction::DEX => {
            let value = cpu.get_x().wrapping_sub(1);
            cpu.set_x(value);
            cpu.update_nz_flags_index(value);
        }
        
        Instruction::DEY => {
            let value = cpu.get_y().wrapping_sub(1);
            cpu.set_y(value);
            cpu.update_nz_flags_index(value);
        }
        
        // Logic Instructions
        Instruction::AND => {
            let result = cpu.get_a() & addressing_result.value;
            cpu.set_a(result);
            cpu.update_nz_flags(result);
        }
        
        Instruction::ORA => {
            let result = cpu.get_a() | addressing_result.value;
            cpu.set_a(result);
            cpu.update_nz_flags(result);
        }
        
        Instruction::EOR => {
            let result = cpu.get_a() ^ addressing_result.value;
            cpu.set_a(result);
            cpu.update_nz_flags(result);
        }
        
        // Flag Instructions
        Instruction::CLC => {
            cpu.set_carry(false);
        }
        
        Instruction::SEC => {
            cpu.set_carry(true);
        }
        
        Instruction::CLD => {
            cpu.set_decimal(false);
        }
        
        Instruction::SED => {
            cpu.set_decimal(true);
        }
        
        Instruction::CLI => {
            cpu.set_irq_disable(false);
        }
        
        Instruction::SEI => {
            cpu.set_irq_disable(true);
        }
        
        Instruction::CLV => {
            cpu.set_overflow(false);
        }
        
        // Branch Instructions
        Instruction::BCC => {
            if !cpu.carry() {
                branch_taken(cpu, addressing_result.address, &mut cycles);
            }
        }
        
        Instruction::BCS => {
            if cpu.carry() {
                branch_taken(cpu, addressing_result.address, &mut cycles);
            }
        }
        
        Instruction::BEQ => {
            if cpu.zero() {
                branch_taken(cpu, addressing_result.address, &mut cycles);
            }
        }
        
        Instruction::BNE => {
            if !cpu.zero() {
                branch_taken(cpu, addressing_result.address, &mut cycles);
            }
        }
        
        Instruction::BMI => {
            if cpu.negative() {
                branch_taken(cpu, addressing_result.address, &mut cycles);
            }
        }
        
        Instruction::BPL => {
            if !cpu.negative() {
                branch_taken(cpu, addressing_result.address, &mut cycles);
            }
        }
        
        Instruction::BVC => {
            if !cpu.overflow() {
                branch_taken(cpu, addressing_result.address, &mut cycles);
            }
        }
        
        Instruction::BVS => {
            if cpu.overflow() {
                branch_taken(cpu, addressing_result.address, &mut cycles);
            }
        }
        
        Instruction::BRA => {
            // Branch always
            branch_taken(cpu, addressing_result.address, &mut cycles);
        }
        
        Instruction::BRL => {
            // Branch long always
            cpu.pc = addressing_result.address;
            cycles += 1; // BRL takes 1 extra cycle
        }
        
        // Jump Instructions
        Instruction::JMP => {
            cpu.pc = addressing_result.address;
        }
        
        Instruction::JML => {
            // Jump long - sets program bank too
            cpu.pc = addressing_result.address;
        }
        
        Instruction::JSR => {
            // Push return address - 1
            let return_addr = cpu.pc.wrapping_sub(1);
            cpu.push_16(bus, return_addr as u16);
            cpu.pc = addressing_result.address;
        }
        
        Instruction::JSL => {
            // Push bank and return address - 1
            let return_addr = cpu.pc.wrapping_sub(1);
            cpu.push_8(bus, cpu.get_pc_bank());
            cpu.push_16(bus, return_addr as u16);
            cpu.pc = addressing_result.address;
        }
        
        Instruction::RTS => {
            // Return from subroutine
            let return_addr = cpu.pop_16(bus);
            cpu.pc = (cpu.pc & 0xFF0000) | (return_addr as u32 + 1);
        }
        
        Instruction::RTL => {
            // Return from subroutine long
            let return_addr = cpu.pop_16(bus);
            let return_bank = cpu.pop_8(bus);
            cpu.pc = ((return_bank as u32) << 16) | (return_addr as u32 + 1);
        }
        
        Instruction::RTI => {
            // Return from interrupt
            cpu.p = cpu.pop_8(bus);
            if cpu.emulation_mode {
                // In emulation mode, force M and X flags
                cpu.set_memory_width(true);
                cpu.set_index_width(true);
            }
            
            let return_addr = cpu.pop_16(bus);
            if !cpu.emulation_mode {
                // Native mode: also pop program bank
                let return_bank = cpu.pop_8(bus);
                cpu.pc = ((return_bank as u32) << 16) | (return_addr as u32);
            } else {
                // Emulation mode: stay in current bank
                cpu.pc = (cpu.pc & 0xFF0000) | (return_addr as u32);
            }
        }
        
        // Shift/Rotate Instructions
        Instruction::ASL => {
            let value = addressing_result.value;
            let result = value << 1;
            cpu.set_carry((value & 0x8000) != 0);
            
            if info.addressing_mode == AddressingMode::Accumulator {
                cpu.set_a(result);
                cpu.update_nz_flags(result);
            } else {
                info.addressing_mode.write_result(cpu, bus, &addressing_result, result);
                cpu.update_nz_flags(result);
            }
        }
        
        Instruction::LSR => {
            let value = addressing_result.value;
            let result = value >> 1;
            cpu.set_carry((value & 0x01) != 0);
            
            if info.addressing_mode == AddressingMode::Accumulator {
                cpu.set_a(result);
                cpu.update_nz_flags(result);
            } else {
                info.addressing_mode.write_result(cpu, bus, &addressing_result, result);
                cpu.update_nz_flags(result);
            }
        }
        
        Instruction::ROL => {
            let value = addressing_result.value;
            let carry_in = if cpu.carry() { 1 } else { 0 };
            let result = (value << 1) | carry_in;
            cpu.set_carry((value & 0x8000) != 0);
            
            if info.addressing_mode == AddressingMode::Accumulator {
                cpu.set_a(result);
                cpu.update_nz_flags(result);
            } else {
                info.addressing_mode.write_result(cpu, bus, &addressing_result, result);
                cpu.update_nz_flags(result);
            }
        }
        
        Instruction::ROR => {
            let value = addressing_result.value;
            let carry_in = if cpu.carry() { 0x8000 } else { 0 };
            let result = (value >> 1) | carry_in;
            cpu.set_carry((value & 0x01) != 0);
            
            if info.addressing_mode == AddressingMode::Accumulator {
                cpu.set_a(result);
                cpu.update_nz_flags(result);
            } else {
                info.addressing_mode.write_result(cpu, bus, &addressing_result, result);
                cpu.update_nz_flags(result);
            }
        }
        
        // Compare Instructions
        Instruction::CMP => {
            let a = cpu.get_a();
            let value = addressing_result.value;
            let result = a.wrapping_sub(value);
            
            cpu.set_carry(a >= value);
            cpu.update_nz_flags(result);
        }
        
        Instruction::CPX => {
            let x = cpu.get_x();
            let value = addressing_result.value;
            let result = x.wrapping_sub(value);
            
            cpu.set_carry(x >= value);
            cpu.update_nz_flags_index(result);
        }
        
        Instruction::CPY => {
            let y = cpu.get_y();
            let value = addressing_result.value;
            let result = y.wrapping_sub(value);
            
            cpu.set_carry(y >= value);
            cpu.update_nz_flags_index(result);
        }
        
        // Bit Test Instructions
        Instruction::BIT => {
            let value = addressing_result.value;
            let result = cpu.get_a() & value;
            
            cpu.set_zero(result == 0);
            if cpu.memory_width() {
                cpu.set_negative((value & 0x80) != 0);
                cpu.set_overflow((value & 0x40) != 0);
            } else {
                cpu.set_negative((value & 0x8000) != 0);
                cpu.set_overflow((value & 0x4000) != 0);
            }
        }
        
        Instruction::TSB => {
            let value = addressing_result.value;
            let a = cpu.get_a();
            let result = value | a;
            
            cpu.set_zero((value & a) == 0);
            info.addressing_mode.write_result(cpu, bus, &addressing_result, result);
        }
        
        Instruction::TRB => {
            let value = addressing_result.value;
            let a = cpu.get_a();
            let result = value & !a;
            
            cpu.set_zero((value & a) == 0);
            info.addressing_mode.write_result(cpu, bus, &addressing_result, result);
        }
        
        // Miscellaneous
        Instruction::NOP => {
            // No operation
        }
        
        Instruction::XBA => {
            // Exchange B and A (swap high and low bytes of accumulator)
            let a = cpu.a;
            cpu.a = ((a & 0xFF) << 8) | ((a >> 8) & 0xFF);
            cpu.update_nz_flags(cpu.a & 0xFF); // Update based on new low byte
        }
        
        Instruction::XCE => {
            // Exchange carry and emulation flags
            let old_carry = cpu.carry();
            cpu.set_carry(cpu.emulation_mode);
            
            if old_carry {
                cpu.enter_emulation_mode();
            } else {
                cpu.enter_native_mode();
            }
        }
        
        Instruction::BRK => {
            // Software interrupt
            cpu.increment_pc(1); // BRK is a 2-byte instruction
            
            // Push PC and P
            if !cpu.emulation_mode {
                cpu.push_8(bus, cpu.get_pc_bank());
            }
            cpu.push_16(bus, cpu.get_pc_offset());
            cpu.push_8(bus, cpu.p | 0x10); // Set B flag in pushed status
            
            // Disable interrupts
            cpu.set_irq_disable(true);
            
            // Jump to interrupt vector
            let vector = if cpu.emulation_mode {
                bus.read16(0xFFFE) // Emulation mode BRK/IRQ vector
            } else {
                bus.read16(0xFFE6) // Native mode BRK vector
            };
            cpu.pc = vector as u32;
        }
        
        Instruction::COP => {
            // Coprocessor interrupt
            cpu.increment_pc(1); // COP is a 2-byte instruction
            
            // Push PC and P
            if !cpu.emulation_mode {
                cpu.push_8(bus, cpu.get_pc_bank());
            }
            cpu.push_16(bus, cpu.get_pc_offset());
            cpu.push_8(bus, cpu.p);
            
            // Disable interrupts
            cpu.set_irq_disable(true);
            
            // Jump to interrupt vector
            let vector = if cpu.emulation_mode {
                bus.read16(0xFFF4) // Emulation mode COP vector
            } else {
                bus.read16(0xFFE4) // Native mode COP vector
            };
            cpu.pc = vector as u32;
        }
        
        Instruction::STP => {
            // Stop the clock - halt CPU until reset
            cpu.halt = true;
        }
        
        Instruction::WAI => {
            // Wait for interrupt
            cpu.waiting_for_interrupt = true;
        }
        
        // Transfer Instructions (remaining)
        Instruction::TXY => {
            let value = cpu.get_x();
            cpu.set_y(value);
            cpu.update_nz_flags_index(value);
        }
        
        Instruction::TYX => {
            let value = cpu.get_y();
            cpu.set_x(value);
            cpu.update_nz_flags_index(value);
        }
        
        Instruction::TCD => {
            // Transfer C (16-bit accumulator) to Direct Page
            cpu.d = cpu.a;
            cpu.update_nz_flags(cpu.d);
        }
        
        Instruction::TDC => {
            // Transfer Direct Page to C (16-bit accumulator)
            cpu.a = cpu.d;
            cpu.update_nz_flags(cpu.a);
        }
        
        Instruction::TCS => {
            // Transfer C (16-bit accumulator) to Stack
            cpu.s = cpu.a;
            // TCS doesn't affect flags
        }
        
        Instruction::TSC => {
            // Transfer Stack to C (16-bit accumulator)
            cpu.a = cpu.s;
            cpu.update_nz_flags(cpu.a);
        }
        
        // Stack Instructions (remaining)
        Instruction::PHB => {
            // Push Data Bank Register
            cpu.push_8(bus, cpu.db);
        }
        
        Instruction::PLB => {
            // Pull Data Bank Register
            cpu.db = cpu.pop_8(bus);
            cpu.update_nz_flags(cpu.db as u16);
        }
        
        Instruction::PHD => {
            // Push Direct Page Register
            cpu.push_16(bus, cpu.d);
        }
        
        Instruction::PLD => {
            // Pull Direct Page Register
            cpu.d = cpu.pop_16(bus);
            cpu.update_nz_flags(cpu.d);
        }
        
        Instruction::PHK => {
            // Push Program Bank Register
            cpu.push_8(bus, cpu.get_pc_bank());
        }
        
        // Special Push Instructions
        Instruction::PEA => {
            // Push Effective Absolute Address
            // PEA pushes the immediate 16-bit value onto the stack
            let value = addressing_result.value;
            cpu.push_16(bus, value);
        }
        
        Instruction::PEI => {
            // Push Effective Indirect Address
            // PEI pushes the 16-bit value at the direct page location
            let value = addressing_result.value;
            cpu.push_16(bus, value);
        }
        
        Instruction::PER => {
            // Push Effective Relative Address
            // PER pushes PC + relative offset
            let target = addressing_result.address;
            cpu.push_16(bus, target as u16);
        }
        
        // Block Move Instructions
        Instruction::MVN => {
            // Move Block Negative
            let src_bank = (addressing_result.value & 0xFF) as u8;
            let dst_bank = ((addressing_result.value >> 8) & 0xFF) as u8;
            
            // Move one byte
            let src_addr = ((src_bank as u32) << 16) | (cpu.get_x() as u32);
            let dst_addr = ((dst_bank as u32) << 16) | (cpu.get_y() as u32);
            let byte = bus.read8(src_addr);
            bus.write8(dst_addr, byte);
            
            // Decrement X and Y
            cpu.set_x(cpu.get_x().wrapping_sub(1));
            cpu.set_y(cpu.get_y().wrapping_sub(1));
            
            // Decrement A (count)
            cpu.set_a(cpu.get_a().wrapping_sub(1));
            
            // If A != 0xFFFF, repeat (by decrementing PC)
            if cpu.a != 0xFFFF {
                cpu.pc = cpu.pc.wrapping_sub(3); // Repeat instruction
            }
            
            cycles = 7; // Base cycles per byte moved
        }
        
        Instruction::MVP => {
            // Move Block Positive
            let src_bank = (addressing_result.value & 0xFF) as u8;
            let dst_bank = ((addressing_result.value >> 8) & 0xFF) as u8;
            
            // Move one byte
            let src_addr = ((src_bank as u32) << 16) | (cpu.get_x() as u32);
            let dst_addr = ((dst_bank as u32) << 16) | (cpu.get_y() as u32);
            let byte = bus.read8(src_addr);
            bus.write8(dst_addr, byte);
            
            // Increment X and Y
            cpu.set_x(cpu.get_x().wrapping_add(1));
            cpu.set_y(cpu.get_y().wrapping_add(1));
            
            // Decrement A (count)
            cpu.set_a(cpu.get_a().wrapping_sub(1));
            
            // If A != 0xFFFF, repeat (by decrementing PC)
            if cpu.a != 0xFFFF {
                cpu.pc = cpu.pc.wrapping_sub(3); // Repeat instruction
            }
            
            cycles = 7; // Base cycles per byte moved
        }
        
        // Status Register Instructions
        Instruction::REP => {
            // Reset Processor Status Bits
            let mask = addressing_result.value as u8;
            cpu.p &= !mask;
            
            // In emulation mode, M and X flags are always set
            if cpu.emulation_mode {
                cpu.set_memory_width(true);
                cpu.set_index_width(true);
            }
        }
        
        Instruction::SEP => {
            // Set Processor Status Bits
            let mask = addressing_result.value as u8;
            cpu.p |= mask;
        }
        
        Instruction::WDM => {
            // William D. Mensch Jr. - Reserved instruction
            // This is used for debugger breakpoints in some systems
            // For now, just skip the operand byte
            cpu.increment_pc(1);
        }
        
    }
    
    Ok(cycles)
}

fn branch_taken(cpu: &mut CpuRegisters, target: u32, cycles: &mut u32) {
    // Add 1 cycle for branch taken
    *cycles += 1;
    
    // Add 1 more cycle if crossing page boundary in emulation mode
    if cpu.emulation_mode && (cpu.pc & 0xFF00) != (target & 0xFF00) {
        *cycles += 1;
    }
    
    cpu.pc = target;
}