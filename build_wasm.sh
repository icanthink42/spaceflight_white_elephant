#!/bin/bash

# Build script for WebAssembly

set -e

echo "Building WASM module..."

# Ensure cargo bin is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Build the WASM module
wasm-pack build --target web --out-dir www/pkg --release

echo "WASM build complete!"
echo "Output directory: www/pkg"
echo ""
echo "To test locally:"
echo "  cd www"
echo "  python3 -m http.server 8080"
echo "  # Or use any other local web server"
echo ""
echo "Then open http://localhost:8080 in your browser"

