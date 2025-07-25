use crate::memory::mappers::MapperType;
use crate::Result;
use anyhow::anyhow;
use std::fmt;

#[derive(Debug, Clone)]
pub struct CartridgeHeader {
    pub title: String,
    pub mapper_type: MapperType,
    pub rom_size: usize,
    pub sram_size: usize,
    pub region: Region,
    pub version: u8,
    pub checksum: u16,
    pub complement: u16,
    pub coprocessor: CoprocessorType,
}

#[derive(Debug, Clone)]
pub struct RomInfo {
    pub title: String,
    pub mapper_type: MapperType,
    pub rom_size: usize,
    pub sram_size: usize,
    pub region: Region,
    pub version: u8,
    pub coprocessor: CoprocessorType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Region {
    Japan,
    USA,
    Europe,
    Sweden,
    Finland,
    Denmark,
    France,
    Netherlands,
    Spain,
    Germany,
    Italy,
    China,
    Indonesia,
    Korea,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoprocessorType {
    None,
    DSP1,
    DSP2,
    DSP3,
    DSP4,
    CX4,
    SA1,
    SDD1,
    RTC,
    OBC1,
    SuperFX,
    SuperFX2,
    Unknown,
}

impl CartridgeHeader {
    pub fn parse(rom_data: &[u8]) -> Result<Self> {
        // Try to detect header location (LoROM vs HiROM)
        let header_offset = Self::detect_header_offset(rom_data)?;
        log::debug!("Detected header offset: 0x{:X}", header_offset);
        
        if rom_data.len() < header_offset + 0x30 {
            return Err(anyhow!("ROM too small to contain valid header"));
        }

        // Extract header data
        let header_data = &rom_data[header_offset..header_offset + 0x30];
        
        // Parse title (21 bytes at offset 0x00)
        let mut title_bytes = header_data[0x00..0x15].to_vec();
        title_bytes.retain(|&b| b != 0 && b >= 0x20); // Remove null bytes and control characters
        let title = String::from_utf8_lossy(&title_bytes).trim().to_string();

        // Parse mapper type (offset 0x15)
        let mapper_byte = header_data[0x15];
        let mapper_type = MapperType::from_header_byte(mapper_byte);

        // Parse coprocessor type (offset 0x16)
        let coprocessor_byte = header_data[0x16];
        let coprocessor = Self::parse_coprocessor(coprocessor_byte);

        // Parse ROM size (offset 0x17)
        let rom_size_byte = header_data[0x17];
        let rom_size = 1024 << rom_size_byte; // 2^rom_size_byte KB

        // Parse SRAM size (offset 0x18)
        let sram_size_byte = header_data[0x18];
        let sram_size = if sram_size_byte == 0 {
            0
        } else {
            1024 << sram_size_byte // 2^sram_size_byte KB
        };

        // Parse region (offset 0x19)
        let region_byte = header_data[0x19];
        let region = Self::parse_region(region_byte);

        // Parse version (offset 0x1B)
        let version = header_data[0x1B];

        // Parse checksum (offset 0x1C-0x1D, little endian)
        let complement = u16::from_le_bytes([header_data[0x1C], header_data[0x1D]]);
        
        // Parse checksum complement (offset 0x1E-0x1F, little endian)
        let checksum = u16::from_le_bytes([header_data[0x1E], header_data[0x1F]]);

        // Validate checksum
        if !Self::validate_checksum(rom_data, checksum, complement) {
            log::warn!("ROM checksum validation failed");
        }

        Ok(CartridgeHeader {
            title,
            mapper_type,
            rom_size,
            sram_size,
            region,
            version,
            checksum,
            complement,
            coprocessor,
        })
    }

    fn detect_header_offset(rom_data: &[u8]) -> Result<usize> {
        // Check if ROM has a 512-byte copier header
        let has_copier_header = (rom_data.len() % 1024) == 512;
        let base_offset = if has_copier_header { 512 } else { 0 };

        // Try both LoROM and HiROM locations and pick the best one
        let lorom_offset = base_offset + 0x7FC0;
        let hirom_offset = base_offset + 0xFFC0;
        
        let lorom_valid = rom_data.len() > lorom_offset + 0x30 && 
                          Self::is_valid_header(&rom_data[lorom_offset..lorom_offset + 0x30]);
        let hirom_valid = rom_data.len() > hirom_offset + 0x30 && 
                          Self::is_valid_header(&rom_data[hirom_offset..hirom_offset + 0x30]);
        
        // If both are valid, check the mapper byte to decide
        if lorom_valid && hirom_valid {
            let lorom_mapper = rom_data[lorom_offset + 0x15];
            let hirom_mapper = rom_data[hirom_offset + 0x15];
            
            // Prefer HiROM if its mapper byte indicates HiROM
            if (hirom_mapper & 0x01) == 0x01 || hirom_mapper == 0x21 || hirom_mapper == 0x31 {
                return Ok(hirom_offset);
            } else {
                return Ok(lorom_offset);
            }
        } else if hirom_valid {
            return Ok(hirom_offset);
        } else if lorom_valid {
            return Ok(lorom_offset);
        }

        // Try ExLoROM header location ($40FFB0)
        let exlorom_offset = base_offset + 0x40FFB0;
        if rom_data.len() > exlorom_offset + 0x30 {
            if Self::is_valid_header(&rom_data[exlorom_offset..exlorom_offset + 0x30]) {
                return Ok(exlorom_offset);
            }
        }

        // Default to LoROM if nothing else works
        if rom_data.len() > lorom_offset + 0x30 {
            Ok(lorom_offset)
        } else {
            Err(anyhow!("Could not detect valid header location"))
        }
    }

    fn is_valid_header(header_data: &[u8]) -> bool {
        // Check if the header looks valid by examining key fields
        if header_data.len() < 0x30 {
            return false;
        }

        // Check ROM size field (should be reasonable)
        let rom_size_byte = header_data[0x17];
        if rom_size_byte > 16 { // Max 64MB
            return false;
        }

        // Check SRAM size field (should be reasonable)
        let sram_size_byte = header_data[0x18];
        if sram_size_byte > 8 { // Max 256KB
            return false;
        }

        // Check title area (should contain mostly printable characters)
        let title_area = &header_data[0x00..0x15];
        let printable_count = title_area.iter()
            .filter(|&&b| b >= 0x20 && b <= 0x7E || b == 0)
            .count();
        
        printable_count >= 15 // At least 15 of 21 characters should be printable or null
    }

    fn parse_region(byte: u8) -> Region {
        match byte {
            0x00 => Region::Japan,
            0x01 => Region::USA,
            0x02 => Region::Europe,
            0x03 => Region::Sweden,
            0x04 => Region::Finland,
            0x05 => Region::Denmark,
            0x06 => Region::France,
            0x07 => Region::Netherlands,
            0x08 => Region::Spain,
            0x09 => Region::Germany,
            0x0A => Region::Italy,
            0x0B => Region::China,
            0x0C => Region::Indonesia,
            0x0D => Region::Korea,
            _ => Region::Unknown,
        }
    }

    fn parse_coprocessor(byte: u8) -> CoprocessorType {
        match byte {
            0x00 => CoprocessorType::None,
            0x01 => CoprocessorType::DSP1,
            0x02 => CoprocessorType::DSP2,
            0x03 => CoprocessorType::DSP3,
            0x04 => CoprocessorType::DSP4,
            0x05 => CoprocessorType::CX4,
            0x34 => CoprocessorType::SA1,
            0x35 => CoprocessorType::SuperFX,
            0x3A => CoprocessorType::SuperFX2,
            0x43 => CoprocessorType::SDD1,
            0x45 => CoprocessorType::RTC,
            0x25 => CoprocessorType::OBC1,
            _ => CoprocessorType::Unknown,
        }
    }

    fn validate_checksum(rom_data: &[u8], checksum: u16, complement: u16) -> bool {
        // Basic checksum validation
        if checksum != (!complement & 0xFFFF) {
            return false;
        }

        // Calculate actual ROM checksum
        let mut calculated_checksum = 0u32;
        for &byte in rom_data.iter() {
            calculated_checksum = calculated_checksum.wrapping_add(byte as u32);
        }

        // Handle different ROM sizes
        let rom_size = rom_data.len();
        let power_of_two_size = rom_size.next_power_of_two();
        
        if power_of_two_size > rom_size {
            // Pad to power of two
            let padding = power_of_two_size - rom_size;
            calculated_checksum = calculated_checksum.wrapping_add((padding as u32) * 0xFF);
        }

        (calculated_checksum & 0xFFFF) == checksum as u32
    }
}

impl fmt::Display for CartridgeHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Title: {}\n", self.title)?;
        write!(f, "Mapper: {:?}\n", self.mapper_type)?;
        write!(f, "ROM Size: {} KB\n", self.rom_size / 1024)?;
        write!(f, "SRAM Size: {} KB\n", self.sram_size / 1024)?;
        write!(f, "Region: {:?}\n", self.region)?;
        write!(f, "Version: {}\n", self.version)?;
        write!(f, "Coprocessor: {:?}", self.coprocessor)
    }
}