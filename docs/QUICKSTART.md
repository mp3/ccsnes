# CCSNES Quick Start Guide

## Getting Started

### 1. Download or Build CCSNES

#### Option A: Build from source
```bash
git clone https://github.com/mp3/ccsnes.git
cd ccsnes
cargo build --release
```

The executable will be at `target/release/ccsnes`.

#### Option B: Download pre-built binary
Download the latest release from the [releases page](https://github.com/mp3/ccsnes/releases).

### 2. Run Your First Game

```bash
# Basic usage
ccsnes run your_game.sfc

# With custom scale
ccsnes run your_game.sfc --scale 3

# Fullscreen mode
ccsnes run your_game.sfc --fullscreen
```

### 3. Controls

| SNES Button | Keyboard |
|-------------|----------|
| D-Pad       | Arrow Keys |
| A           | Z |
| B           | X |
| X           | A |
| Y           | S |
| L           | Q |
| R           | W |
| Start       | Enter |
| Select      | Right Shift |

### 4. Save States

During gameplay:
- F5: Save state
- F7: Load state
- F1-F4: Select save slot

### 5. Configuration

CCSNES creates a config file at `~/.config/ccsnes/config.toml` on first run.

Common settings to adjust:

```toml
[video]
scale = 3              # Window scale (1-4)
vsync = true          # Enable V-Sync
fullscreen = false    # Start in fullscreen

[audio]
volume = 0.8          # Volume (0.0-1.0)
enabled = true        # Enable audio

[emulation]
speed = 1.0           # Emulation speed (1.0 = normal)
```

## Troubleshooting

### No Sound
- Check that audio is enabled in config
- Ensure system volume is not muted
- Try adjusting buffer_size in config

### Poor Performance
- Build with `--release` flag
- Reduce video scale
- Disable vsync
- Close other applications

### ROM Won't Load
- Ensure ROM file is not compressed (extract .zip files)
- Check that the ROM is a valid SNES ROM
- Try different ROM files

### Controls Not Working
- Check keyboard layout (QWERTY expected)
- Ensure game window has focus
- Check control mappings in config

## Advanced Features

### Debugging
```bash
# Show FPS counter
ccsnes run game.sfc --show-fps

# Enable debug logging
RUST_LOG=debug ccsnes run game.sfc
```

### Benchmarking
```bash
# Run performance benchmark
ccsnes bench game.sfc --frames 1000
```

### ROM Information
```bash
# Display ROM header info
ccsnes info game.sfc
```

## WebAssembly Usage

### Building for Web
```bash
wasm-pack build --target web
```

### Basic HTML Setup
```html
<!DOCTYPE html>
<html>
<head>
    <title>CCSNES Web</title>
</head>
<body>
    <canvas id="screen"></canvas>
    <input type="file" id="rom-input" accept=".sfc,.smc">
    
    <script type="module">
        import init, { WasmEmulator } from './pkg/ccsnes.js';
        
        async function run() {
            await init();
            
            const emulator = new WasmEmulator();
            const canvas = document.getElementById('screen');
            const ctx = canvas.getContext('2d');
            
            // Handle ROM loading
            document.getElementById('rom-input').onchange = async (e) => {
                const file = e.target.files[0];
                const romData = new Uint8Array(await file.arrayBuffer());
                emulator.load_rom(romData);
                
                // Start emulation loop
                function frame() {
                    emulator.step_frame();
                    
                    // Render frame
                    const imageData = ctx.createImageData(256, 224);
                    imageData.data.set(emulator.get_frame_buffer_rgba());
                    ctx.putImageData(imageData, 0, 0);
                    
                    requestAnimationFrame(frame);
                }
                frame();
            };
        }
        
        run();
    </script>
</body>
</html>
```

## Tips

1. **Save your SRAM**: Games with battery saves will automatically save to `~/.local/share/ccsnes/sram/`

2. **Custom configs per game**: Use `--config` to load game-specific configurations

3. **Speed up gameplay**: Hold Tab for fast-forward (configurable speed)

4. **Take screenshots**: Press F12 to save a screenshot

5. **Record gameplay**: Use external screen recording software for best results