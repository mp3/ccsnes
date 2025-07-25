# CCSNES - Super Nintendo Entertainment System Emulator

A high-performance SNES emulator written in Rust with WebAssembly support.

## Features

### Core Emulation
- **Complete 65C816 CPU emulation** with all addressing modes and instructions
- **Full PPU (Picture Processing Unit) implementation**
  - All background modes (0-7) including Mode 7 graphics
  - Sprite rendering with priority and size support
  - Window effects and color math
- **APU (Audio Processing Unit) emulation**
  - SPC700 CPU implementation
  - DSP audio generation
- **Memory mapping** for LoROM and HiROM cartridges
- **DMA and HDMA** controllers
- **Save states** with compression
- **Controller input** support

### Advanced Features
- **WebAssembly support** for browser-based emulation
- **Performance optimizations**
  - O(1) CPU instruction decode using static lookup tables
  - PPU tile caching system
  - Memory access cache
- **Configuration system** with TOML support
- **Comprehensive debugging tools**
  - Breakpoint manager
  - CPU execution trace
  - Performance profiler

## Building

### Prerequisites
- Rust 1.70 or later
- For WebAssembly: wasm-pack

### Native Build

```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release

# Run tests
cargo test
```

### WebAssembly Build

```bash
# Install wasm-pack if not already installed
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build for web
wasm-pack build --target web

# The output will be in the pkg/ directory
```

## Usage

### Configuration

The emulator uses a TOML configuration file stored at `~/.ccsnes/config.toml`. It will be created automatically on first run with default settings.

Example configuration:

```toml
[video]
scale = 2
fullscreen = false
vsync = true
aspect_ratio_correction = true
integer_scaling = true
scanline_intensity = 0
crt_filter = false

[audio]
master_volume = 80
sample_rate = 48000
buffer_size = 512
enabled = true
low_pass_filter = true

[input.player1]
up = "Up"
down = "Down"
left = "Left"
right = "Right"
a = "X"
b = "Z"
x = "S"
y = "A"
l = "Q"
r = "W"
select = "RShift"
start = "Return"

[emulation]
region = "Auto"
fast_forward_speed = 8.0
rewind_buffer_frames = 600
auto_save_sram = true
sram_save_interval = 10

[debug]
show_fps = false
cpu_trace = false
ppu_layer_debug = false
```

### Controls

Default keyboard mappings:

**Player 1:**
- D-Pad: Arrow Keys
- A: X
- B: Z
- X: S
- Y: A
- L: Q
- R: W
- Select: Right Shift
- Start: Enter

**Player 2:**
- D-Pad: I/J/K/L
- A: G
- B: F
- X: T
- Y: R
- L: E
- R: Y
- Select: V
- Start: B

## Architecture

The emulator is organized into the following modules:

- `cpu/`: 65C816 CPU implementation
  - Complete instruction set
  - All addressing modes
  - Interrupt handling
- `ppu/`: Picture Processing Unit
  - Background rendering
  - Sprite rendering
  - Mode 7 support
  - Scrolling and windows
- `apu/`: Audio Processing Unit
  - SPC700 CPU
  - DSP for sound generation
- `memory/`: Memory bus and mappers
  - LoROM/HiROM support
  - Memory-mapped I/O
  - Caching system
- `cartridge/`: ROM loading and parsing
- `dma/`: DMA and HDMA controllers
- `input/`: Controller input handling
- `savestate/`: Save state serialization
- `config/`: Configuration management
- `debug/`: Debugging tools

## Debugging Features

### Breakpoints
- PC (execution) breakpoints
- Memory read/write breakpoints
- Conditional breakpoints based on register values

### CPU Trace
- Instruction-level tracing
- Configurable filters (PC range, banks, instruction types)
- Circular buffer for efficiency
- Export to file

### Performance Profiler
- Frame timing statistics
- Function-level profiling
- Hot spot detection
- Component breakdown (CPU, PPU, APU)

## Performance Optimizations

- **Static instruction decode table**: O(1) opcode lookup instead of large match statements
- **Tile caching**: Pre-decoded tile data to avoid repeated decoding
- **Memory access cache**: Direct-mapped cache for frequently accessed memory regions
- **Optimized pixel operations**: Priority-based scanline rendering

## Save States

The emulator supports compressed save states that include:
- Complete CPU state
- PPU state including VRAM, CGRAM, and OAM
- APU state with SPC700 and DSP
- Memory contents (WRAM and cartridge SRAM)
- DMA controller state

Save states use gzip compression and include version checking for compatibility.

## Testing

Run the test suite with:

```bash
cargo test
```

Tests include:
- CPU instruction tests
- PPU rendering tests
- DMA transfer tests
- Memory mapping tests
- Save state tests

## License

This project is licensed under the MIT License.

## Acknowledgments

- The SNES development community for comprehensive documentation
- Rust community for excellent tooling and libraries
- Various open-source SNES emulator projects for reference