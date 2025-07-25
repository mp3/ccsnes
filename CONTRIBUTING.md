# Contributing to CCSNES

Thank you for your interest in contributing to CCSNES! This document provides guidelines and information for contributors.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/ccsnes.git`
3. Create a new branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test`
6. Commit your changes using conventional commits
7. Push to your fork and submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70 or later
- For WebAssembly development: wasm-pack
- For audio development: System audio libraries (ALSA on Linux, CoreAudio on macOS)

### Building

```bash
# Native debug build
cargo build

# Native release build
cargo build --release

# WebAssembly build
wasm-pack build --target web

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## Code Style

- Follow Rust standard style guidelines
- Use `cargo fmt` before committing
- Use `cargo clippy` to catch common issues
- Add documentation comments for public APIs
- Keep functions focused and under 50 lines when possible

## Architecture Overview

### Core Components

- `cpu/`: 65C816 CPU emulation
  - `core.rs`: Main CPU loop
  - `instructions.rs`: Instruction definitions
  - `addressing.rs`: Addressing modes
  - `execute.rs`: Instruction execution
  
- `ppu/`: Picture Processing Unit
  - `core.rs`: Main PPU loop
  - `backgrounds.rs`: Background layer rendering
  - `sprites.rs`: Sprite rendering
  - `mode7.rs`: Mode 7 graphics
  
- `apu/`: Audio Processing Unit
  - `spc700.rs`: SPC700 CPU
  - `dsp.rs`: Digital Signal Processor
  
- `memory/`: Memory management
  - `bus.rs`: System bus
  - `mappers.rs`: Cartridge mappers

### Adding New Features

1. **CPU Instructions**: Add to `cpu/instructions.rs` and `cpu/execute.rs`
2. **PPU Features**: Modify appropriate files in `ppu/`
3. **Memory Mappers**: Add to `memory/mappers.rs`
4. **Save States**: Update `savestate.rs` structures

## Testing

### Unit Tests

Place unit tests in the same file as the code being tested:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature() {
        // Test implementation
    }
}
```

### Integration Tests

Place integration tests in the `tests/` directory.

### Test ROMs

- Use homebrew test ROMs for testing specific features
- Document expected behavior for each test
- Add test results to CI pipeline

## Debugging

### Debug Features

- Use `log` crate for debug output
- Enable with `RUST_LOG=debug cargo run`
- Use the built-in debugger for complex issues

### Performance Profiling

- Use `cargo flamegraph` for performance analysis
- Profile in release mode for accurate results
- Focus on hot paths identified by the profiler

## Commit Guidelines

We use conventional commits for clear history:

- `feat:` New features
- `fix:` Bug fixes
- `perf:` Performance improvements
- `docs:` Documentation changes
- `test:` Test additions or changes
- `refactor:` Code refactoring
- `style:` Code style changes
- `chore:` Maintenance tasks

Example:
```
feat(cpu): implement BRK instruction handling

- Add BRK opcode to instruction set
- Implement interrupt vector handling
- Add tests for BRK behavior
```

## Pull Request Process

1. Ensure all tests pass
2. Update documentation if needed
3. Add tests for new features
4. Keep PRs focused on a single feature/fix
5. Provide clear description of changes
6. Link related issues

## Areas for Contribution

### High Priority

- Accuracy improvements
- Performance optimizations
- Additional cartridge mapper support
- Audio accuracy improvements

### Medium Priority

- Save state enhancements
- Debugging tools
- Frontend improvements
- Documentation

### Low Priority

- Special chip support (SA-1, SuperFX)
- Peripheral support (Mouse, Super Scope)
- Network play features

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on constructive feedback
- Assume good intentions

## Getting Help

- Open an issue for bugs or feature requests
- Join discussions in pull requests
- Check existing issues before creating new ones

## License

By contributing to CCSNES, you agree that your contributions will be licensed under the MIT license.