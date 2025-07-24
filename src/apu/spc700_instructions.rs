// SPC700 CPU Instructions

use super::spc700::Spc700;

// Flag bit positions in PSW
const FLAG_N: u8 = 0x80;  // Negative
const FLAG_V: u8 = 0x40;  // Overflow
const FLAG_P: u8 = 0x20;  // Direct page
const FLAG_B: u8 = 0x10;  // Break
const FLAG_H: u8 = 0x08;  // Half carry
const FLAG_I: u8 = 0x04;  // Interrupt disable
const FLAG_Z: u8 = 0x02;  // Zero
const FLAG_C: u8 = 0x01;  // Carry

impl Spc700 {
    pub fn execute_instruction(&mut self) {
        let opcode = self.fetch8();
        
        match opcode {
            // NOP
            0x00 => {
                self.cycles += 2;
            }
            
            // MOV A, #imm
            0xE8 => {
                let imm = self.fetch8();
                self.a = imm;
                self.set_nz(self.a);
                self.cycles += 2;
            }
            
            // MOV X, #imm
            0xCD => {
                let imm = self.fetch8();
                self.x = imm;
                self.set_nz(self.x);
                self.cycles += 2;
            }
            
            // MOV Y, #imm
            0x8D => {
                let imm = self.fetch8();
                self.y = imm;
                self.set_nz(self.y);
                self.cycles += 2;
            }
            
            // MOV A, X
            0x7D => {
                self.a = self.x;
                self.set_nz(self.a);
                self.cycles += 2;
            }
            
            // MOV A, Y
            0xDD => {
                self.a = self.y;
                self.set_nz(self.a);
                self.cycles += 2;
            }
            
            // MOV X, A
            0x5D => {
                self.x = self.a;
                self.set_nz(self.x);
                self.cycles += 2;
            }
            
            // MOV Y, A
            0xFD => {
                self.y = self.a;
                self.set_nz(self.y);
                self.cycles += 2;
            }
            
            // MOV dp, A
            0xC4 => {
                let dp = self.fetch8();
                let addr = self.get_dp_addr(dp);
                self.write8(addr, self.a);
                self.cycles += 4;
            }
            
            // MOV A, dp
            0xE4 => {
                let dp = self.fetch8();
                let addr = self.get_dp_addr(dp);
                self.a = self.read8(addr);
                self.set_nz(self.a);
                self.cycles += 3;
            }
            
            // MOV !abs, A
            0xC5 => {
                let addr = self.fetch16();
                self.write8(addr, self.a);
                self.cycles += 5;
            }
            
            // MOV A, !abs
            0xE5 => {
                let addr = self.fetch16();
                self.a = self.read8(addr);
                self.set_nz(self.a);
                self.cycles += 4;
            }
            
            // INC A
            0xBC => {
                self.a = self.a.wrapping_add(1);
                self.set_nz(self.a);
                self.cycles += 2;
            }
            
            // INC X
            0x3D => {
                self.x = self.x.wrapping_add(1);
                self.set_nz(self.x);
                self.cycles += 2;
            }
            
            // INC Y
            0xFC => {
                self.y = self.y.wrapping_add(1);
                self.set_nz(self.y);
                self.cycles += 2;
            }
            
            // DEC A
            0x9C => {
                self.a = self.a.wrapping_sub(1);
                self.set_nz(self.a);
                self.cycles += 2;
            }
            
            // DEC X
            0x1D => {
                self.x = self.x.wrapping_sub(1);
                self.set_nz(self.x);
                self.cycles += 2;
            }
            
            // DEC Y
            0xDC => {
                self.y = self.y.wrapping_sub(1);
                self.set_nz(self.y);
                self.cycles += 2;
            }
            
            // ADC A, #imm
            0x88 => {
                let imm = self.fetch8();
                self.adc(imm);
                self.cycles += 2;
            }
            
            // ADC A, dp
            0x84 => {
                let dp = self.fetch8();
                let addr = self.get_dp_addr(dp);
                let value = self.read8(addr);
                self.adc(value);
                self.cycles += 3;
            }
            
            // SBC A, #imm
            0xA8 => {
                let imm = self.fetch8();
                self.sbc(imm);
                self.cycles += 2;
            }
            
            // SBC A, dp
            0xA4 => {
                let dp = self.fetch8();
                let addr = self.get_dp_addr(dp);
                let value = self.read8(addr);
                self.sbc(value);
                self.cycles += 3;
            }
            
            // CMP A, #imm
            0x68 => {
                let imm = self.fetch8();
                self.cmp(self.a, imm);
                self.cycles += 2;
            }
            
            // CMP X, #imm
            0xC8 => {
                let imm = self.fetch8();
                self.cmp(self.x, imm);
                self.cycles += 2;
            }
            
            // CMP Y, #imm
            0xAD => {
                let imm = self.fetch8();
                self.cmp(self.y, imm);
                self.cycles += 2;
            }
            
            // AND A, #imm
            0x28 => {
                let imm = self.fetch8();
                self.a &= imm;
                self.set_nz(self.a);
                self.cycles += 2;
            }
            
            // OR A, #imm
            0x08 => {
                let imm = self.fetch8();
                self.a |= imm;
                self.set_nz(self.a);
                self.cycles += 2;
            }
            
            // EOR A, #imm
            0x48 => {
                let imm = self.fetch8();
                self.a ^= imm;
                self.set_nz(self.a);
                self.cycles += 2;
            }
            
            // ASL A
            0x1C => {
                let carry = (self.a & 0x80) != 0;
                self.a = self.a << 1;
                self.set_flag(FLAG_C, carry);
                self.set_nz(self.a);
                self.cycles += 2;
            }
            
            // LSR A
            0x5C => {
                let carry = (self.a & 0x01) != 0;
                self.a = self.a >> 1;
                self.set_flag(FLAG_C, carry);
                self.set_nz(self.a);
                self.cycles += 2;
            }
            
            // ROL A
            0x3C => {
                let carry = (self.a & 0x80) != 0;
                self.a = (self.a << 1) | if self.get_flag(FLAG_C) { 1 } else { 0 };
                self.set_flag(FLAG_C, carry);
                self.set_nz(self.a);
                self.cycles += 2;
            }
            
            // ROR A
            0x7C => {
                let carry = (self.a & 0x01) != 0;
                self.a = (self.a >> 1) | if self.get_flag(FLAG_C) { 0x80 } else { 0 };
                self.set_flag(FLAG_C, carry);
                self.set_nz(self.a);
                self.cycles += 2;
            }
            
            // PUSH A
            0x2D => {
                self.push8(self.a);
                self.cycles += 4;
            }
            
            // POP A
            0xAE => {
                self.a = self.pop8();
                self.cycles += 4;
            }
            
            // PUSH X
            0x4D => {
                self.push8(self.x);
                self.cycles += 4;
            }
            
            // POP X
            0xCE => {
                self.x = self.pop8();
                self.cycles += 4;
            }
            
            // PUSH Y
            0x6D => {
                self.push8(self.y);
                self.cycles += 4;
            }
            
            // POP Y
            0xEE => {
                self.y = self.pop8();
                self.cycles += 4;
            }
            
            // MOV SP, X
            0xBD => {
                self.sp = self.x;
                self.cycles += 2;
            }
            
            // MOV (X), A
            0xC6 => {
                let addr = self.get_dp_addr(self.x);
                self.write8(addr, self.a);
                self.cycles += 4;
            }
            
            // JMP abs
            0x5F => {
                let addr = self.fetch16();
                self.pc = addr;
                self.cycles += 3;
            }
            
            // BRA rel
            0x2F => {
                let offset = self.fetch8() as i8;
                self.pc = (self.pc as i32 + offset as i32) as u16;
                self.cycles += 4;
            }
            
            // BEQ rel
            0xF0 => {
                let offset = self.fetch8() as i8;
                if self.get_flag(FLAG_Z) {
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                    self.cycles += 4;
                } else {
                    self.cycles += 2;
                }
            }
            
            // BNE rel
            0xD0 => {
                let offset = self.fetch8() as i8;
                if !self.get_flag(FLAG_Z) {
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                    self.cycles += 4;
                } else {
                    self.cycles += 2;
                }
            }
            
            // BCC rel
            0x90 => {
                let offset = self.fetch8() as i8;
                if !self.get_flag(FLAG_C) {
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                    self.cycles += 4;
                } else {
                    self.cycles += 2;
                }
            }
            
            // BCS rel
            0xB0 => {
                let offset = self.fetch8() as i8;
                if self.get_flag(FLAG_C) {
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                    self.cycles += 4;
                } else {
                    self.cycles += 2;
                }
            }
            
            // JSR abs
            0x3F => {
                let addr = self.fetch16();
                let return_addr = self.pc;
                self.push16(return_addr);
                self.pc = addr;
                self.cycles += 8;
            }
            
            // RTS
            0x6F => {
                self.pc = self.pop16();
                self.cycles += 5;
            }
            
            // CLRC
            0x60 => {
                self.set_flag(FLAG_C, false);
                self.cycles += 2;
            }
            
            // SETC
            0x80 => {
                self.set_flag(FLAG_C, true);
                self.cycles += 2;
            }
            
            // CLRP
            0x20 => {
                self.set_flag(FLAG_P, false);
                self.cycles += 2;
            }
            
            // SETP
            0x40 => {
                self.set_flag(FLAG_P, true);
                self.cycles += 2;
            }
            
            // EI
            0xA0 => {
                self.set_flag(FLAG_I, false);
                self.cycles += 3;
            }
            
            // DI
            0xC0 => {
                self.set_flag(FLAG_I, true);
                self.cycles += 3;
            }
            
            _ => {
                // Unknown opcode
                println!("SPC700: Unknown opcode 0x{:02X} at PC=0x{:04X}", opcode, self.pc.wrapping_sub(1));
                self.cycles += 2;
            }
        }
    }
    
    // Helper functions
    fn fetch8(&mut self) -> u8 {
        let value = self.read8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }
    
    fn fetch16(&mut self) -> u16 {
        let low = self.fetch8() as u16;
        let high = self.fetch8() as u16;
        (high << 8) | low
    }
    
    fn get_dp_addr(&self, dp: u8) -> u16 {
        if self.get_flag(FLAG_P) {
            0x0100 | dp as u16
        } else {
            dp as u16
        }
    }
    
    fn push8(&mut self, value: u8) {
        self.write8(0x0100 | self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }
    
    fn pop8(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read8(0x0100 | self.sp as u16)
    }
    
    fn push16(&mut self, value: u16) {
        self.push8((value >> 8) as u8);
        self.push8((value & 0xFF) as u8);
    }
    
    fn pop16(&mut self) -> u16 {
        let low = self.pop8() as u16;
        let high = self.pop8() as u16;
        (high << 8) | low
    }
    
    fn get_flag(&self, flag: u8) -> bool {
        (self.psw & flag) != 0
    }
    
    fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.psw |= flag;
        } else {
            self.psw &= !flag;
        }
    }
    
    fn set_nz(&mut self, value: u8) {
        self.set_flag(FLAG_N, (value & 0x80) != 0);
        self.set_flag(FLAG_Z, value == 0);
    }
    
    fn adc(&mut self, value: u8) {
        let carry = if self.get_flag(FLAG_C) { 1u16 } else { 0u16 };
        let result = self.a as u16 + value as u16 + carry;
        let half_carry = ((self.a & 0x0F) as u16 + (value & 0x0F) as u16 + carry) > 0x0F;
        let overflow = ((self.a ^ value ^ 0x80) & (self.a ^ result as u8) & 0x80) != 0;
        
        self.a = result as u8;
        self.set_flag(FLAG_C, result > 0xFF);
        self.set_flag(FLAG_H, half_carry);
        self.set_flag(FLAG_V, overflow);
        self.set_nz(self.a);
    }
    
    fn sbc(&mut self, value: u8) {
        let carry = if self.get_flag(FLAG_C) { 0 } else { 1 };
        let result = self.a as i16 - value as i16 - carry as i16;
        let half_carry = (self.a & 0x0F) < (value & 0x0F) + carry;
        let overflow = ((self.a ^ value) & (self.a ^ result as u8) & 0x80) != 0;
        
        self.a = result as u8;
        self.set_flag(FLAG_C, result >= 0);
        self.set_flag(FLAG_H, !half_carry);
        self.set_flag(FLAG_V, overflow);
        self.set_nz(self.a);
    }
    
    fn cmp(&mut self, reg: u8, value: u8) {
        let result = reg as i16 - value as i16;
        self.set_flag(FLAG_C, result >= 0);
        self.set_flag(FLAG_N, (result & 0x80) != 0);
        self.set_flag(FLAG_Z, result == 0);
    }
}