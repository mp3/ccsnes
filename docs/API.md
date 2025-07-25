# CCSNES API Documentation

This document describes the public API for using CCSNES as a library in your own projects.

## Core Types

### `Emulator`

The main emulator instance that manages all hardware components.

```rust
use ccsnes::{Emulator, Result};

// Create a new emulator instance
let mut emulator = Emulator::new()?;

// Load a ROM
let rom_data = std::fs::read("game.sfc")?;
emulator.load_rom(&rom_data)?;

// Run one frame
emulator.step_frame()?;

// Get video output (RGB565 format, 256x224)
let frame_buffer = emulator.get_video_buffer();

// Get audio samples (stereo f32)
let audio_samples = emulator.get_audio_samples();
```

### Methods

#### `Emulator::new() -> Result<Self>`
Creates a new emulator instance with default settings.

#### `Emulator::load_rom(&mut self, rom_data: &[u8]) -> Result<()>`
Loads a ROM from a byte array. Automatically detects the mapper type.

#### `Emulator::reset(&mut self) -> Result<()>`
Performs a soft reset of the system.

#### `Emulator::step(&mut self) -> Result<()>`
Executes one CPU instruction. Useful for debugging.

#### `Emulator::step_frame(&mut self) -> Result<()>`
Executes emulation for one frame (1/60th of a second).

#### `Emulator::set_controller_input(&mut self, player: u8, buttons: u16)`
Sets the controller state for a player (0-3).

Button bits:
- `0x0001` - Right
- `0x0002` - Left  
- `0x0004` - Down
- `0x0008` - Up
- `0x0010` - Start
- `0x0020` - Select
- `0x0040` - Y
- `0x0080` - B
- `0x0100` - (unused)
- `0x0200` - (unused)
- `0x0400` - (unused)
- `0x0800` - (unused)
- `0x1000` - R
- `0x2000` - L
- `0x4000` - X
- `0x8000` - A

#### `Emulator::get_video_buffer(&self) -> &[u8]`
Returns the current frame buffer in RGB565 format (512 bytes per scanline, 224 scanlines).

#### `Emulator::get_audio_samples(&mut self) -> Vec<f32>`
Returns and clears the audio sample buffer. Samples are stereo interleaved at 32kHz.

#### `Emulator::save_state(&self) -> Result<SaveState>`
Creates a save state of the current emulation state.

#### `Emulator::load_state(&mut self, state: &SaveState) -> Result<()>`
Loads a previously saved state.

#### `Emulator::save_state_to_file(&self, path: &str) -> Result<()>`
Saves the current state to a file with compression.

#### `Emulator::load_state_from_file(&mut self, path: &str) -> Result<()>`
Loads a state from a compressed file.

#### `Emulator::load_sram(&mut self, sram_data: &[u8]) -> Result<()>`
Loads cartridge SRAM data (for games with battery backup).

#### `Emulator::get_sram(&self) -> Option<Vec<u8>>`
Gets the current SRAM contents if the game has battery backup.

#### `Emulator::get_rom_info(&self) -> Option<RomInfo>`
Gets information about the loaded ROM.

## Configuration

### `Config`

Configuration structure for the emulator.

```rust
use ccsnes::config::Config;

// Load default configuration
let config = Config::default();

// Load from file
let config = Config::load_from_file("config.toml")?;

// Save to file
config.save_to_file("config.toml")?;
```

## Error Handling

CCSNES uses a custom error type `EmulatorError` with the following variants:

- `RomLoadError`: ROM loading failed
- `InvalidRomFormat`: ROM format not recognized
- `MemoryError`: Memory access error
- `CpuError`: CPU execution error
- `SaveStateError`: Save state operation failed
- `ConfigError`: Configuration error
- `AudioError`: Audio subsystem error
- `VideoError`: Video subsystem error

## WebAssembly API

When compiled to WebAssembly, CCSNES exposes these additional functions:

```javascript
import init, { WasmEmulator } from './ccsnes_wasm.js';

await init();

// Create emulator
const emulator = new WasmEmulator();

// Load ROM
const romData = new Uint8Array(await fetch('game.sfc').then(r => r.arrayBuffer()));
emulator.load_rom(romData);

// Run frame
emulator.step_frame();

// Get video buffer (RGBA format for web)
const videoBuffer = emulator.get_frame_buffer_rgba();

// Set input
emulator.set_input(0, buttons);

// Get audio buffer
const audioBuffer = emulator.get_audio_buffer();
```

## Example: Basic Emulator Loop

```rust
use ccsnes::{Emulator, Result};
use std::time::{Duration, Instant};

fn run_emulator(rom_path: &str) -> Result<()> {
    let mut emulator = Emulator::new()?;
    let rom_data = std::fs::read(rom_path)?;
    emulator.load_rom(&rom_data)?;
    
    let frame_duration = Duration::from_secs_f64(1.0 / 60.0);
    let mut last_frame = Instant::now();
    
    loop {
        // Check if it's time for next frame
        let now = Instant::now();
        if now.duration_since(last_frame) >= frame_duration {
            last_frame = now;
            
            // Run one frame
            emulator.step_frame()?;
            
            // Process video
            let video = emulator.get_video_buffer();
            // ... render video ...
            
            // Process audio
            let audio = emulator.get_audio_samples();
            // ... play audio ...
        }
        
        // Handle input
        let buttons = get_input_state(); // Your input handling
        emulator.set_controller_input(0, buttons);
    }
}
```