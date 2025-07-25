// Custom error types for better error handling
use thiserror::Error;
use std::fmt;

#[derive(Error, Debug)]
pub enum EmulatorError {
    #[error("ROM loading error: {0}")]
    RomLoadError(String),
    
    #[error("Invalid ROM format: {0}")]
    InvalidRomFormat(String),
    
    #[error("Invalid mapper type: {0}")]
    InvalidMapperType(String),
    
    #[error("Memory error: {0}")]
    MemoryError(String),
    
    #[error("CPU error: {0}")]
    CpuError(String),
    
    #[error("PPU error: {0}")]
    PpuError(String),
    
    #[error("APU error: {0}")]
    ApuError(String),
    
    #[error("Save state error: {0}")]
    SaveStateError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Input error: {0}")]
    InputError(String),
    
    #[error("Audio error: {0}")]
    AudioError(String),
    
    #[error("Video error: {0}")]
    VideoError(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),
    
    #[error("TOML deserialization error: {0}")]
    TomlDeError(#[from] toml::de::Error),
    
    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),
}

impl EmulatorError {
    /// Create a ROM loading error
    pub fn rom_load<S: Into<String>>(msg: S) -> Self {
        EmulatorError::RomLoadError(msg.into())
    }
    
    /// Create an invalid ROM format error
    pub fn invalid_rom<S: Into<String>>(msg: S) -> Self {
        EmulatorError::InvalidRomFormat(msg.into())
    }
    
    /// Create a memory error
    pub fn memory<S: Into<String>>(msg: S) -> Self {
        EmulatorError::MemoryError(msg.into())
    }
    
    /// Create a CPU error
    pub fn cpu<S: Into<String>>(msg: S) -> Self {
        EmulatorError::CpuError(msg.into())
    }
    
    /// Create a PPU error
    pub fn ppu<S: Into<String>>(msg: S) -> Self {
        EmulatorError::PpuError(msg.into())
    }
    
    /// Create an APU error
    pub fn apu<S: Into<String>>(msg: S) -> Self {
        EmulatorError::ApuError(msg.into())
    }
    
    /// Create a save state error
    pub fn save_state<S: Into<String>>(msg: S) -> Self {
        EmulatorError::SaveStateError(msg.into())
    }
    
    /// Create a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        EmulatorError::ConfigError(msg.into())
    }
    
    /// Create an input error
    pub fn input<S: Into<String>>(msg: S) -> Self {
        EmulatorError::InputError(msg.into())
    }
    
    /// Create an audio error
    pub fn audio<S: Into<String>>(msg: S) -> Self {
        EmulatorError::AudioError(msg.into())
    }
    
    /// Create a video error
    pub fn video<S: Into<String>>(msg: S) -> Self {
        EmulatorError::VideoError(msg.into())
    }
}

/// Result type alias for emulator operations
pub type EmulatorResult<T> = Result<T, EmulatorError>;

/// Error context for debugging
pub struct ErrorContext {
    pub component: String,
    pub operation: String,
    pub details: Option<String>,
}

impl ErrorContext {
    pub fn new(component: &str, operation: &str) -> Self {
        Self {
            component: component.to_string(),
            operation: operation.to_string(),
            details: None,
        }
    }
    
    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.component, self.operation)?;
        if let Some(ref details) = self.details {
            write!(f, " - {}", details)?;
        }
        Ok(())
    }
}

/// Trait for adding context to errors
pub trait ErrorContextExt<T> {
    fn context(self, ctx: ErrorContext) -> EmulatorResult<T>;
}

impl<T, E> ErrorContextExt<T> for Result<T, E>
where
    E: Into<EmulatorError>,
{
    fn context(self, ctx: ErrorContext) -> EmulatorResult<T> {
        self.map_err(|e| {
            let mut err = e.into();
            // Add context information to error message
            match &mut err {
                EmulatorError::RomLoadError(msg) |
                EmulatorError::InvalidRomFormat(msg) |
                EmulatorError::MemoryError(msg) |
                EmulatorError::CpuError(msg) |
                EmulatorError::PpuError(msg) |
                EmulatorError::ApuError(msg) |
                EmulatorError::SaveStateError(msg) |
                EmulatorError::ConfigError(msg) |
                EmulatorError::InputError(msg) |
                EmulatorError::AudioError(msg) |
                EmulatorError::VideoError(msg) => {
                    *msg = format!("{}: {}", ctx, msg);
                }
                _ => {}
            }
            err
        })
    }
}