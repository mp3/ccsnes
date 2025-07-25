# Changelog

All notable changes to CCSNES will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-01-25

### Added
- Initial release of CCSNES
- Complete 65C816 CPU emulation with all instructions and addressing modes
- Full PPU (Picture Processing Unit) implementation
  - All background modes (0-7) including Mode 7
  - Sprite rendering with priority system
  - Window masking and color math
  - High-resolution modes
- APU (Audio Processing Unit) with SPC700 CPU and DSP
- DMA and HDMA controllers for fast memory transfers
- Memory mapping support for LoROM and HiROM cartridges
- Save state functionality with compression
- Configuration system using TOML format
- Command-line interface with multiple commands
- Native frontend with hardware-accelerated rendering (wgpu)
- Audio playback using cpal
- WebAssembly support for browser-based emulation
- Comprehensive debugging tools
  - Breakpoint manager
  - Execution trace
  - Memory watches
  - Performance profiler
- Performance optimizations
  - Static instruction decode tables
  - Tile caching system
  - Memory access cache

### Known Issues
- Special enhancement chips (SA-1, SuperFX, etc.) not yet supported
- Some minor PPU timing edge cases
- WebAssembly audio may have latency on some browsers