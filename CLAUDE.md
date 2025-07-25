# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

### Native Build
```bash
# Standard release build
cargo build --release

# Development build with debug symbols
cargo build

# Run with a ROM
./target/release/ccsnes run game.sfc

# Alternative: use the build script
./build.sh native
```

### WebAssembly Build
```bash
# Install wasm-pack if needed
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build WASM module
./build-wasm.sh

# Alternative: manual build
wasm-pack build --target web --out-dir web/pkg --features wasm

# Serve the web demo
cd web && python3 -m http.server 8000
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test module
cargo test cpu_tests
cargo test ppu_tests
cargo test savestate_tests

# Run tests with output
cargo test -- --nocapture

# Run test suite with test ROMs
./tests/run_test_suite.sh

# Generate simple test ROM
cd tests && python3 create_simple_test.py
```

### Code Quality
```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Check for unused dependencies
cargo udeps

# Check WASM build
cargo check --target wasm32-unknown-unknown --lib
```

## High-Level Architecture

### Core Emulation Loop
The emulator follows a cycle-accurate design where the main components (CPU, PPU, APU) are synchronized:

1. **Main Loop** (`src/emulator.rs`):
   - `step_frame()` runs one complete frame (262 scanlines)
   - Each scanline executes CPU cycles, then PPU/APU cycles
   - DMA/HDMA transfers happen between CPU cycles

2. **Memory Bus** (`src/memory/bus.rs`):
   - Central hub for all memory access
   - Routes reads/writes to appropriate components (PPU, APU, cartridge, etc.)
   - Implements memory-mapped I/O registers
   - Contains direct-mapped cache for performance

### Component Communication
```
Emulator
├── CPU (65C816)
├── PPU (Picture Processing Unit)
├── APU (Audio Processing Unit)
├── DMA Controller
├── Memory Bus ←── All components access memory through this
├── Input (Controllers)
└── Cartridge (ROM + mapper)
```

### Key Design Patterns

1. **Static Decode Tables** (`src/cpu/decode_table.rs`):
   - CPU instructions are decoded using a static 256-entry lookup table
   - O(1) instruction decode instead of large match statements
   - Each entry contains addressing mode, operation, and cycle count

2. **PPU Rendering Pipeline** (`src/ppu/`):
   - Modular design with separate renderers for backgrounds, sprites, Mode 7
   - Tile caching system (`render_cache.rs`) to avoid repeated decoding
   - Priority-based pixel selection during scanline rendering

3. **Save States** (`src/savestate.rs`):
   - Uses serde for serialization
   - Compressed with gzip
   - Version checking for compatibility
   - Separate state structs for each component

### WebAssembly Integration
- Conditional compilation using `#[cfg(target_arch = "wasm32")]`
- WASM bindings in `src/wasm/mod.rs` expose a JavaScript-friendly API
- Native frontend code in `src/frontend/native/` is excluded from WASM builds
- Canvas rendering happens directly in the WASM module

### Performance Optimizations

1. **Memory Access Cache** (`src/memory/cache.rs`):
   - Direct-mapped cache for frequently accessed memory regions
   - Reduces bus routing overhead

2. **PPU Tile Cache** (`src/ppu/render_cache.rs`):
   - Pre-decoded tile data cached by tile address and palette
   - Invalidated on VRAM writes

3. **CPU Optimization**:
   - Addressing mode calculations inlined
   - Common operations (flags, memory access) use optimized bit manipulation

### Testing Strategy
- Unit tests for individual CPU instructions (`tests/cpu_tests.rs`)
- Integration tests for PPU rendering modes (`tests/ppu_tests.rs`)
- Save state round-trip tests (`tests/savestate_tests.rs`)
- Test ROM generation for accuracy testing (`tests/create_simple_test.py`)

### Configuration System
- TOML-based configuration in `~/.ccsnes/config.toml`
- Loaded at startup with defaults if missing
- Can be overridden via CLI arguments
- Separate sections for video, audio, input, emulation, and debug settings

### Critical File Relationships
- `src/emulator.rs` coordinates all components and timing
- `src/memory/bus.rs` handles all memory routing and must be updated when adding memory-mapped registers
- `src/cpu/decode_table.rs` and `src/cpu/instructions.rs` work together for CPU execution
- `src/ppu/core.rs` manages the rendering pipeline and calls into specific renderers

When implementing new features:
1. Memory-mapped I/O goes in `src/memory/bus.rs`
2. New CPU instructions need entries in `decode_table.rs`
3. PPU features may need updates to multiple renderer modules
4. Save state changes require updating `SaveState` struct and version number