#!/bin/bash
# Build CCSNES for WebAssembly

set -e

echo "Building CCSNES for WebAssembly..."

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WASM module
echo "Building WASM module..."
wasm-pack build --target web --out-dir web/pkg --features wasm

# Post-processing
echo "Post-processing build files..."
# Fix the import path in the generated JS file if needed
sed -i.bak 's/ccsnes_bg.wasm/ccsnes.wasm/g' web/pkg/ccsnes.js || true
rm -f web/pkg/ccsnes.js.bak

echo "Build complete! Files generated in web/pkg/"
echo ""
echo "To run the demo:"
echo "  1. Start a local web server in the 'web' directory"
echo "  2. Open http://localhost:8000 in your browser"
echo ""
echo "Example using Python:"
echo "  cd web && python3 -m http.server 8000"