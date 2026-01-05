# Getting Started

This section helps you get Praxis up and running.

## Contents

- [Installation](installation.md) - Requirements and setup
- [Project Structure](project-structure.md) - Understanding the workspace layout
- [Core Features](core-features.md) - Default engine capabilities
- [Feature Flags](feature-flags.md) - Optional systems and how to enable them

## Quick Start

```bash
# Clone and build
git clone https://github.com/gariasf/praxis
cd praxis
cargo build

# Run an example
cargo run --example comprehensive_scene_demo

# Or build with optional features
cargo build --features editor
cargo run --features editor --example editor_demo
```

## Requirements

- **Rust**: Latest stable via [rustup](https://rustup.rs/)
- **Vulkan**: GPU and drivers with Vulkan support
- **Platform**: Windows, Linux, or macOS

## Understanding the Engine

Praxis is modular by design:

- **[Core Features](core-features.md)** are always available: rendering, ECS, physics, audio, animation, and more
- **[Feature Flags](feature-flags.md)** unlock optional systems: editor tools, Lua scripting, networking, and terrain

This keeps builds fast and focused on what you need.

## Next Steps

After installation:
1. Run a few [examples](../../examples/README.md) to see the engine in action
2. Read the [Beginner's Guide](../beginners-guide.md) for core concepts
3. Explore [Guides](../guides/README.md) for specific features
