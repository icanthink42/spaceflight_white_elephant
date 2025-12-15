#!/bin/bash
set -e

# Set up Rust environment
export CARGO_HOME=/tmp/.cargo
export RUSTUP_HOME=/tmp/.rustup
export PATH="/tmp/.cargo/bin:$PATH"

# Install Rust
echo "Installing Rust..."
curl https://sh.rustup.rs -sSf | sh -s -- -y --no-modify-path

# Add WASM target
echo "Adding wasm32 target..."
rustup target add wasm32-unknown-unknown

# Install wasm-pack
echo "Installing wasm-pack..."
cargo install wasm-pack

# Build WASM
echo "Building WASM module..."
wasm-pack build --target web --out-dir www/pkg --release

echo "Build complete!"

