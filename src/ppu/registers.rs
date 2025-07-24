// TODO: Implement PPU registers ($2100-$213F)

pub struct PpuRegisters {
    // Display control registers
    pub inidisp: u8,    // $2100 - Screen display
    pub obsel: u8,      // $2101 - Object size and character address
    pub oamaddl: u8,    // $2102 - OAM address (low)
    pub oamaddh: u8,    // $2103 - OAM address (high)
    
    // Background control
    pub bgmode: u8,     // $2105 - BG mode and character size
    pub mosaic: u8,     // $2106 - Mosaic settings
    pub bg1sc: u8,      // $2107 - BG1 screen address
    pub bg2sc: u8,      // $2108 - BG2 screen address
    pub bg3sc: u8,      // $2109 - BG3 screen address
    pub bg4sc: u8,      // $210A - BG4 screen address
    
    // TODO: Add all other PPU registers
}

impl PpuRegisters {
    pub fn new() -> Self {
        Self {
            inidisp: 0x80, // Screen blanked by default
            obsel: 0,
            oamaddl: 0,
            oamaddh: 0,
            bgmode: 0,
            mosaic: 0,
            bg1sc: 0,
            bg2sc: 0,
            bg3sc: 0,
            bg4sc: 0,
        }
    }
}