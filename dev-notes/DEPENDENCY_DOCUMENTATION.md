# Dependency Documentation Changes

This document tracks the documentation improvements made to all Cargo.toml files.

## Overview

All 19 workspace crates have been updated with comprehensive inline documentation for their dependencies.

## Documentation Format

Each Cargo.toml now follows this structure:

```toml
[dependencies]
# Category header (e.g., "Core Vulkan rendering")
dependency_name = "version"                    # Brief purpose
another_dependency = "version"                 # Purpose with usage location

# Another category
grouped_dependency_1 = "version"              # Purpose
grouped_dependency_2 = "version"              # Purpose

# Internal dependencies (always grouped together)
praxis_crate = { path = "../praxis_crate", version = "0.1.0" }
```

## Changes by Crate

### praxis_graphics ✨ Major Changes

**Removed:**
- `pollster = "0.4.0"` - Unused dependency
- `raw-window-handle = "0.6.2"` - Indirect dependency via winit/vulkano

**Added Documentation:**
```toml
# Core Vulkan rendering
vulkano = "0.35.1"
vulkano-shaders = "0.35.0"

# Data conversion and hashing (descriptor set hashing in lib.rs)
bytemuck = { version = "1.23.1", features = ["derive"] }

# Window abstraction (needed for Surface creation in device.rs)
winit = { version = "0.30.11", features = ["rwh_05"] }

# Internal dependencies
praxis_utils = { path = "../praxis_utils", version = "0.1.0" }
praxis_math = { path = "../praxis_math", version = "0.1.0" }
praxis_procedural = { path = "../praxis_procedural", version = "0.1.0" }

# Image loading for textures (texture.rs)
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }

# Random number generation (particles.rs, ssao.rs)
rand = "0.8"

# Thread-safe synchronization primitives (mesh.rs streaming system)
parking_lot = "0.12"

# Multi-producer multi-consumer channels (mesh.rs async streaming)
crossbeam-channel = "0.5"

# Serialization (optimization_config.rs, adaptive_quality.rs)
serde = { version = "1.0", features = ["derive"] }
```

### praxis_assets

**Added Documentation:**
- 3D model format loaders section
- Async I/O explanation
- Usage file references

### praxis_audio

**Added Documentation:**
- Audio playback library note
- ECS integration note
- Optional serialization feature documentation

### praxis_core

**Added Documentation:**
- Noted all dependencies are internal workspace crates

### praxis_ecs

**Added Documentation:**
- ECS framework with serialization note
- Component serialization explanation

### praxis_editor

**Added Documentation:**
- Editor GUI framework section
- File system watching explanation
- Native file dialogs note
- All 15 dependencies documented with purposes

### praxis_gui

**Added Documentation:**
- Immediate mode GUI framework section
- Winit and Vulkan integration notes
- Optional scripting support documentation

### praxis_input

**Added Documentation:**
- Window and input event handling
- Optional serialization feature

### praxis_math

**Added Documentation:**
- SIMD-optimized 3D math library note

### praxis_networking

**Added Documentation:**
- Async networking runtime section
- Binary serialization explanation
- Concurrent data structures section
- All 10 dependencies categorized

### praxis_physics

**Added Documentation:**
- 3D physics engine note
- Collision layer bitflags explanation
- Optional serialization feature

### praxis_procedural

**Added Documentation:**
- Vulkan compute shaders explanation
- GPU buffer conversion notes
- Runtime shader compilation explanation
- All 6 dependencies documented

### praxis_profiling

**Added Documentation:**
- GPU profiling notes
- Chrome trace format explanation
- Thread-safe timing data storage

### praxis_scene

**Added Documentation:**
- Scene and animation serialization
- Save file timestamps explanation

### praxis_scripting

**Added Documentation:**
- Lua 5.4 scripting engine note
- File watching for hot-reload
- Thread-safe script state management
- All 7 dependencies documented

### praxis_spatial

**Added Documentation:**
- GPU compute shaders for culling and LOD
- GPU buffer data conversion notes

### praxis_terrain

**Added Documentation:**
- Terrain rendering shaders section
- Heightmap loading explanation
- Procedural generation notes
- Parallel mesh generation
- All 7 dependencies categorized

### praxis_utils

**Added Documentation:**
- Structured logging
- Enhanced error reporting notes

### praxis_window

**Added Documentation:**
- Window management and event loop
- Async operation blocking explanation

## Documentation Principles Applied

1. **Purpose Over Mechanics:** Explain *why* not just *what*
2. **Usage Context:** Reference specific files/modules where used
3. **Grouping:** Related dependencies are grouped together
4. **Consistency:** Similar dependencies across crates use similar descriptions
5. **Brevity:** Comments are concise but informative

## Examples of Good Documentation

### Clear Purpose with Context
```toml
# Thread-safe synchronization primitives (mesh.rs streaming system)
parking_lot = "0.12"
```

### Feature Explanation
```toml
# 3D model format loaders
tobj = "4.0"                                           # OBJ file format loader
gltf = { version = "1.4", features = ["names"] }      # glTF 2.0 format loader with name preservation
```

### Optional Dependencies
```toml
# Optional Lua scripting support for console (console_panel.rs)
mlua = { version = "0.9", optional = true }
```

## Before and After Comparison

### Before (praxis_graphics)
```toml
[dependencies]
vulkano = "0.35.1"
vulkano-shaders = "0.35.0"
bytemuck = { version = "1.23.1", features = ["derive"] }
pollster = "0.4.0"
raw-window-handle = "0.6.2"
winit = { version = "0.30.11", features = ["rwh_05"] }
# ... more dependencies
```

### After (praxis_graphics)
```toml
[dependencies]
# Core Vulkan rendering
vulkano = "0.35.1"
vulkano-shaders = "0.35.0"

# Data conversion and hashing (descriptor set hashing in lib.rs)
bytemuck = { version = "1.23.1", features = ["derive"] }

# Window abstraction (needed for Surface creation in device.rs)
winit = { version = "0.30.11", features = ["rwh_05"] }
# ... more dependencies with clear documentation
```

## Maintenance Guidelines

When adding new dependencies:

1. Add a comment explaining its purpose
2. Include the file/module where it's primarily used
3. Group it with related dependencies
4. Update the dependency reference documentation
5. Consider if an existing dependency could be used instead

## Related Documentation

- [DEPENDENCY_AUDIT.md](../DEPENDENCY_AUDIT.md) - Full audit report
- [docs/reference/dependencies.md](../docs/reference/dependencies.md) - Quick reference
- [DEPENDENCY_AUDIT_SUMMARY.md](../DEPENDENCY_AUDIT_SUMMARY.md) - Executive summary

## Review Checklist

When reviewing dependency changes:

- [ ] Purpose is clearly documented
- [ ] Usage location is specified where helpful
- [ ] Dependency is actually used in the code
- [ ] No redundant dependencies exist
- [ ] Version is consistent with workspace policy
- [ ] Optional dependencies are marked with features
- [ ] Related dependencies are grouped together
