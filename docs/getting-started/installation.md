# Installation

## Requirements

- **Rust**: 1.75+ (stable)
- **Vulkan SDK**: 1.3+ (for graphics)
- **CMake**: For building some dependencies

### Platform-Specific

**Windows:**
- Visual Studio Build Tools 2019+
- Vulkan SDK from LunarG

**Linux:**
- `libvulkan-dev`, `libxcb1-dev`, `libxkbcommon-dev`
- Mesa Vulkan drivers or proprietary

**macOS:**
- MoltenVK (Vulkan over Metal)

## Clone and Build

```bash
git clone https://github.com/your-repo/praxis.git
cd praxis
cargo build
```

## Verify Installation

```bash
# Run a simple example
cargo run --example ecs_integration

# Run tests
cargo test --workspace
```

## IDE Setup

**VS Code:**
- Install rust-analyzer extension
- Recommended: Enable format on save

**CLion/RustRover:**
- Built-in Rust support

## Next Steps

- [Project Structure](project-structure.md) - Understand the workspace layout
- [Beginners Guide](../beginners-guide.md) - Learn Praxis through examples
