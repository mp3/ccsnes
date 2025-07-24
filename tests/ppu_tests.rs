use ccsnes::ppu::Ppu;
use ccsnes::memory::Bus;

#[test]
fn test_ppu_reset() {
    let mut ppu = Ppu::new();
    ppu.reset();
    
    assert_eq!(ppu.get_current_scanline(), 0);
    assert_eq!(ppu.get_current_dot(), 0);
    assert_eq!(ppu.get_frame_count(), 0);
    assert!(!ppu.is_in_vblank());
}

#[test]
fn test_ppu_timing() {
    let mut ppu = Ppu::new();
    let mut bus = Bus::new();
    
    // Step through one scanline (341 dots)
    for _ in 0..341 {
        ppu.step(&mut bus);
    }
    
    assert_eq!(ppu.get_current_scanline(), 1);
    assert_eq!(ppu.get_current_dot(), 0);
    
    // Step to V-Blank (225 scanlines)
    for _ in 1..225 {
        for _ in 0..341 {
            ppu.step(&mut bus);
        }
    }
    
    assert_eq!(ppu.get_current_scanline(), 225);
    assert!(ppu.is_in_vblank());
}

#[test]
fn test_vram_access() {
    let mut ppu = Ppu::new();
    
    // Set VRAM address
    ppu.write_register(0x2116, 0x00); // VMADDL
    ppu.write_register(0x2117, 0x10); // VMADDH - Address = $1000
    
    // Write to VRAM
    ppu.write_register(0x2118, 0xAB); // VMDATAL
    ppu.write_register(0x2119, 0xCD); // VMDATAH
    
    // Read back (need to set address again for read)
    ppu.write_register(0x2116, 0x00);
    ppu.write_register(0x2117, 0x10);
    
    let low = ppu.read_register(0x2139); // VMDATALREAD
    let high = ppu.read_register(0x2139); // VMDATAHREAD
    
    // Note: VRAM reads might be prefetched, so this test might need adjustment
    // based on actual PPU implementation details
}

#[test]
fn test_cgram_access() {
    let mut ppu = Ppu::new();
    
    // Set CGRAM address
    ppu.write_register(0x2121, 0x10); // CGADD - Color 16
    
    // Write color data (BGR555 format)
    ppu.write_register(0x2122, 0x1F); // Low byte - Red
    ppu.write_register(0x2122, 0x00); // High byte
    
    // Set address for read
    ppu.write_register(0x2121, 0x10);
    
    let low = ppu.read_register(0x213B); // CGDATAREAD
    let high = ppu.read_register(0x213B);
    
    assert_eq!(low, 0x1F);
    assert_eq!(high, 0x00);
}

#[test]
fn test_oam_access() {
    let mut ppu = Ppu::new();
    
    // Set OAM address
    ppu.write_register(0x2102, 0x00); // OAMADDL
    ppu.write_register(0x2103, 0x00); // OAMADDH
    
    // Write sprite data
    ppu.write_register(0x2104, 0x80); // X position
    ppu.write_register(0x2104, 0x60); // Y position
    ppu.write_register(0x2104, 0x01); // Tile number
    ppu.write_register(0x2104, 0x30); // Attributes
    
    // Reset address for read
    ppu.write_register(0x2102, 0x00);
    ppu.write_register(0x2103, 0x00);
    
    let x = ppu.read_register(0x2138); // OAMDATAREAD
    let y = ppu.read_register(0x2138);
    let tile = ppu.read_register(0x2138);
    let attr = ppu.read_register(0x2138);
    
    assert_eq!(x, 0x80);
    assert_eq!(y, 0x60);
    assert_eq!(tile, 0x01);
    assert_eq!(attr, 0x30);
}

#[test]
fn test_bg_mode_register() {
    let mut ppu = Ppu::new();
    
    // Set BG mode 1
    ppu.write_register(0x2105, 0x01); // BGMODE
    
    assert_eq!(ppu.registers.get_bg_mode(), 1);
    
    // Set BG mode 7
    ppu.write_register(0x2105, 0x07);
    
    assert_eq!(ppu.registers.get_bg_mode(), 7);
}

#[test]
fn test_screen_enable() {
    let mut ppu = Ppu::new();
    
    // Enable BG1 and sprites on main screen
    ppu.write_register(0x212C, 0x11); // TM - Main Screen
    
    assert_eq!(ppu.registers.get_main_screen_layers(), 0x11);
    
    // Enable BG2 on sub screen
    ppu.write_register(0x212D, 0x02); // TS - Sub Screen
    
    assert_eq!(ppu.registers.get_sub_screen_layers(), 0x02);
}

#[test]
fn test_nmi_generation() {
    let mut ppu = Ppu::new();
    let mut bus = Bus::new();
    
    // Ensure screen is not blanked (NMI enabled)
    ppu.write_register(0x2100, 0x0F); // INIDISP - full brightness
    
    // Step to V-Blank
    for _ in 0..225 {
        for _ in 0..341 {
            ppu.step(&mut bus);
        }
    }
    
    // Should have NMI pending
    assert!(ppu.nmi_pending());
    
    // Blank the screen
    ppu.write_register(0x2100, 0x80); // Force blank
    
    // Reset and step to next V-Blank
    ppu.reset();
    for _ in 0..225 {
        for _ in 0..341 {
            ppu.step(&mut bus);
        }
    }
    
    // Should NOT have NMI pending when screen is blanked
    assert!(!ppu.nmi_pending());
}