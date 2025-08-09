# Rugo - Go Game 🎯

A browser-based implementation of the ancient board game Go (Weiqi/Baduk) using Rust compiled to WebAssembly. Optimized for desktop, tablet, and mobile devices.

🎮 **[Play Online](https://[YOUR-USERNAME].github.io/rugo/)** (GitHub Pages)

## Features

- **🎯 Full Go Gameplay**: Complete rule implementation with capture mechanics
- **📱 Mobile Optimized**: Responsive design for phones and tablets with touch support
- **⚡ Rust + WebAssembly**: Fast, safe, and efficient game logic
- **🎲 Multiple Board Sizes**: 9×9, 13×13, and 19×19 boards
- **🔄 Game History**: Undo/redo functionality with state persistence
- **⏯️ Pass Moves**: Full game state management including pass functionality
- **💾 URL State**: Game state saved in URL for easy sharing and resuming
- **🎨 Touch-Friendly UI**: Optimized buttons and interactions for mobile devices
- **🌐 Cross-platform**: Runs in any modern browser

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- Modern web browser (Chrome, Firefox, Safari, Edge)
- No additional setup required - runs in any browser with WebAssembly support

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

## GitHub Pages Deployment

This project is set up for automatic deployment to GitHub Pages using GitHub Actions.

### Setup Instructions:

1. **Fork or clone this repository to your GitHub account**

2. **Enable GitHub Pages**:
   - Go to your repository settings
   - Scroll to "Pages" section
   - Under "Source", select "GitHub Actions"

3. **Push to main branch**:
   ```bash
   git add .
   git commit -m "Initial commit"
   git push origin main
   ```

4. **Automatic deployment**:
   - The GitHub Action will automatically build and deploy
   - Your game will be available at `https://[YOUR-USERNAME].github.io/rugo/`
   - Build status can be seen in the "Actions" tab

### Manual Deployment:

If you prefer to deploy manually:

```bash
# Build the project
./build.sh

# The entire directory can be served as static files
# including index.html and the pkg/ folder
```

### Deployment Notes:

- The WebAssembly files are built automatically by GitHub Actions
- No pre-built binaries are committed to the repository
- The deployment includes all necessary files for the game to run
- HTTPS is required for WebAssembly to work properly (GitHub Pages provides this)

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

- ✅ **Canvas-based rendering** (optimized for all devices)
- ✅ **Complete Go rules** (stone placement, capture, suicide prevention)
- ✅ **Multiple board sizes** (9×9, 13×13, 19×19)
- ✅ **Game history** (undo/redo functionality)
- ✅ **Pass moves** (full turn management)
- ✅ **Mobile optimization** (responsive design, touch events)
- ✅ **State persistence** (URL-based game state saving)
- ✅ **Touch-friendly interface** (optimized for phones/tablets)
- ✅ **Stone rendering** (high-quality circular stones with proper styling)
- ✅ **Capture detection** (group capture with liberty calculation)
- ✅ **Score tracking** (capture count display)
- ✅ **Error handling** (invalid move detection and user feedback)

### Next Steps

1. **Ko Rule Implementation**: Prevent immediate recapture situations
2. **Game End Detection**: Detect when both players pass consecutively
3. **Territory Scoring**: Implement area counting for game end
4. **Visual Enhancements**: Add animations and visual feedback
5. **AI Opponent**: Basic computer player using simple heuristics
6. **Time Controls**: Add game timers and time management
7. **Game Analysis**: Move history viewer and game review features

## WebGPU Browser Support

To enable WebGPU in your browser:

- **Chrome**: Visit `chrome://flags/` and enable "Unsafe WebGPU"
- **Firefox**: Visit `about:config` and set `dom.webgpu.enabled` to `true`
- **Safari**: Use Safari Technology Preview

## Contributing

This is a learning project exploring WebGPU and Rust for game development. Contributions, suggestions, and improvements are welcome!

## License

MIT License - feel free to use this code for your own projects.
