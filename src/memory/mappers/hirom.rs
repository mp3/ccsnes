use super::Mapper;

/// HiROM memory mapper (Mode 21)
/// Maps 64KB ROM banks directly
pub struct HiROMMapper {
    rom_size: usize,
    sram_size: usize,
}

impl HiROMMapper {
    pub fn new(rom_size: usize, sram_size: usize) -> Self {
        Self { rom_size, sram_size }
    }
}

impl Mapper for HiROMMapper {
    fn map_address(&self, address: u32) -> Option<usize> {
        let bank = (address >> 16) & 0xFF;
        let addr = address & 0xFFFF;
        
        match bank {
            // Banks $00-$3F: System area
            0x00..=0x3F => {
                if addr >= 0x8000 {
                    // ROM area ($8000-$FFFF)
                    let rom_offset = (bank << 16) | addr;
                    if (rom_offset as usize) < self.rom_size {
                        Some(rom_offset as usize)
                    } else {
                        None
                    }
                } else {
                    None // System area
                }
            }
            
            // Banks $40-$7D: ROM area
            0x40..=0x7D => {
                let rom_offset = (bank << 16) | addr;
                if (rom_offset as usize) < self.rom_size {
                    Some(rom_offset as usize)
                } else {
                    None
                }
            }
            
            // Banks $80-$BF: ROM mirror
            0x80..=0xBF => {
                let rom_offset = ((bank - 0x80) << 16) | addr;
                if (rom_offset as usize) < self.rom_size {
                    Some(rom_offset as usize)
                } else {
                    None
                }
            }
            
            // Banks $C0-$FF: ROM area
            0xC0..=0xFF => {
                let rom_offset = ((bank - 0x80) << 16) | addr;
                if (rom_offset as usize) < self.rom_size {
                    Some(rom_offset as usize)
                } else {
                    None
                }
            }
            
            _ => None,
        }
    }
    
    fn map_sram_address(&self, address: u32) -> Option<usize> {
        if self.sram_size == 0 {
            return None;
        }
        
        let bank = (address >> 16) & 0xFF;
        let addr = address & 0xFFFF;
        
        match bank {
            // Banks $20-$3F, $A0-$BF: SRAM area
            0x20..=0x3F | 0xA0..=0xBF => {
                if addr >= 0x6000 && addr <= 0x7FFF {
                    // SRAM area ($6000-$7FFF)
                    let bank_offset = if bank >= 0xA0 { bank - 0xA0 } else { bank - 0x20 };
                    let sram_offset = (bank_offset << 13) | (addr - 0x6000);
                    if (sram_offset as usize) < self.sram_size {
                        Some(sram_offset as usize)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            
            _ => None,
        }
    }
    
    fn name(&self) -> &'static str {
        "HiROM"
    }
}