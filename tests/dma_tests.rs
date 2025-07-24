use ccsnes::dma::DmaController;
use ccsnes::ppu::Ppu;
use ccsnes::memory::Bus;

#[test]
fn test_dma_single_byte_transfer() {
    let mut dma = DmaController::new();
    let mut bus = Bus::new();
    let mut ppu = Ppu::new();
    
    // Setup source data
    bus.write8(0x1000, 0xAA);
    bus.write8(0x1001, 0xBB);
    bus.write8(0x1002, 0xCC);
    
    // Configure DMA channel 0
    dma.write_register(0x4300, 0x00); // Single byte, A to B, increment
    dma.write_register(0x4301, 0x18); // B address = $2118 (VMDATAL)
    dma.write_register(0x4302, 0x00); // A address low
    dma.write_register(0x4303, 0x10); // A address high = $1000
    dma.write_register(0x4304, 0x00); // A bank = $00
    dma.write_register(0x4305, 0x03); // Transfer size low = 3
    dma.write_register(0x4306, 0x00); // Transfer size high
    
    // Set VRAM address in PPU
    ppu.write_register(0x2116, 0x00); // VMADDL
    ppu.write_register(0x2117, 0x00); // VMADDH
    
    // Enable DMA channel 0
    dma.write_register(0x420B, 0x01);
    
    // Execute DMA
    let cycles = dma.execute_dma(&mut bus, &mut ppu);
    
    assert!(cycles > 0);
    assert_eq!(dma.read_register(0x420B), 0x00); // DMA enable cleared
    
    // Verify data was transferred
    // Note: This would need proper PPU VRAM access implementation
}

#[test]
fn test_dma_two_registers_mode() {
    let mut dma = DmaController::new();
    let mut bus = Bus::new();
    let mut ppu = Ppu::new();
    
    // Setup source data
    bus.write8(0x2000, 0x11);
    bus.write8(0x2001, 0x22);
    bus.write8(0x2002, 0x33);
    bus.write8(0x2003, 0x44);
    
    // Configure DMA channel 1 for mode 1 (two registers)
    dma.write_register(0x4310, 0x01); // Two registers mode
    dma.write_register(0x4311, 0x18); // B address = $2118
    dma.write_register(0x4312, 0x00); // A address low
    dma.write_register(0x4313, 0x20); // A address high = $2000
    dma.write_register(0x4314, 0x00); // A bank
    dma.write_register(0x4315, 0x04); // Transfer 4 bytes
    dma.write_register(0x4316, 0x00);
    
    // Enable DMA channel 1
    dma.write_register(0x420B, 0x02);
    
    let cycles = dma.execute_dma(&mut bus, &mut ppu);
    
    assert!(cycles > 0);
}

#[test]
fn test_dma_fixed_address() {
    let mut dma = DmaController::new();
    let mut bus = Bus::new();
    let mut ppu = Ppu::new();
    
    // Setup a single value to transfer multiple times
    bus.write8(0x3000, 0xFF);
    
    // Configure DMA with fixed source address
    dma.write_register(0x4320, 0x08); // Single byte, fixed address
    dma.write_register(0x4321, 0x18); // B address
    dma.write_register(0x4322, 0x00); // A address low
    dma.write_register(0x4323, 0x30); // A address high = $3000
    dma.write_register(0x4324, 0x00); // A bank
    dma.write_register(0x4325, 0x10); // Transfer 16 bytes
    dma.write_register(0x4326, 0x00);
    
    // Enable DMA channel 2
    dma.write_register(0x420B, 0x04);
    
    let cycles = dma.execute_dma(&mut bus, &mut ppu);
    
    assert!(cycles > 0);
}

#[test]
fn test_hdma_init() {
    let mut dma = DmaController::new();
    let mut bus = Bus::new();
    
    // Setup HDMA table
    bus.write8(0x4000, 0x81); // 1 scanline, repeat mode
    bus.write8(0x4001, 0xAA); // Data byte
    bus.write8(0x4002, 0x02); // 2 scanlines, single mode
    bus.write8(0x4003, 0xBB); // Data byte 1
    bus.write8(0x4004, 0xCC); // Data byte 2
    bus.write8(0x4005, 0x00); // End of table
    
    // Configure HDMA channel 0
    dma.write_register(0x4300, 0x00); // Direct mode, single byte
    dma.write_register(0x4301, 0x00); // B address (dummy)
    dma.write_register(0x4302, 0x00); // Table address low
    dma.write_register(0x4303, 0x40); // Table address high = $4000
    dma.write_register(0x4304, 0x00); // Bank
    
    // Enable HDMA channel 0
    dma.write_register(0x420C, 0x01);
    
    // Initialize HDMA
    dma.init_hdma(&mut bus);
    
    // Verify HDMA is active
    assert_eq!(dma.read_register(0x420C), 0x01);
}

#[test]
fn test_multiple_dma_channels() {
    let mut dma = DmaController::new();
    let mut bus = Bus::new();
    let mut ppu = Ppu::new();
    
    // Setup data for two channels
    bus.write8(0x1000, 0x11);
    bus.write8(0x2000, 0x22);
    
    // Configure channel 0
    dma.write_register(0x4300, 0x00);
    dma.write_register(0x4301, 0x18);
    dma.write_register(0x4302, 0x00);
    dma.write_register(0x4303, 0x10);
    dma.write_register(0x4304, 0x00);
    dma.write_register(0x4305, 0x01);
    dma.write_register(0x4306, 0x00);
    
    // Configure channel 1
    dma.write_register(0x4310, 0x00);
    dma.write_register(0x4311, 0x19); // Different B address
    dma.write_register(0x4312, 0x00);
    dma.write_register(0x4313, 0x20);
    dma.write_register(0x4314, 0x00);
    dma.write_register(0x4315, 0x01);
    dma.write_register(0x4316, 0x00);
    
    // Enable both channels
    dma.write_register(0x420B, 0x03);
    
    let cycles = dma.execute_dma(&mut bus, &mut ppu);
    
    assert!(cycles > 16); // Should be more than single channel
    assert_eq!(dma.read_register(0x420B), 0x00);
}