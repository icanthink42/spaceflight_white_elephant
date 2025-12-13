#!/bin/bash

# Spaceflight Elephant - Windows Build Script
# This script cross-compiles the application for Windows from Linux

set -e

echo "========================================="
echo "Building Spaceflight Elephant for Windows"
echo "========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if the Windows target is installed
echo -e "${YELLOW}Checking if Windows target is installed...${NC}"
if ! rustup target list | grep -q "x86_64-pc-windows-gnu (installed)"; then
    echo -e "${YELLOW}Installing Windows target...${NC}"
    rustup target add x86_64-pc-windows-gnu
else
    echo -e "${GREEN}Windows target already installed${NC}"
fi

# Check if mingw-w64 is installed
echo -e "${YELLOW}Checking for mingw-w64 toolchain...${NC}"
if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo -e "${YELLOW}mingw-w64 not found. Please install it with:${NC}"
    echo "  Arch Linux: sudo pacman -S mingw-w64-gcc"
    echo "  Ubuntu/Debian: sudo apt install mingw-w64"
    echo "  Fedora: sudo dnf install mingw64-gcc"
    exit 1
else
    echo -e "${GREEN}mingw-w64 toolchain found${NC}"
fi

# Build for Windows
echo ""
echo -e "${YELLOW}Building for Windows target...${NC}"
cargo build --release --target x86_64-pc-windows-gnu

# Create output directory
OUTPUT_DIR="./windows_build"
mkdir -p "$OUTPUT_DIR"

# Copy the executable
echo ""
echo -e "${YELLOW}Copying executable to $OUTPUT_DIR/${NC}"
cp target/x86_64-pc-windows-gnu/release/spaceflight_elephant.exe "$OUTPUT_DIR/"

echo ""
echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}Build completed successfully!${NC}"
echo -e "${GREEN}=========================================${NC}"
echo ""
echo "Your Windows executable is located at:"
echo "  $OUTPUT_DIR/spaceflight_elephant.exe"
echo ""
echo "You can now transfer this .exe file to a Windows machine and run it."

