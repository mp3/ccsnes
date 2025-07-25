use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::dma::DmaController;
use crate::input::Input;
use crate::memory::Bus;
use crate::ppu::Ppu;
use crate::savestate::SaveState;
use crate::Result;
use log::{debug, info};

pub struct Emulator {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub apu: Apu,
    pub dma: DmaController,
    pub bus: Bus,
    pub input: Input,
    pub cartridge: Option<Cartridge>,
    pub cycles: u64,
    pub running: bool,
    
    // Track HDMA initialization state
    hdma_init_pending: bool,
}

impl Emulator {
    pub fn new() -> Result<Self> {
        info!("Initializing SNES emulator");
        
        Ok(Self {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            apu: Apu::new(),
            dma: DmaController::new(),
            bus: Bus::new(),
            input: Input::new(),
            cartridge: None,
            cycles: 0,
            running: false,
            hdma_init_pending: false,
        })
    }

    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<()> {
        info!("Loading ROM ({} bytes)", rom_data.len());
        
        let cartridge = Cartridge::load(rom_data)?;
        info!("ROM loaded: {}", cartridge.header.title);
        info!("Mapper type: {:?}", cartridge.header.mapper_type);
        
        self.cartridge = Some(cartridge);
        if let Some(ref mut cartridge) = self.cartridge {
            self.bus.install_cartridge(cartridge);
        }
        self.bus.connect_input(&mut self.input);
        self.bus.connect_apu(&mut self.apu);
        
        self.reset()?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        debug!("Resetting emulator");
        
        self.cpu.reset(&mut self.bus)?;
        self.ppu.reset();
        self.apu.reset();
        self.dma.reset();
        self.cycles = 0;
        self.running = true;
        self.hdma_init_pending = false;
        
        Ok(())
    }

    pub fn step(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }

        // Handle DMA register writes
        let dma_enable = self.bus.read8(0x420B);
        if dma_enable != 0 {
            // Execute DMA transfers
            let dma_cycles = self.dma.execute_dma(&mut self.bus, &mut self.ppu);
            self.cycles += dma_cycles as u64;
            
            // Clear DMA enable register
            self.bus.write8(0x420B, 0);
            return Ok(());
        }
        
        // Handle HDMA initialization at start of frame
        if self.ppu.get_current_scanline() == 0 && !self.hdma_init_pending {
            let hdma_enable = self.bus.read8(0x420C);
            if hdma_enable != 0 {
                self.dma.init_hdma(&mut self.bus);
                self.hdma_init_pending = true;
            }
        }
        
        // Reset HDMA init flag when we're past scanline 0
        if self.ppu.get_current_scanline() > 0 {
            self.hdma_init_pending = false;
        }

        // Update DMA registers from bus
        for addr in 0x4300..=0x437F {
            let value = self.bus.read8(addr);
            self.dma.write_register(addr as u16, value);
        }

        // Handle PPU register reads/writes through the bus
        for addr in 0x2100..=0x213F {
            if self.bus.ppu_register(addr) != 0 {
                let value = self.bus.ppu_register(addr);
                self.ppu.write_register(addr, value);
                self.bus.set_ppu_register(addr, 0); // Clear after handling
            }
        }

        let cpu_cycles = self.cpu.step(&mut self.bus)?;
        
        // Track current scanline for HDMA
        let old_scanline = self.ppu.get_current_scanline();
        
        for _ in 0..cpu_cycles * 4 {
            self.ppu.step(&mut self.bus);
            
            // Check if we crossed a scanline boundary
            let new_scanline = self.ppu.get_current_scanline();
            if new_scanline != old_scanline && new_scanline < 224 {
                // Execute HDMA for this scanline
                let hdma_cycles = self.dma.execute_hdma(&mut self.bus, &mut self.ppu);
                self.cycles += hdma_cycles as u64;
            }
        }
        
        for _ in 0..cpu_cycles {
            self.apu.step();
        }
        
        self.cycles += cpu_cycles as u64;
        
        if self.ppu.nmi_pending() {
            self.cpu.trigger_nmi(&mut self.bus)?;
        }
        
        if self.ppu.irq_pending() {
            self.cpu.trigger_irq(&mut self.bus)?;
        }
        
        Ok(())
    }

    pub fn step_frame(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }

        let start_cycles = self.cycles;
        const CYCLES_PER_FRAME: u64 = 357366; // NTSC: ~21.477MHz / 60fps
        
        while self.cycles - start_cycles < CYCLES_PER_FRAME {
            self.step()?;
        }
        
        Ok(())
    }

    pub fn set_controller_input(&mut self, player: u8, buttons: u16) {
        self.input.set_controller_state(player, buttons);
    }

    pub fn get_video_buffer(&self) -> &[u8] {
        self.ppu.get_frame_buffer()
    }

    pub fn get_audio_samples(&mut self) -> Vec<f32> {
        self.apu.get_audio_samples()
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn pause(&mut self) {
        self.running = false;
    }

    pub fn resume(&mut self) {
        self.running = true;
    }
    
    // Save state functionality
    pub fn save_state(&self) -> Result<SaveState> {
        let mut state = SaveState::new();
        
        // Save CPU state
        state.cpu = self.cpu.save_state();
        
        // Save PPU state
        state.ppu = self.ppu.save_state();
        
        // Save APU state
        state.apu = self.apu.save_state();
        
        // Save memory state
        state.memory = self.bus.save_memory_state();
        
        // Save DMA state
        state.dma = self.dma.save_state();
        
        // Save emulator state
        state.cycles = self.cycles;
        
        Ok(state)
    }
    
    pub fn load_state(&mut self, state: &SaveState) -> Result<()> {
        // Load CPU state
        self.cpu.load_state(&state.cpu);
        
        // Load PPU state
        self.ppu.load_state(&state.ppu);
        
        // Load APU state
        self.apu.load_state(&state.apu);
        
        // Load memory state
        self.bus.load_memory_state(&state.memory)?;
        
        // Load DMA state
        self.dma.load_state(&state.dma);
        
        // Load emulator state
        self.cycles = state.cycles;
        
        Ok(())
    }
    
    pub fn save_state_to_file(&self, path: &str) -> Result<()> {
        let state = self.save_state()?;
        state.save_to_file(path)?;
        info!("Save state saved to: {}", path);
        Ok(())
    }
    
    pub fn load_state_from_file(&mut self, path: &str) -> Result<()> {
        let state = SaveState::load_from_file(path)?;
        self.load_state(&state)?;
        info!("Save state loaded from: {}", path);
        Ok(())
    }
    
    // Information and stats methods
    pub fn get_rom_info(&self) -> Option<crate::cartridge::header::RomInfo> {
        if let Some(cartridge) = self.cartridge.as_ref() {
            Some(cartridge.get_info())
        } else {
            None
        }
    }
    
    pub fn get_cycle_count(&self) -> u64 {
        self.cycles
    }
    
    pub fn get_frame_count(&self) -> u64 {
        self.ppu.get_frame_count()
    }
    
    pub fn get_frame_buffer(&self) -> &[u8] {
        self.ppu.get_frame_buffer()
    }
    
    // SRAM access methods
    pub fn load_sram(&mut self, sram_data: &[u8]) -> Result<()> {
        if let Some(cartridge) = self.cartridge.as_mut() {
            cartridge.load_sram(sram_data)?;
            info!("Loaded SRAM ({} bytes)", sram_data.len());
        }
        Ok(())
    }
    
    pub fn get_sram(&self) -> Option<Vec<u8>> {
        if let Some(cartridge) = self.cartridge.as_ref() {
            cartridge.get_sram().map(|s| s.to_vec())
        } else {
            None
        }
    }
}