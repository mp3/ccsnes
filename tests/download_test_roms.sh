#!/bin/bash
# Download public domain test ROMs for SNES emulator testing

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
TEST_ROMS_DIR="$SCRIPT_DIR/test_roms"

# Create test ROMs directory
mkdir -p "$TEST_ROMS_DIR"

echo "Downloading test ROMs..."

# Function to download with error handling
download_file() {
    local url="$1"
    local output="$2"
    local description="$3"
    
    echo "Downloading $description..."
    if curl -L -o "$output" "$url" 2>/dev/null; then
        echo "✓ Downloaded $description"
    else
        echo "✗ Failed to download $description"
        return 1
    fi
}

# Test ROM URLs (public domain / homebrew test ROMs)
# Note: These are example URLs - you should verify the actual URLs for test ROMs

# CPU Test ROMs
download_file \
    "https://github.com/PeterLemon/SNES/raw/master/CPUTest/CPU/CPUADC/CPUADC.sfc" \
    "$TEST_ROMS_DIR/cpu_adc_test.sfc" \
    "CPU ADC instruction test"

# PPU Test ROMs  
download_file \
    "https://github.com/PeterLemon/SNES/raw/master/PPU/BGMAP/8x8/BGMAP8x8.sfc" \
    "$TEST_ROMS_DIR/ppu_bgmap_test.sfc" \
    "PPU background map test"

# Mode 7 Test
download_file \
    "https://github.com/PeterLemon/SNES/raw/master/PPU/Mode7/Rotate/Mode7Rotate.sfc" \
    "$TEST_ROMS_DIR/mode7_test.sfc" \
    "Mode 7 rotation test"

# Hello World test
download_file \
    "https://github.com/PeterLemon/SNES/raw/master/HelloWorld/HelloWorld.sfc" \
    "$TEST_ROMS_DIR/hello_world.sfc" \
    "Hello World test"

echo ""
echo "Test ROMs downloaded to: $TEST_ROMS_DIR"
echo ""
echo "Note: These URLs are examples. Please verify and use actual test ROM URLs."
echo "Recommended test ROMs:"
echo "- SNES Test Program by Sour"
echo "- Super Mario World (for general compatibility)"
echo "- F-Zero (for Mode 7 testing)"
echo "- Super Metroid (for special effects)"
echo ""