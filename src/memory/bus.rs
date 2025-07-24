use crate::cartridge::Cartridge;

const WRAM_SIZE: usize = 0x20000; // 128KB Work RAM
const VRAM_SIZE: usize = 0x10000; // 64KB Video RAM
const OAM_SIZE: usize = 0x220;    // 544 bytes OAM (Object Attribute Memory)
const CGRAM_SIZE: usize = 0x200;  // 512 bytes Color Generator RAM

pub struct Bus {
    wram: Vec<u8>,       // $7E0000-$7FFFFF: Work RAM
    vram: Vec<u8>,       // PPU Video RAM
    oam: Vec<u8>,        // PPU Object Attribute Memory
    cgram: Vec<u8>,      // PPU Color Generator RAM
    
    cartridge: Option<*const Cartridge>,
    
    // PPU registers ($2100-$213F)
    ppu_regs: [u8; 0x40],
    
    // APU registers ($2140-$217F)
    apu_regs: [u8; 0x40],
    
    // Controller registers ($4016-$4017, $4200-$421F)
    controller_regs: [u8; 0x20],
    
    // DMA registers ($4300-$437F)
    dma_regs: [u8; 0x80],
}

impl Bus {
    pub fn new() -> Self {
        Self {
            wram: vec![0; WRAM_SIZE],
            vram: vec![0; VRAM_SIZE],
            oam: vec![0; OAM_SIZE],
            cgram: vec![0; CGRAM_SIZE],
            cartridge: None,
            ppu_regs: [0; 0x40],
            apu_regs: [0; 0x40],
            controller_regs: [0; 0x20],
            dma_regs: [0; 0x80],
        }
    }

    pub fn install_cartridge(&mut self, cartridge: &Cartridge) {
        self.cartridge = Some(cartridge as *const Cartridge);
    }

    pub fn read8(&self, address: u32) -> u8 {
        let bank = (address >> 16) & 0xFF;
        let addr = address & 0xFFFF;

        match bank {
            // Banks $00-$3F and $80-$BF: System area
            0x00..=0x3F | 0x80..=0xBF => {
                match addr {
                    // Low RAM mirror ($0000-$1FFF)
                    0x0000..=0x1FFF => self.wram[addr as usize],
                    
                    // PPU registers ($2100-$213F)
                    0x2100..=0x213F => self.read_ppu_register(addr as u16),
                    
                    // APU registers ($2140-$217F)
                    0x2140..=0x217F => self.apu_regs[(addr - 0x2140) as usize],
                    
                    // Controller registers ($4016-$4017)
                    0x4016..=0x4017 => self.controller_regs[(addr - 0x4016) as usize],
                    
                    // System registers ($4200-$421F)
                    0x4200..=0x421F => self.controller_regs[(addr - 0x4200 + 2) as usize],
                    
                    // DMA registers ($4300-$437F)
                    0x4300..=0x437F => self.dma_regs[(addr - 0x4300) as usize],
                    
                    // ROM area ($8000-$FFFF in banks $00-$3F, $0000-$FFFF in banks $80-$BF)
                    _ => self.read_cartridge(address),
                }
            }
            
            // Banks $40-$7D: Upper ROM area
            0x40..=0x7D => self.read_cartridge(address),
            
            // Banks $7E-$7F: Work RAM
            0x7E => self.wram[addr as usize],
            0x7F => {
                if addr <= 0xFFFF {
                    self.wram[(0x10000 + addr) as usize]
                } else {
                    0
                }
            }
            
            // Banks $C0-$FF: ROM area
            0xC0..=0xFF => self.read_cartridge(address),
            
            _ => 0,
        }
    }

    pub fn write8(&mut self, address: u32, value: u8) {
        let bank = (address >> 16) & 0xFF;
        let addr = address & 0xFFFF;

        match bank {
            // Banks $00-$3F and $80-$BF: System area
            0x00..=0x3F | 0x80..=0xBF => {
                match addr {
                    // Low RAM mirror ($0000-$1FFF)
                    0x0000..=0x1FFF => self.wram[addr as usize] = value,
                    
                    // PPU registers ($2100-$213F)
                    0x2100..=0x213F => self.write_ppu_register(addr as u16, value),
                    
                    // APU registers ($2140-$217F)
                    0x2140..=0x217F => self.apu_regs[(addr - 0x2140) as usize] = value,
                    
                    // Controller registers ($4016-$4017)
                    0x4016..=0x4017 => self.controller_regs[(addr - 0x4016) as usize] = value,
                    
                    // System registers ($4200-$421F)
                    0x4200..=0x421F => self.controller_regs[(addr - 0x4200 + 2) as usize] = value,
                    
                    // DMA registers ($4300-$437F)
                    0x4300..=0x437F => self.dma_regs[(addr - 0x4300) as usize] = value,
                    
                    // ROM area - read only
                    _ => {}
                }
            }
            
            // Banks $7E-$7F: Work RAM
            0x7E => self.wram[addr as usize] = value,
            0x7F => {
                if addr <= 0xFFFF {
                    self.wram[(0x10000 + addr) as usize] = value;
                }
            }
            
            // Other banks - mostly ROM, read only
            _ => {}
        }
    }

    pub fn read16(&self, address: u32) -> u16 {
        let low = self.read8(address) as u16;
        let high = self.read8(address + 1) as u16;
        low | (high << 8)
    }

    pub fn write16(&mut self, address: u32, value: u16) {
        self.write8(address, (value & 0xFF) as u8);
        self.write8(address + 1, (value >> 8) as u8);
    }

    pub fn read24(&self, address: u32) -> u32 {
        let low = self.read16(address) as u32;
        let high = self.read8(address + 2) as u32;
        low | (high << 16)
    }

    fn read_cartridge(&self, address: u32) -> u8 {
        if let Some(cartridge_ptr) = self.cartridge {
            unsafe {
                (*cartridge_ptr).read(address)
            }
        } else {
            0
        }
    }

    fn read_ppu_register(&self, addr: u16) -> u8 {
        match addr {
            0x2134..=0x2136 => {
                // Multiplication result registers
                self.ppu_regs[(addr - 0x2100) as usize]
            }
            0x2137 => {
                // Software latch for H/V counters
                self.ppu_regs[(addr - 0x2100) as usize]
            }
            0x2138 => {
                // OAM data read
                // TODO: Implement proper OAM reading
                self.ppu_regs[(addr - 0x2100) as usize]
            }
            0x2139..=0x213A => {
                // VRAM data read
                // TODO: Implement proper VRAM reading
                self.ppu_regs[(addr - 0x2100) as usize]
            }
            0x213B => {
                // CGRAM data read
                // TODO: Implement proper CGRAM reading
                self.ppu_regs[(addr - 0x2100) as usize]
            }
            0x213C..=0x213F => {
                // Horizontal/Vertical counter reads
                self.ppu_regs[(addr - 0x2100) as usize]
            }
            _ => {
                // Most PPU registers are write-only
                0
            }
        }
    }

    fn write_ppu_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x2118..=0x2119 => {
                // VRAM data write
                // TODO: Implement proper VRAM writing
                self.ppu_regs[(addr - 0x2100) as usize] = value;
            }
            0x2122 => {
                // CGRAM data write
                // TODO: Implement proper CGRAM writing
                self.ppu_regs[(addr - 0x2100) as usize] = value;
            }
            0x2104 => {
                // OAM data write
                // TODO: Implement proper OAM writing
                self.ppu_regs[(addr - 0x2100) as usize] = value;
            }
            _ => {
                self.ppu_regs[(addr - 0x2100) as usize] = value;
            }
        }
    }

    // Direct memory access methods for PPU
    pub fn vram(&self) -> &[u8] {
        &self.vram
    }

    pub fn vram_mut(&mut self) -> &mut [u8] {
        &mut self.vram
    }

    pub fn oam(&self) -> &[u8] {
        &self.oam
    }

    pub fn oam_mut(&mut self) -> &mut [u8] {
        &mut self.oam
    }

    pub fn cgram(&self) -> &[u8] {
        &self.cgram
    }

    pub fn cgram_mut(&mut self) -> &mut [u8] {
        &mut self.cgram
    }

    pub fn ppu_register(&self, addr: u16) -> u8 {
        if addr >= 0x2100 && addr <= 0x213F {
            self.ppu_regs[(addr - 0x2100) as usize]
        } else {
            0
        }
    }

    pub fn set_ppu_register(&mut self, addr: u16, value: u8) {
        if addr >= 0x2100 && addr <= 0x213F {
            self.ppu_regs[(addr - 0x2100) as usize] = value;
        }
    }
}