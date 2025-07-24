use crate::cartridge::CartridgeHeader;
use crate::memory::mappers::{create_mapper, Mapper};
use crate::Result;
use anyhow::anyhow;
use log::info;

pub struct Cartridge {
    pub header: CartridgeHeader,
    pub rom_data: Vec<u8>,
    pub sram: Vec<u8>,
    pub mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn load(rom_data: &[u8]) -> Result<Self> {
        // Remove copier header if present
        let clean_rom_data = Self::remove_copier_header(rom_data);
        
        // Parse header
        let header = CartridgeHeader::parse(&clean_rom_data)?;
        
        info!("Loaded cartridge:");
        info!("{}", header);
        
        // Validate ROM size
        if clean_rom_data.len() > header.rom_size * 2 {
            return Err(anyhow!("ROM file size is larger than expected"));
        }
        
        // Create mapper
        let mapper = create_mapper(
            header.mapper_type,
            clean_rom_data.len(),
            header.sram_size,
        )?;
        
        // Initialize SRAM
        let sram = vec![0; header.sram_size];
        
        Ok(Cartridge {
            header,
            rom_data: clean_rom_data,
            sram,
            mapper,
        })
    }

    pub fn read(&self, address: u32) -> u8 {
        // Try to map ROM address
        if let Some(rom_offset) = self.mapper.map_address(address) {
            if rom_offset < self.rom_data.len() {
                return self.rom_data[rom_offset];
            }
        }
        
        // Try to map SRAM address
        if let Some(sram_offset) = self.mapper.map_sram_address(address) {
            if sram_offset < self.sram.len() {
                return self.sram[sram_offset];
            }
        }
        
        // Return open bus value (typically 0x00 or last value on bus)
        0x00
    }

    pub fn write(&mut self, address: u32, value: u8) {
        // Only SRAM is writable
        if let Some(sram_offset) = self.mapper.map_sram_address(address) {
            if sram_offset < self.sram.len() {
                self.sram[sram_offset] = value;
            }
        }
        // ROM writes are ignored
    }

    pub fn load_sram(&mut self, sram_data: &[u8]) -> Result<()> {
        if sram_data.len() != self.sram.len() {
            return Err(anyhow!(
                "SRAM data size mismatch: expected {}, got {}",
                self.sram.len(),
                sram_data.len()
            ));
        }
        
        self.sram.copy_from_slice(sram_data);
        info!("Loaded SRAM data ({} bytes)", sram_data.len());
        Ok(())
    }

    pub fn save_sram(&self) -> Vec<u8> {
        self.sram.clone()
    }

    pub fn has_sram(&self) -> bool {
        self.header.sram_size > 0
    }

    pub fn has_battery_backup(&self) -> bool {
        // Most games with SRAM have battery backup
        // This could be more sophisticated based on header analysis
        self.has_sram()
    }

    fn remove_copier_header(rom_data: &[u8]) -> Vec<u8> {
        // Check if ROM has a 512-byte copier header
        if (rom_data.len() % 1024) == 512 {
            info!("Removing 512-byte copier header");
            rom_data[512..].to_vec()
        } else {
            rom_data.to_vec()
        }
    }

    pub fn get_title(&self) -> &str {
        &self.header.title
    }

    pub fn get_rom_size(&self) -> usize {
        self.rom_data.len()
    }

    pub fn get_sram_size(&self) -> usize {
        self.sram.len()
    }

    pub fn get_mapper_type(&self) -> crate::memory::mappers::MapperType {
        self.header.mapper_type
    }

    pub fn get_region(&self) -> crate::cartridge::header::Region {
        self.header.region
    }

    pub fn has_coprocessor(&self) -> bool {
        !matches!(self.header.coprocessor, crate::cartridge::header::CoprocessorType::None)
    }

    pub fn get_coprocessor(&self) -> crate::cartridge::header::CoprocessorType {
        self.header.coprocessor
    }
    
    // Save state functionality
    pub fn get_sram(&self) -> Option<&[u8]> {
        if self.sram.is_empty() {
            None
        } else {
            Some(&self.sram)
        }
    }
    
}