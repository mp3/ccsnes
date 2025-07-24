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
    let cycles = info.base_cycles as u32 + addressing_result.cycles;
    
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
        
        // Miscellaneous
        Instruction::NOP => {
            // No operation
        }
        
        // TODO: Implement remaining instructions
        _ => {
            log::warn!("Unimplemented instruction: {:?}", info.instruction);
        }
    }
    
    Ok(cycles)
}