#!/bin/bash

set -e

echo "🦀 Building CCSNES..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Please install it:"
    echo "curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Build for different targets
case "${1:-web}" in
    "native")
        echo "🏗️  Building native version..."
        cargo build --release
        echo "✅ Native build complete: target/release/ccsnes"
        ;;
    "web")
        echo "🌐 Building WebAssembly version..."
        wasm-pack build --target web --out-dir web/pkg --release
        echo "✅ WebAssembly build complete: web/pkg/"
        echo "🚀 To serve the web version:"
        echo "   cd web && python -m http.server 8000"
        echo "   Then open http://localhost:8000"
        ;;
    "all")
        echo "🏗️  Building both native and web versions..."
        
        # Native build
        echo "Building native..."
        cargo build --release --target x86_64-unknown-linux-gnu 2>/dev/null || cargo build --release
        
        # WebAssembly build
        echo "Building WebAssembly..."
        wasm-pack build --target web --out-dir web/pkg --release
        
        echo "✅ All builds complete!"
        ;;
    *)
        echo "Usage: $0 [native|web|all]"
        echo "  native - Build native executable"
        echo "  web    - Build WebAssembly for web (default)"
        echo "  all    - Build both versions"
        exit 1
        ;;
esac

echo "🎮 CCSNES build finished!"