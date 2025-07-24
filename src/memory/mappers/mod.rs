pub mod lorom;
pub mod hirom;

use crate::Result;
use anyhow::anyhow;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MapperType {
    LoROM,
    HiROM,
    ExLoROM,
    ExHiROM,
    SuperFX,
    SA1,
    SDD1,
    Unknown,
}

impl MapperType {
    pub fn from_header_byte(byte: u8) -> Self {
        match byte {
            0x20 | 0x30 => MapperType::LoROM,      // LoROM/FastLoROM
            0x21 | 0x31 => MapperType::HiROM,      // HiROM/FastHiROM
            0x22 | 0x32 => MapperType::ExLoROM,    // ExLoROM/FastExLoROM
            0x23 => MapperType::SA1,               // SA-1
            0x25 | 0x35 => MapperType::ExHiROM,    // ExHiROM/FastExHiROM
            _ => {
                // Fallback: check lower nibble for some cases
                match byte & 0x0F {
                    0x00 => MapperType::LoROM,
                    0x01 => MapperType::HiROM,
                    _ => MapperType::Unknown,
                }
            }
        }
    }
}

pub trait Mapper: Send + Sync {
    /// Map a CPU address to a ROM offset
    fn map_address(&self, address: u32) -> Option<usize>;
    
    /// Map a CPU address to an SRAM offset
    fn map_sram_address(&self, address: u32) -> Option<usize>;
    
    /// Get mapper name
    fn name(&self) -> &'static str;
}

pub fn create_mapper(mapper_type: MapperType, rom_size: usize, sram_size: usize) -> Result<Box<dyn Mapper>> {
    match mapper_type {
        MapperType::LoROM => Ok(Box::new(lorom::LoROMMapper::new(rom_size, sram_size))),
        MapperType::HiROM => Ok(Box::new(hirom::HiROMMapper::new(rom_size, sram_size))),
        _ => Err(anyhow!("Unsupported mapper type: {:?}", mapper_type)),
    }
}