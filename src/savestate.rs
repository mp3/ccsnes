use crate::{Result, EmulatorError};
use serde::{Serialize, Deserialize};
use std::fs::File;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;

// Save state version for compatibility checking
const SAVE_STATE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    // Version info
    pub version: u32,
    
    // CPU state
    pub cpu: CpuState,
    
    // PPU state
    pub ppu: PpuState,
    
    // APU state
    pub apu: ApuState,
    
    // Memory state
    pub memory: MemoryState,
    
    // DMA state
    pub dma: DmaState,
    
    // Emulator state
    pub cycles: u64,
}

#[derive(Serialize, Deserialize)]
pub struct CpuState {
    // Registers
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub s: u16,
    pub d: u16,
    pub db: u8,
    pub pb: u8,
    pub pc: u16,
    pub p: u8,
    pub emulation_mode: bool,
    
    // Internal state
    pub stopped: bool,
    pub waiting_for_interrupt: bool,
    pub nmi_pending: bool,
    pub irq_pending: bool,
}

#[derive(Serialize, Deserialize)]
pub struct PpuState {
    // Registers
    pub registers: Vec<u8>,
    
    // VRAM
    pub vram: Vec<u8>,
    
    // CGRAM
    pub cgram: Vec<u8>,
    
    // OAM
    pub oam: Vec<u8>,
    
    // Internal state
    pub current_scanline: u16,
    pub current_cycle: u16,
    pub frame_count: u64,
    pub vblank: bool,
    pub hblank: bool,
    pub nmi_flag: bool,
    pub irq_flag: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ApuState {
    // SPC700 state
    pub spc700: Spc700State,
    
    // DSP state
    pub dsp: DspState,
    
    // Audio buffer
    pub audio_buffer: Vec<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct Spc700State {
    // Registers
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub psw: u8,
    
    // Memory
    pub ram: Vec<u8>,
    
    // I/O state
    pub ipl_rom_enable: bool,
    pub port_in: [u8; 4],
    pub port_out: [u8; 4],
    pub timer_enable: u8,
    pub timer_target: [u8; 3],
    pub timer_counter: [u8; 3],
    pub timer_output: [u8; 3],
    
    pub cycles: u64,
}

#[derive(Serialize, Deserialize)]
pub struct DspState {
    pub channels: Vec<ChannelState>,
    pub main_volume_left: u8,
    pub main_volume_right: u8,
    pub echo_volume_left: u8,
    pub echo_volume_right: u8,
    pub sample_counter: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChannelState {
    pub volume_left: u8,
    pub volume_right: u8,
    pub pitch: u16,
    pub source_number: u8,
    pub adsr: u16,
    pub gain: u8,
    pub envelope: u16,
}

#[derive(Serialize, Deserialize)]
pub struct MemoryState {
    // Work RAM
    pub wram: Vec<u8>,
    
    // Cartridge SRAM (if present)
    pub sram: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
pub struct DmaState {
    pub channels: Vec<DmaChannelState>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DmaChannelState {
    pub enabled: bool,
    pub hdma_enabled: bool,
    pub direction: u8,
    pub indirect: bool,
    pub reverse_transfer: bool,
    pub fixed_transfer: bool,
    pub transfer_mode: u8,
    
    pub b_address: u8,
    pub a_address: u16,
    pub a_bank: u8,
    pub transfer_size: u16,
    pub indirect_bank: u8,
    
    pub hdma_line_counter: u8,
    pub hdma_address: u16,
    pub hdma_completed: bool,
}

impl SaveState {
    pub fn new() -> Self {
        Self {
            version: SAVE_STATE_VERSION,
            cpu: CpuState::default(),
            ppu: PpuState::default(),
            apu: ApuState::default(),
            memory: MemoryState::default(),
            dma: DmaState::default(),
            cycles: 0,
        }
    }
    
    /// Save the state to a file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let file = File::create(path)?;
        let encoder = GzEncoder::new(file, Compression::default());
        
        bincode::serialize_into(encoder, self)
            .map_err(|e| EmulatorError::SaveStateError(format!("Failed to serialize save state: {}", e)))?;
            
        Ok(())
    }
    
    /// Load the state from a file
    pub fn load_from_file(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let decoder = GzDecoder::new(file);
        
        let state: SaveState = bincode::deserialize_from(decoder)
            .map_err(|e| EmulatorError::SaveStateError(format!("Failed to deserialize save state: {}", e)))?;
            
        // Check version compatibility
        if state.version != SAVE_STATE_VERSION {
            return Err(EmulatorError::SaveStateError(format!(
                "Save state version mismatch: expected {}, got {}",
                SAVE_STATE_VERSION, state.version
            )));
        }
        
        Ok(state)
    }
}

// Default implementations
impl Default for CpuState {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            s: 0x01FF,
            d: 0,
            db: 0,
            pb: 0,
            pc: 0,
            p: 0x34,
            emulation_mode: true,
            stopped: false,
            waiting_for_interrupt: false,
            nmi_pending: false,
            irq_pending: false,
        }
    }
}

impl Default for PpuState {
    fn default() -> Self {
        Self {
            registers: vec![0; 0x40],
            vram: vec![0; 0x10000],
            cgram: vec![0; 0x200],
            oam: vec![0; 0x220],
            current_scanline: 0,
            current_cycle: 0,
            frame_count: 0,
            vblank: false,
            hblank: false,
            nmi_flag: false,
            irq_flag: false,
        }
    }
}

impl Default for ApuState {
    fn default() -> Self {
        Self {
            spc700: Spc700State::default(),
            dsp: DspState::default(),
            audio_buffer: Vec::new(),
        }
    }
}

impl Default for Spc700State {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFF,
            pc: 0xFFC0,
            psw: 0x02,
            ram: vec![0; 0x10000],
            ipl_rom_enable: true,
            port_in: [0; 4],
            port_out: [0; 4],
            timer_enable: 0,
            timer_target: [0; 3],
            timer_counter: [0; 3],
            timer_output: [0; 3],
            cycles: 0,
        }
    }
}

impl Default for DspState {
    fn default() -> Self {
        Self {
            channels: vec![ChannelState::default(); 8],
            main_volume_left: 0,
            main_volume_right: 0,
            echo_volume_left: 0,
            echo_volume_right: 0,
            sample_counter: 0,
        }
    }
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            volume_left: 0,
            volume_right: 0,
            pitch: 0,
            source_number: 0,
            adsr: 0,
            gain: 0,
            envelope: 0,
        }
    }
}

impl Default for MemoryState {
    fn default() -> Self {
        Self {
            wram: vec![0; 0x20000],
            sram: None,
        }
    }
}

impl Default for DmaState {
    fn default() -> Self {
        Self {
            channels: vec![DmaChannelState::default(); 8],
        }
    }
}

impl Default for DmaChannelState {
    fn default() -> Self {
        Self {
            enabled: false,
            hdma_enabled: false,
            direction: 0,
            indirect: false,
            reverse_transfer: false,
            fixed_transfer: false,
            transfer_mode: 0,
            b_address: 0,
            a_address: 0,
            a_bank: 0,
            transfer_size: 0,
            indirect_bank: 0,
            hdma_line_counter: 0,
            hdma_address: 0,
            hdma_completed: false,
        }
    }
}