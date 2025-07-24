use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MapperType {
    LoROM,
    HiROM,
    ExLoROM,
    ExHiROM,
    SA1,
    SuperFX,
    SDD1,
    Unknown,
}

impl MapperType {
    pub fn from_header_byte(byte: u8) -> Self {
        match byte & 0x0F {
            0x00 => MapperType::LoROM,
            0x01 => MapperType::HiROM,
            0x02 => MapperType::SA1,
            0x03 => MapperType::SuperFX,
            0x04 => MapperType::SDD1,
            0x05 => MapperType::ExLoROM,
            0x0A => MapperType::ExHiROM,
            _ => MapperType::Unknown,
        }
    }
}

pub trait Mapper {
    fn map_address(&self, address: u32) -> Option<usize>;
    fn map_sram_address(&self, address: u32) -> Option<usize>;
}

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
            // Banks $00-$7F: LoROM mapping
            0x00..=0x7F => {
                match addr {
                    // ROM area $8000-$FFFF
                    0x8000..=0xFFFF => {
                        let rom_addr = ((bank as usize) << 15) | ((addr - 0x8000) as usize);
                        if rom_addr < self.rom_size {
                            Some(rom_addr)
                        } else {
                            None
                        }
                    }
                    // SRAM area $6000-$7FFF (banks $70-$7F only)
                    0x6000..=0x7FFF if bank >= 0x70 => {
                        self.map_sram_address(address)
                    }
                    _ => None,
                }
            }
            // Banks $80-$FF: Mirror of $00-$7F
            0x80..=0xFF => {
                self.map_address(address & 0x7FFFFF)
            }
            _ => None,
        }
    }

    fn map_sram_address(&self, address: u32) -> Option<usize> {
        let bank = (address >> 16) & 0xFF;
        let addr = address & 0xFFFF;

        if bank >= 0x70 && bank <= 0x7F && addr >= 0x6000 && addr <= 0x7FFF {
            let sram_addr = (((bank - 0x70) as usize) << 13) | ((addr - 0x6000) as usize);
            if sram_addr < self.sram_size {
                Some(sram_addr)
            } else {
                None
            }
        } else {
            None
        }
    }
}

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
                match addr {
                    // ROM area $8000-$FFFF
                    0x8000..=0xFFFF => {
                        let rom_addr = ((bank as usize) << 16) | (addr as usize);
                        if rom_addr < self.rom_size {
                            Some(rom_addr)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            // Banks $40-$7F: ROM area
            0x40..=0x7F => {
                let rom_addr = ((bank as usize) << 16) | (addr as usize);
                if rom_addr < self.rom_size {
                    Some(rom_addr)
                } else {
                    None
                }
            }
            // Banks $80-$BF: Mirror of $00-$3F
            0x80..=0xBF => {
                self.map_address((address & 0x3FFFFF) | 0x000000)
            }
            // Banks $C0-$FF: ROM area
            0xC0..=0xFF => {
                let rom_addr = ((bank as usize) << 16) | (addr as usize);
                if rom_addr < self.rom_size {
                    Some(rom_addr)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn map_sram_address(&self, address: u32) -> Option<usize> {
        let bank = (address >> 16) & 0xFF;
        let addr = address & 0xFFFF;

        // SRAM in HiROM is typically at $20-$3F:$6000-$7FFF and $A0-$BF:$6000-$7FFF
        if ((bank >= 0x20 && bank <= 0x3F) || (bank >= 0xA0 && bank <= 0xBF)) 
            && addr >= 0x6000 && addr <= 0x7FFF {
            let sram_bank = if bank >= 0xA0 { bank - 0xA0 } else { bank - 0x20 };
            let sram_addr = ((sram_bank as usize) << 13) | ((addr - 0x6000) as usize);
            if sram_addr < self.sram_size {
                Some(sram_addr)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub fn create_mapper(mapper_type: MapperType, rom_size: usize, sram_size: usize) -> Result<Box<dyn Mapper>> {
    match mapper_type {
        MapperType::LoROM | MapperType::ExLoROM => {
            Ok(Box::new(LoROMMapper::new(rom_size, sram_size)))
        }
        MapperType::HiROM | MapperType::ExHiROM => {
            Ok(Box::new(HiROMMapper::new(rom_size, sram_size)))
        }
        _ => {
            log::warn!("Unsupported mapper type: {:?}, falling back to LoROM", mapper_type);
            Ok(Box::new(LoROMMapper::new(rom_size, sram_size)))
        }
    }
}