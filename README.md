# Rugo - WebGPU Go Game

A browser-based implementation of the ancient board game Go (Weiqi/Baduk) using WebGPU and Rust compiled to WebAssembly.

## Features

- **WebGPU Rendering**: High-performance graphics using the modern WebGPU API
- **Rust + WebAssembly**: Fast, safe, and efficient game logic
- **Standard Go Rules**: 19x19 board with traditional Go gameplay
- **Interactive UI**: Click to place stones, visual feedback
- **Cross-platform**: Runs in any modern browser supporting WebGPU

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- Modern browser with WebGPU support:
  - Chrome/Chromium 113+ (with WebGPU enabled)
  - Firefox Nightly (with WebGPU enabled)
  - Safari Technology Preview 164+

## Building and Running

1. **Clone and enter the project directory**:
   ```bash
   cd rugo
   ```

2. **Build the project**:
   ```bash
   ./build.sh
   ```
   This will:
   - Install `wasm-pack` if not present
   - Compile Rust to WebAssembly
   - Generate JavaScript bindings

3. **Serve the project**:
   ```bash
   python3 -m http.server 8000
   ```
   Or if you have Node.js:
   ```bash
   npm run serve
   ```

4. **Open in browser**: Navigate to `http://localhost:8000`

## Development

### Project Structure

```
rugo/
├── src/
│   ├── lib.rs          # Main game logic
│   └── shader.wgsl     # WebGPU shaders
├── index.html          # Web interface
├── build.sh           # Build script
├── Cargo.toml         # Rust dependencies
└── README.md          # This file
```

### Current Implementation

- ✅ WebGPU initialization
- ✅ Basic board rendering (grid lines)
- ✅ Click handling infrastructure
- ✅ Game state management
- 🚧 Stone rendering (basic framework)
- ⏳ Go rules implementation
- ⏳ Capture detection
- ⏳ Scoring system

### Next Steps

1. **Stone Rendering**: Add circular stone geometry and rendering
2. **Game Rules**: Implement capture, ko rule, and illegal move detection
3. **Visual Polish**: Better graphics, animations, and UI
4. **Game Features**: Undo, save/load, time controls
5. **AI Opponent**: Basic computer player

## WebGPU Browser Support

To enable WebGPU in your browser:

- **Chrome**: Visit `chrome://flags/` and enable "Unsafe WebGPU"
- **Firefox**: Visit `about:config` and set `dom.webgpu.enabled` to `true`
- **Safari**: Use Safari Technology Preview

## Contributing

This is a learning project exploring WebGPU and Rust for game development. Contributions, suggestions, and improvements are welcome!

## License

MIT License - feel free to use this code for your own projects.
