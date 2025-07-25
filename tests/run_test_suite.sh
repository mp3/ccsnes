#!/bin/bash
# Run test suite for CCSNES emulator

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/.."
EMULATOR="$PROJECT_ROOT/target/release/ccsnes"
TEST_ROMS_DIR="$SCRIPT_DIR/test_roms"
RESULTS_DIR="$SCRIPT_DIR/test_results"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create results directory
mkdir -p "$RESULTS_DIR"

# Build emulator in release mode
echo "Building emulator..."
cd "$PROJECT_ROOT"
cargo build --release

# Check if emulator exists
if [ ! -f "$EMULATOR" ]; then
    echo -e "${RED}Error: Emulator not found at $EMULATOR${NC}"
    exit 1
fi

echo ""
echo "Running CCSNES Test Suite"
echo "========================="
echo ""

# Function to run a test ROM
run_test() {
    local rom_path="$1"
    local test_name="$2"
    local expected_result="$3"
    
    if [ ! -f "$rom_path" ]; then
        echo -e "${YELLOW}⚠ Skipping $test_name - ROM not found${NC}"
        return
    fi
    
    echo -n "Testing $test_name... "
    
    # Run emulator with test ROM for a short time
    if timeout 5s "$EMULATOR" test "$rom_path" > "$RESULTS_DIR/${test_name}.log" 2>&1; then
        echo -e "${GREEN}✓ PASSED${NC}"
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        echo "  See $RESULTS_DIR/${test_name}.log for details"
        return 1
    fi
}

# Function to benchmark a ROM
benchmark_rom() {
    local rom_path="$1"
    local rom_name="$2"
    
    if [ ! -f "$rom_path" ]; then
        return
    fi
    
    echo -n "Benchmarking $rom_name... "
    
    if "$EMULATOR" bench "$rom_path" --frames 1000 > "$RESULTS_DIR/${rom_name}_bench.log" 2>&1; then
        # Extract FPS from log
        fps=$(grep -oP "Average FPS: \K[0-9.]+" "$RESULTS_DIR/${rom_name}_bench.log" || echo "N/A")
        echo -e "${GREEN}$fps FPS${NC}"
    else
        echo -e "${RED}Failed${NC}"
    fi
}

# Function to check ROM info
check_rom_info() {
    local rom_path="$1"
    local rom_name="$2"
    
    if [ ! -f "$rom_path" ]; then
        return
    fi
    
    echo "ROM Info for $rom_name:"
    "$EMULATOR" info "$rom_path" | sed 's/^/  /'
    echo ""
}

# Run CPU instruction tests
echo "1. CPU Instruction Tests"
echo "------------------------"
run_test "$TEST_ROMS_DIR/cpu_adc_test.sfc" "cpu_adc" "pass"
run_test "$TEST_ROMS_DIR/cpu_test.sfc" "cpu_general" "pass"
echo ""

# Run PPU tests
echo "2. PPU Rendering Tests"
echo "----------------------"
run_test "$TEST_ROMS_DIR/ppu_bgmap_test.sfc" "ppu_bgmap" "pass"
run_test "$TEST_ROMS_DIR/mode7_test.sfc" "mode7" "pass"
run_test "$TEST_ROMS_DIR/sprite_test.sfc" "sprites" "pass"
echo ""

# Run timing tests
echo "3. Timing Tests"
echo "---------------"
run_test "$TEST_ROMS_DIR/timing_test.sfc" "timing" "pass"
echo ""

# Run audio tests
echo "4. Audio Tests"
echo "--------------"
run_test "$TEST_ROMS_DIR/spc_test.sfc" "spc700" "pass"
echo ""

# Test with real games (if available)
echo "5. Game Compatibility Tests"
echo "---------------------------"
if [ -d "$HOME/roms/snes" ]; then
    # Test some popular games if available
    run_test "$HOME/roms/snes/Super Mario World (USA).sfc" "mario_world" "playable"
    run_test "$HOME/roms/snes/The Legend of Zelda - A Link to the Past (USA).sfc" "zelda" "playable"
    run_test "$HOME/roms/snes/Super Metroid (Japan, USA).sfc" "metroid" "playable"
else
    echo -e "${YELLOW}No game ROMs found for compatibility testing${NC}"
fi
echo ""

# Performance benchmarks
echo "6. Performance Benchmarks"
echo "------------------------"
benchmark_rom "$TEST_ROMS_DIR/hello_world.sfc" "hello_world"
benchmark_rom "$HOME/roms/snes/Super Mario World (USA).sfc" "mario_world"
echo ""

# ROM information
echo "7. ROM Header Parsing"
echo "--------------------"
check_rom_info "$TEST_ROMS_DIR/hello_world.sfc" "hello_world"

# Summary
echo ""
echo "Test Suite Complete!"
echo "==================="
echo "Results saved to: $RESULTS_DIR"
echo ""

# Generate summary report
echo "Generating test report..."
cat > "$RESULTS_DIR/summary.txt" << EOF
CCSNES Test Suite Results
=========================
Date: $(date)
Emulator Version: $(cd "$PROJECT_ROOT" && git describe --tags --always)

Test Results:
$(ls -1 "$RESULTS_DIR"/*.log 2>/dev/null | wc -l) tests executed

Performance Summary:
$(grep -h "Average FPS" "$RESULTS_DIR"/*_bench.log 2>/dev/null || echo "No benchmark results")

EOF

echo "Summary report: $RESULTS_DIR/summary.txt"