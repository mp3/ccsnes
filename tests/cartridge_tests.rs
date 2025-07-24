use ccsnes::cartridge::{Cartridge, CartridgeHeader};
use ccsnes::memory::mappers::MapperType;

#[test]
fn test_lorom_header_detection() {
    // Create a minimal LoROM with header at $7FC0
    let mut rom = vec![0; 0x8000];
    
    // Add header at LoROM location ($7FC0)
    let header_offset = 0x7FC0;
    
    // Title (21 bytes)
    for i in 0..21 {
        rom[header_offset + i] = b'A' + (i as u8 % 26);
    }
    
    // Mapper type (LoROM)
    rom[header_offset + 0x15] = 0x20;
    
    // ROM size (256KB = 2^8 KB)
    rom[header_offset + 0x17] = 8;
    
    // SRAM size (8KB = 2^3 KB)
    rom[header_offset + 0x18] = 3;
    
    // Region (USA)
    rom[header_offset + 0x19] = 0x01;
    
    // Version
    rom[header_offset + 0x1B] = 0x00;
    
    // Checksum complement and checksum
    rom[header_offset + 0x1C] = 0xFF;
    rom[header_offset + 0x1D] = 0xFF;
    rom[header_offset + 0x1E] = 0x00;
    rom[header_offset + 0x1F] = 0x00;
    
    let header = CartridgeHeader::parse(&rom).unwrap();
    assert_eq!(header.mapper_type, MapperType::LoROM);
    assert_eq!(header.rom_size, 256 * 1024);
    assert_eq!(header.sram_size, 8 * 1024);
}

#[test]
fn test_hirom_header_detection() {
    // Create a minimal HiROM with header at $FFC0
    // HiROM needs at least 64KB to avoid being detected as LoROM
    let mut rom = vec![0; 0x10000];
    
    // Add header at HiROM location ($FFC0)
    let header_offset = 0xFFC0;
    
    // Title
    rom[header_offset..header_offset + 21].copy_from_slice(b"TEST ROM            \0");
    
    // Mapper type (HiROM)
    rom[header_offset + 0x15] = 0x21;
    
    // ROM size (512KB = 2^9 KB)
    rom[header_offset + 0x17] = 9;
    
    // SRAM size (0KB)
    rom[header_offset + 0x18] = 0;
    
    // Region (Japan)
    rom[header_offset + 0x19] = 0x00;
    
    // Version
    rom[header_offset + 0x1B] = 0x01;
    
    // Checksum complement and checksum
    rom[header_offset + 0x1C] = 0xFF;
    rom[header_offset + 0x1D] = 0xFF;
    rom[header_offset + 0x1E] = 0x00;
    rom[header_offset + 0x1F] = 0x00;
    
    let header = CartridgeHeader::parse(&rom).unwrap();
    eprintln!("HiROM test - mapper byte: 0x{:02X}, detected mapper: {:?}", rom[header_offset + 0x15], header.mapper_type);
    assert_eq!(header.mapper_type, MapperType::HiROM);
    assert_eq!(header.rom_size, 512 * 1024);
    assert_eq!(header.sram_size, 0);
}

#[test]
fn test_copier_header_removal() {
    // Create ROM with 512-byte copier header
    let mut rom = vec![0; 0x8200]; // 32KB + 512 bytes
    
    // Copier header (512 bytes of garbage)
    for i in 0..512 {
        rom[i] = (i & 0xFF) as u8;
    }
    
    // Real header at $7FC0 + 512
    let header_offset = 512 + 0x7FC0;
    
    // Title
    rom[header_offset..header_offset + 21].copy_from_slice(b"COPIER TEST         \0");
    
    // Mapper type
    rom[header_offset + 0x15] = 0x20;
    
    // ROM size
    rom[header_offset + 0x17] = 8;
    
    // SRAM size
    rom[header_offset + 0x18] = 0;
    
    // Region
    rom[header_offset + 0x19] = 0x02;
    
    // Checksum
    rom[header_offset + 0x1C] = 0xFF;
    rom[header_offset + 0x1D] = 0xFF;
    rom[header_offset + 0x1E] = 0x00;
    rom[header_offset + 0x1F] = 0x00;
    
    let cartridge = Cartridge::load(&rom).unwrap();
    assert_eq!(cartridge.get_rom_size(), 0x8000); // Should be 32KB without header
    assert_eq!(cartridge.get_title().trim(), "COPIER TEST");
}

#[test]
fn test_cartridge_memory_mapping() {
    // Create a small LoROM
    let mut rom = vec![0; 0x8000];
    
    // Add pattern to ROM data
    for (i, byte) in rom.iter_mut().enumerate() {
        *byte = (i & 0xFF) as u8;
    }
    
    // Add header
    let header_offset = 0x7FC0;
    rom[header_offset..header_offset + 21].copy_from_slice(b"MAPPING TEST        \0");
    rom[header_offset + 0x15] = 0x20; // LoROM
    rom[header_offset + 0x17] = 8; // ROM size
    rom[header_offset + 0x18] = 0; // No SRAM
    rom[header_offset + 0x19] = 0x01;
    
    // Checksum
    rom[header_offset + 0x1C] = 0xFF;
    rom[header_offset + 0x1D] = 0xFF;
    rom[header_offset + 0x1E] = 0x00;
    rom[header_offset + 0x1F] = 0x00;
    
    let cartridge = Cartridge::load(&rom).unwrap();
    
    // Test LoROM mapping
    // Bank $00, address $8000 should map to ROM offset $0000
    assert_eq!(cartridge.read(0x008000), 0x00);
    assert_eq!(cartridge.read(0x008001), 0x01);
    
    // Bank $00, address $FFFF should map to ROM offset $7FFF
    assert_eq!(cartridge.read(0x00FFFF), 0xFF);
    
    // Bank $80 should mirror bank $00
    assert_eq!(cartridge.read(0x808000), 0x00);
}

#[test]
fn test_sram_access() {
    // Create a LoROM with SRAM
    let mut rom = vec![0; 0x8000];
    
    // Add header
    let header_offset = 0x7FC0;
    rom[header_offset..header_offset + 21].copy_from_slice(b"SRAM TEST           \0");
    rom[header_offset + 0x15] = 0x20; // LoROM
    rom[header_offset + 0x17] = 8; // ROM size
    rom[header_offset + 0x18] = 3; // 8KB SRAM
    rom[header_offset + 0x19] = 0x01;
    
    // Checksum
    rom[header_offset + 0x1C] = 0xFF;
    rom[header_offset + 0x1D] = 0xFF;
    rom[header_offset + 0x1E] = 0x00;
    rom[header_offset + 0x1F] = 0x00;
    
    let mut cartridge = Cartridge::load(&rom).unwrap();
    
    // Test SRAM write and read
    // LoROM SRAM is at banks $70-$7D, addresses $0000-$7FFF
    cartridge.write(0x700000, 0x42);
    cartridge.write(0x700001, 0x43);
    
    assert_eq!(cartridge.read(0x700000), 0x42);
    assert_eq!(cartridge.read(0x700001), 0x43);
    
    // Test SRAM persistence
    let sram_data = cartridge.save_sram();
    assert_eq!(sram_data[0], 0x42);
    assert_eq!(sram_data[1], 0x43);
}