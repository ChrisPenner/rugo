#!/bin/bash

echo "Building WebGPU Go Game..."

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    cargo install wasm-pack
fi

# Build the project
echo "Building Rust to WebAssembly..."
wasm-pack build --target web --out-dir pkg

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "You can now serve the project using a local web server."
    echo ""
    echo "For example:"
    echo "  python3 -m http.server 8000"
    echo "  # or"
    echo "  npx serve ."
    echo ""
    echo "Then visit http://localhost:8000 in your browser"
else
    echo "Build failed!"
    exit 1
fi
