use super::Mapper;

/// LoROM memory mapper (Mode 20)
/// Maps 32KB ROM banks to the upper half of each 64KB bank
pub struct LoROMMapper {
    rom_size: usize,
    sram_size: usize,
}

impl LoROMMapper {
    pub fn new(rom_size: usize, sram_size: usize) -> Self {
        Self { rom_size, sram_size }
    }
}

impl Mapper for LoROMMapper {
    fn map_address(&self, address: u32) -> Option<usize> {
        let bank = (address >> 16) & 0xFF;
        let addr = address & 0xFFFF;
        
        match bank {
            // Banks $00-$7D: System area
            0x00..=0x7D => {
                if addr >= 0x8000 {
                    // ROM area ($8000-$FFFF)
                    let rom_offset = ((bank & 0x7F) << 15) | (addr & 0x7FFF);
                    if (rom_offset as usize) < self.rom_size {
                        Some(rom_offset as usize)
                    } else {
                        None
                    }
                } else {
                    None // System area
                }
            }
            
            // Banks $80-$FF: ROM mirror
            0x80..=0xFF => {
                let rom_offset = ((bank & 0x7F) << 15) | (addr & 0x7FFF);
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
            // Banks $70-$7D: SRAM area
            0x70..=0x7D => {
                if addr < 0x8000 {
                    // SRAM area ($0000-$7FFF)
                    let sram_offset = ((bank - 0x70) << 15) | addr;
                    if (sram_offset as usize) < self.sram_size {
                        Some(sram_offset as usize)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            
            // Banks $F0-$FF: SRAM mirror (some games use this)
            0xF0..=0xFF => {
                if addr < 0x8000 {
                    let sram_offset = ((bank - 0xF0) << 15) | addr;
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
        "LoROM"
    }
}