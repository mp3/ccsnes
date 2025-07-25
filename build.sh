#!/bin/bash

# CCSNES Build Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[BUILD]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Rust is not installed. Please install it from https://rustup.rs/"
    exit 1
fi

# Build function
build_native() {
    print_status "Building native version..."
    
    # Build in release mode
    cargo build --release
    
    if [ $? -eq 0 ]; then
        print_status "Native build successful!"
        print_status "Binary located at: target/release/ccsnes"
    else
        print_error "Native build failed!"
        exit 1
    fi
}

build_wasm() {
    print_status "Building WebAssembly version..."
    
    # Check if wasm-pack is installed
    if ! command -v wasm-pack &> /dev/null; then
        print_warning "wasm-pack is not installed. Installing..."
        curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    fi
    
    # Add wasm32 target if not already added
    rustup target add wasm32-unknown-unknown
    
    # Build with wasm-pack
    wasm-pack build --target web --out-dir pkg
    
    if [ $? -eq 0 ]; then
        print_status "WebAssembly build successful!"
        print_status "Output in pkg/ directory"
        
        # Copy HTML file to pkg directory
        if [ -f "index.html" ]; then
            cp index.html pkg/
            print_status "Copied index.html to pkg/"
        fi
    else
        print_error "WebAssembly build failed!"
        exit 1
    fi
}

run_tests() {
    print_status "Running tests..."
    cargo test
    
    if [ $? -eq 0 ]; then
        print_status "All tests passed!"
    else
        print_error "Some tests failed!"
        exit 1
    fi
}

clean() {
    print_status "Cleaning build artifacts..."
    cargo clean
    rm -rf pkg/
    print_status "Clean complete!"
}

# Main script
case "${1:-all}" in
    native)
        build_native
        ;;
    wasm|web)
        build_wasm
        ;;
    test)
        run_tests
        ;;
    clean)
        clean
        ;;
    all)
        build_native
        build_wasm
        ;;
    *)
        echo "Usage: $0 {native|wasm|web|test|clean|all}"
        echo ""
        echo "Commands:"
        echo "  native - Build native executable"
        echo "  wasm   - Build WebAssembly version"
        echo "  web    - Same as wasm"
        echo "  test   - Run test suite"
        echo "  clean  - Clean build artifacts"
        echo "  all    - Build both native and wasm (default)"
        exit 1
        ;;
esac

print_status "Build complete!"