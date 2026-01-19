# Dependency Reference

Quick reference for all external dependencies used in the Praxis engine.

## Core Dependencies

### Rendering & Graphics
- **vulkano** (0.35.1) - Rust bindings for Vulkan API
- **vulkano-shaders** (0.35.0) - Shader compilation macros
- **bytemuck** (1.23.1) - Zero-cost type conversions for GPU buffers
- **image** (0.25) - Image loading and decoding
- **shaderc** (0.8) - GLSL to SPIR-V shader compilation

### ECS & Game Architecture  
- **bevy_ecs** (0.14) - Entity Component System framework
- **glam** (0.30.4) - SIMD-optimized 3D math library

### Window & Input
- **winit** (0.30.11) - Cross-platform window management
- **pollster** (0.4) - Block on async operations

### Serialization
- **serde** (1.0) - Serialization framework
- **ron** (0.8) - Rusty Object Notation format
- **bincode** (1.3) - Binary serialization
- **serde_json** (1.0) - JSON serialization

### Concurrency & Threading
- **parking_lot** (0.12) - Faster std::sync alternatives
- **crossbeam-channel** (0.5) - Multi-producer multi-consumer channels
- **rayon** (1.10) - Data parallelism
- **tokio** (1.40) - Async runtime
- **dashmap** (6.1) - Concurrent HashMap

### Physics & Simulation
- **rapier3d** (0.22) - 3D physics engine
- **noise** (0.9) - Procedural noise generation

### Audio
- **kira** (0.9) - Audio playback library

### Scripting
- **mlua** (0.9) - Lua scripting bindings

### GUI
- **egui** (0.29) - Immediate mode GUI
- **egui-winit** (0.29) - Winit integration for egui
- **egui_winit_vulkano** (0.28) - Vulkan renderer for egui
- **egui_dock** (0.14) - Dockable panels for egui

### Asset Loading
- **tobj** (4.0) - Wavefront OBJ loader
- **gltf** (1.4) - glTF 2.0 loader

### Utilities
- **tracing** (0.1) - Structured logging
- **tracing-subscriber** (0.3) - Log output formatting
- **color-eyre** (0.6) - Enhanced error reporting
- **notify** (6.1) - File system watching
- **chrono** (0.4) - Date and time handling
- **rfd** (0.15) - Native file dialogs
- **bitflags** (2.6) - Type-safe bit flags
- **seahash** (4.1) - Fast non-cryptographic hash
- **async-trait** (0.1) - Async trait support

## Dependency Usage by Crate

### High-Level Crates
- **praxis_core** - Only internal dependencies
- **praxis_editor** - egui, vulkano, winit, notify, image, serde, ron, rfd, chrono
- **praxis_window** - winit, pollster

### Rendering Subsystem
- **praxis_graphics** - vulkano, vulkano-shaders, bytemuck, winit, image, rand, parking_lot, crossbeam-channel, serde
- **praxis_procedural** - vulkano, vulkano-shaders, bytemuck, rand, seahash, shaderc
- **praxis_gui** - egui, egui-winit, egui_winit_vulkano, vulkano, winit, parking_lot, mlua (optional)
- **praxis_terrain** - vulkano, vulkano-shaders, bytemuck, image, noise, rayon, rand

### Game Systems
- **praxis_ecs** - bevy_ecs, serde, ron
- **praxis_scene** - bevy_ecs, serde, ron, chrono
- **praxis_physics** - rapier3d, bevy_ecs, bitflags, serde (optional), ron (optional)
- **praxis_audio** - kira, bevy_ecs, serde (optional), ron (optional)
- **praxis_input** - winit, bevy_ecs, serde (optional)
- **praxis_scripting** - mlua, notify, bevy_ecs, parking_lot, serde, serde_json

### Specialized Systems
- **praxis_spatial** - bevy_ecs, vulkano, vulkano-shaders, bytemuck
- **praxis_assets** - tobj, gltf, tokio, crossbeam-channel, async-trait
- **praxis_networking** - tokio, bincode, serde, crossbeam-channel, parking_lot, dashmap, color-eyre
- **praxis_profiling** - vulkano, bevy_ecs, serde, serde_json, parking_lot

### Foundation
- **praxis_math** - glam
- **praxis_utils** - tracing, tracing-subscriber, color-eyre

## Version Policy

- **Vulkan ecosystem**: Pinned to 0.35.x for API stability
- **bevy_ecs**: Pinned to 0.14 for feature parity
- **winit**: Pinned to 0.30.11 for raw-window-handle compatibility
- **Other dependencies**: Use latest compatible versions with semantic versioning

## Feature Flags

Several crates support optional features:

- **praxis_audio**: `serialization` (default: enabled)
- **praxis_physics**: `serialization` (default: enabled)
- **praxis_input**: `serialization` (default: disabled)
- **praxis_gui**: `scripting` (default: disabled)
- **praxis_editor**: `terrain` (default: disabled)

## Adding New Dependencies

When adding a new dependency:

1. Add it to the appropriate crate's Cargo.toml
2. Include a comment explaining its purpose and usage location
3. Update this reference document
4. Verify it's actually used (avoid speculative dependencies)
5. Consider if it could be replaced by existing dependencies

## Security Considerations

- **Vendored builds**: mlua uses vendored Lua for reproducible builds
- **Cryptography**: No cryptographic dependencies (seahash is non-crypto)
- **Network**: tokio and related networking crates for multiplayer
- **File system**: notify, rfd, and tokio::fs for file operations

## Build Times

Heavy dependencies (impact on build time):
- **High impact**: vulkano, tokio, mlua (vendored), rapier3d, egui
- **Medium impact**: bevy_ecs, winit, gltf
- **Low impact**: glam, serde, parking_lot, crossbeam-channel

## Platform Support

All dependencies support:
- ✅ Windows
- ✅ Linux  
- ✅ macOS

Some dependencies have platform-specific backends:
- **kira**: Uses CPAL for cross-platform audio
- **winit**: Native window management per platform
- **vulkano**: Requires Vulkan driver support

## License Compatibility

All dependencies use permissive licenses compatible with MIT:
- **MIT**: Most dependencies
- **Apache-2.0**: Some dependencies (dual-licensed)
- **BSD**: Some dependencies
- **Unlicense/CC0**: Some dependencies

No GPL or copyleft licenses in the dependency tree.
