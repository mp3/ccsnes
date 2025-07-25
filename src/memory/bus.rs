use crate::cartridge::Cartridge;
use crate::input::Input;
use crate::apu::Apu;
use crate::savestate::MemoryState;
use crate::Result;

const WRAM_SIZE: usize = 0x20000; // 128KB Work RAM
const VRAM_SIZE: usize = 0x10000; // 64KB Video RAM
const OAM_SIZE: usize = 0x220;    // 544 bytes OAM (Object Attribute Memory)
const CGRAM_SIZE: usize = 0x200;  // 512 bytes Color Generator RAM

pub struct Bus {
    wram: Vec<u8>,       // $7E0000-$7FFFFF: Work RAM
    vram: Vec<u8>,       // PPU Video RAM
    oam: Vec<u8>,        // PPU Object Attribute Memory
    cgram: Vec<u8>,      // PPU Color Generator RAM
    
    cartridge: Option<*mut Cartridge>,
    
    // PPU registers ($2100-$213F)
    ppu_regs: [u8; 0x40],
    
    // APU registers ($2140-$217F)
    apu_regs: [u8; 0x40],
    
    // Controller registers ($4016-$4017, $4200-$421F)
    controller_regs: [u8; 0x20],
    
    // DMA registers ($4300-$437F)
    dma_regs: [u8; 0x80],
    
    // Input system pointer
    input: Option<*mut Input>,
    
    // APU pointer
    apu: Option<*mut Apu>,
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
            input: None,
            apu: None,
        }
    }

    pub fn install_cartridge(&mut self, cartridge: &mut Cartridge) {
        self.cartridge = Some(cartridge as *mut Cartridge);
    }
    
    pub fn connect_input(&mut self, input: &mut Input) {
        self.input = Some(input as *mut Input);
    }
    
    pub fn connect_apu(&mut self, apu: &mut Apu) {
        self.apu = Some(apu as *mut Apu);
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
                    0x2140..=0x217F => {
                        if let Some(apu_ptr) = self.apu {
                            let apu = unsafe { &*apu_ptr };
                            // Read from APU ports 0-3
                            match addr {
                                0x2140 => apu.read_port(0),
                                0x2141 => apu.read_port(1),
                                0x2142 => apu.read_port(2),
                                0x2143 => apu.read_port(3),
                                _ => 0,
                            }
                        } else {
                            self.apu_regs[(addr - 0x2140) as usize]
                        }
                    }
                    
                    // Controller registers ($4016-$4017)
                    0x4016..=0x4017 => self.read_controller(addr as u16),
                    
                    // System registers ($4200-$421F)
                    0x4200..=0x421F => self.controller_regs[(addr - 0x4200 + 2) as usize],
                    
                    // DMA registers ($4300-$437F)
                    0x4300..=0x437F => self.dma_regs[(addr - 0x4300) as usize],
                    
                    // ROM area ($8000-$FFFF in banks $00-$3F, $0000-$FFFF in banks $80-$BF)
                    _ => {
                        if self.cartridge.is_none() && addr >= 0x8000 {
                            // For testing: read from upper WRAM when no cartridge loaded
                            let test_addr = (addr - 0x8000) as usize;
                            if test_addr < self.wram.len() - 0x8000 {
                                self.wram[0x8000 + test_addr]
                            } else {
                                0
                            }
                        } else {
                            self.read_cartridge(address)
                        }
                    }
                }
            }
            
            // Banks $40-$7D: Upper ROM area (but $70-$7D might be SRAM)
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
                    0x2140..=0x217F => {
                        if let Some(apu_ptr) = self.apu {
                            let apu = unsafe { &mut *apu_ptr };
                            // Write to APU ports 0-3
                            match addr {
                                0x2140 => apu.write_port(0, value),
                                0x2141 => apu.write_port(1, value),
                                0x2142 => apu.write_port(2, value),
                                0x2143 => apu.write_port(3, value),
                                _ => {}
                            }
                        } else {
                            self.apu_regs[(addr - 0x2140) as usize] = value;
                        }
                    }
                    
                    // Controller registers ($4016-$4017)
                    0x4016..=0x4017 => self.write_controller(addr as u16, value),
                    
                    // System registers ($4200-$421F)
                    0x4200..=0x421F => self.controller_regs[(addr - 0x4200 + 2) as usize] = value,
                    
                    // DMA registers ($4300-$437F)
                    0x4300..=0x437F => self.dma_regs[(addr - 0x4300) as usize] = value,
                    
                    // ROM area - normally read only, but allow writes for testing when no cartridge loaded
                    _ => {
                        if self.cartridge.is_none() && addr >= 0x8000 {
                            // For testing: store ROM area writes in upper WRAM
                            let test_addr = (addr - 0x8000) as usize;
                            if test_addr < self.wram.len() - 0x8000 {
                                self.wram[0x8000 + test_addr] = value;
                            }
                        } else if self.cartridge.is_some() {
                            // Pass writes to cartridge (for SRAM)
                            self.write_cartridge(address, value);
                        }
                    }
                }
            }
            
            // Banks $40-$7D: Check for SRAM writes
            0x40..=0x7D => {
                if self.cartridge.is_some() {
                    self.write_cartridge(address, value);
                }
            }
            
            // Banks $7E-$7F: Work RAM
            0x7E => self.wram[addr as usize] = value,
            0x7F => {
                if addr <= 0xFFFF {
                    self.wram[(0x10000 + addr) as usize] = value;
                }
            }
            
            // Other banks - mostly ROM, but might have SRAM
            _ => {
                if self.cartridge.is_some() {
                    self.write_cartridge(address, value);
                }
            }
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
    
    fn write_cartridge(&mut self, address: u32, value: u8) {
        if let Some(cartridge_ptr) = self.cartridge {
            unsafe {
                let cartridge = &mut *(cartridge_ptr as *mut Cartridge);
                cartridge.write(address, value);
            }
        }
    }

    fn read_ppu_register(&self, addr: u16) -> u8 {
        // PPU register reads are handled by the PPU itself
        // For now, return the cached value
        if addr >= 0x2100 && addr <= 0x213F {
            self.ppu_regs[(addr - 0x2100) as usize]
        } else {
            0
        }
    }

    fn write_ppu_register(&mut self, addr: u16, value: u8) {
        // Cache the value for direct access
        if addr >= 0x2100 && addr <= 0x213F {
            self.ppu_regs[(addr - 0x2100) as usize] = value;
        }
        // Actual PPU register writes are handled by the PPU itself
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
    
    fn read_controller(&self, addr: u16) -> u8 {
        if let Some(input_ptr) = self.input {
            unsafe {
                let input = &mut *input_ptr;
                match addr {
                    0x4016 => {
                        // Controller 1 data
                        input.read_controller(0)
                    }
                    0x4017 => {
                        // Controller 2 data
                        input.read_controller(1)
                    }
                    _ => 0,
                }
            }
        } else {
            0
        }
    }
    
    fn write_controller(&mut self, addr: u16, value: u8) {
        match addr {
            0x4016 => {
                // Controller strobe register
                if let Some(input_ptr) = self.input {
                    unsafe {
                        let input = &mut *input_ptr;
                        input.strobe_controllers((value & 0x01) != 0);
                    }
                }
                self.controller_regs[0] = value;
            }
            0x4017 => {
                // Controller 2 port (not used for standard controllers)
                self.controller_regs[1] = value;
            }
            _ => {}
        }
    }
    
    // Save state functionality
    pub fn save_memory_state(&self) -> MemoryState {
        let sram = if let Some(cartridge_ptr) = self.cartridge {
            unsafe {
                let cartridge = &*cartridge_ptr;
                cartridge.get_sram().map(|s| s.to_vec())
            }
        } else {
            None
        };
        
        MemoryState {
            wram: self.wram.clone(),
            sram,
        }
    }
    
    pub fn load_memory_state(&mut self, state: &MemoryState) -> Result<()> {
        self.wram = state.wram.clone();
        
        if let (Some(sram_data), Some(cartridge_ptr)) = (&state.sram, self.cartridge) {
            unsafe {
                let cartridge = &mut *cartridge_ptr;
                cartridge.load_sram(sram_data)?;
            }
        }
        
        Ok(())
    }
}