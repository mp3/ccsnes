// Configuration system for the SNES emulator
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Video settings
    pub video: VideoConfig,
    
    // Audio settings
    pub audio: AudioConfig,
    
    // Input settings
    pub input: InputConfig,
    
    // Emulation settings
    pub emulation: EmulationConfig,
    
    // Path settings
    pub paths: PathConfig,
    
    // Debug settings
    pub debug: DebugConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    // Window scale (1-4)
    pub scale: u32,
    
    // Fullscreen mode
    pub fullscreen: bool,
    
    // VSync enable
    pub vsync: bool,
    
    // Aspect ratio correction
    pub aspect_ratio_correction: bool,
    
    // Integer scaling only
    pub integer_scaling: bool,
    
    // Scanline effect intensity (0-100)
    pub scanline_intensity: u8,
    
    // CRT filter enable
    pub crt_filter: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    // Master volume (0-100)
    pub master_volume: u8,
    
    // Sample rate (32000, 44100, 48000)
    pub sample_rate: u32,
    
    // Audio buffer size in frames
    pub buffer_size: u32,
    
    // Enable audio
    pub enabled: bool,
    
    // Low-pass filter
    pub low_pass_filter: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    // Controller mappings for player 1
    pub player1: ControllerMapping,
    
    // Controller mappings for player 2
    pub player2: ControllerMapping,
    
    // Turbo button speed (frames between presses)
    pub turbo_speed: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerMapping {
    // D-Pad
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
    
    // Face buttons
    pub a: String,
    pub b: String,
    pub x: String,
    pub y: String,
    
    // Shoulder buttons
    pub l: String,
    pub r: String,
    
    // Other buttons
    pub select: String,
    pub start: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulationConfig {
    // Region (NTSC/PAL)
    pub region: Region,
    
    // Fast forward speed multiplier
    pub fast_forward_speed: f32,
    
    // Rewind buffer size in frames
    pub rewind_buffer_frames: u32,
    
    // Auto-save SRAM
    pub auto_save_sram: bool,
    
    // SRAM save interval (seconds)
    pub sram_save_interval: u32,
    
    // Run ahead frames (for input lag reduction)
    pub run_ahead_frames: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Region {
    NTSC,
    PAL,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    // ROM directory
    pub rom_dir: PathBuf,
    
    // Save state directory
    pub save_state_dir: PathBuf,
    
    // SRAM directory
    pub sram_dir: PathBuf,
    
    // Screenshot directory
    pub screenshot_dir: PathBuf,
    
    // BIOS/firmware directory
    pub bios_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    // Show FPS counter
    pub show_fps: bool,
    
    // Show frame time
    pub show_frame_time: bool,
    
    // CPU trace logging
    pub cpu_trace: bool,
    
    // PPU layer debugging
    pub ppu_layer_debug: bool,
    
    // Memory access logging
    pub memory_trace: bool,
    
    // Performance profiling
    pub profiling: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            video: VideoConfig::default(),
            audio: AudioConfig::default(),
            input: InputConfig::default(),
            emulation: EmulationConfig::default(),
            paths: PathConfig::default(),
            debug: DebugConfig::default(),
        }
    }
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            scale: 2,
            fullscreen: false,
            vsync: true,
            aspect_ratio_correction: true,
            integer_scaling: true,
            scanline_intensity: 0,
            crt_filter: false,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 80,
            sample_rate: 48000,
            buffer_size: 512,
            enabled: true,
            low_pass_filter: true,
        }
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            player1: ControllerMapping::default_player1(),
            player2: ControllerMapping::default_player2(),
            turbo_speed: 6,
        }
    }
}

impl ControllerMapping {
    pub fn default_player1() -> Self {
        Self {
            up: "Up".to_string(),
            down: "Down".to_string(),
            left: "Left".to_string(),
            right: "Right".to_string(),
            a: "X".to_string(),
            b: "Z".to_string(),
            x: "S".to_string(),
            y: "A".to_string(),
            l: "Q".to_string(),
            r: "W".to_string(),
            select: "RShift".to_string(),
            start: "Return".to_string(),
        }
    }
    
    pub fn default_player2() -> Self {
        Self {
            up: "I".to_string(),
            down: "K".to_string(),
            left: "J".to_string(),
            right: "L".to_string(),
            a: "G".to_string(),
            b: "F".to_string(),
            x: "T".to_string(),
            y: "R".to_string(),
            l: "E".to_string(),
            r: "Y".to_string(),
            select: "V".to_string(),
            start: "B".to_string(),
        }
    }
}

impl Default for EmulationConfig {
    fn default() -> Self {
        Self {
            region: Region::Auto,
            fast_forward_speed: 8.0,
            rewind_buffer_frames: 600, // 10 seconds at 60fps
            auto_save_sram: true,
            sram_save_interval: 10,
            run_ahead_frames: 0,
        }
    }
}

impl Default for PathConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let base = home.join(".ccsnes");
        
        Self {
            rom_dir: base.join("roms"),
            save_state_dir: base.join("saves"),
            sram_dir: base.join("sram"),
            screenshot_dir: base.join("screenshots"),
            bios_dir: base.join("bios"),
        }
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            show_fps: false,
            show_frame_time: false,
            cpu_trace: false,
            ppu_layer_debug: false,
            memory_trace: false,
            profiling: false,
        }
    }
}

impl Config {
    // Load configuration from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config = toml::from_str(&contents)?;
        Ok(config)
    }
    
    // Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let contents = toml::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
    
    // Get default config path
    pub fn default_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".ccsnes").join("config.toml")
    }
    
    // Load or create default config
    pub fn load_or_default() -> Self {
        let path = Self::default_path();
        
        if path.exists() {
            match Self::load_from_file(&path) {
                Ok(config) => {
                    log::info!("Loaded config from {:?}", path);
                    config
                }
                Err(e) => {
                    log::warn!("Failed to load config: {}, using defaults", e);
                    Self::default()
                }
            }
        } else {
            log::info!("No config file found, using defaults");
            let config = Self::default();
            
            // Try to create config directory and save default config
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = config.save_to_file(&path);
            
            config
        }
    }
    
    // Create necessary directories
    pub fn create_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.paths.rom_dir)?;
        fs::create_dir_all(&self.paths.save_state_dir)?;
        fs::create_dir_all(&self.paths.sram_dir)?;
        fs::create_dir_all(&self.paths.screenshot_dir)?;
        fs::create_dir_all(&self.paths.bios_dir)?;
        Ok(())
    }
}