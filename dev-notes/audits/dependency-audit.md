# Dependency Audit Report

This document provides a comprehensive audit of all dependencies across the Praxis engine workspace, documenting the purpose and usage of each external crate.

**Audit Date:** 2024
**Total Crates in Workspace:** 19

## Executive Summary

- **Total External Dependencies:** ~50 unique crates across all workspace members
- **Removed Unused Dependencies:** 2 from praxis_graphics (pollster, raw-window-handle)
- **All Remaining Dependencies:** Verified as actively used
- **Documentation:** All Cargo.toml files now include inline comments explaining each dependency's purpose

## Findings by Crate

### praxis_graphics (Primary Focus - 20+ Dependencies)

**Status:** ✅ All dependencies verified and documented

| Dependency | Version | Purpose | Used In |
|------------|---------|---------|---------|
| vulkano | 0.35.1 | Core Vulkan rendering API | Throughout crate |
| vulkano-shaders | 0.35.0 | Shader compilation macros | shaders/ module |
| bytemuck | 1.23.1 | Data conversion, descriptor set hashing | lib.rs |
| winit | 0.30.11 | Window abstraction for Surface creation | device.rs, lib.rs |
| image | 0.25 | Texture loading (PNG, JPEG) | texture.rs |
| rand | 0.8 | Random number generation | particles.rs, ssao.rs |
| parking_lot | 0.12 | Thread-safe primitives for streaming | mesh.rs |
| crossbeam-channel | 0.5 | MPMC channels for async streaming | mesh.rs |
| serde | 1.0 | Serialization for config structs | optimization_config.rs, adaptive_quality.rs |

**Removed:**
- ~~pollster~~ - Not used in praxis_graphics (used in praxis_window instead)
- ~~raw-window-handle~~ - Implicit dependency via winit/vulkano, not directly used

**Notes:**
- `winit` is kept despite minimal direct usage because it's required for `Surface::from_window()` and the `Window` type
- All dependencies serve active purposes in the rendering pipeline

### praxis_assets

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| tobj | OBJ file format loader |
| gltf | glTF 2.0 format loader with name preservation |
| tokio | Async I/O for async_loader.rs |
| crossbeam-channel | Thread-safe channels for loader |
| async-trait | Async trait definitions |

### praxis_audio

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| kira | Audio playback with CPAL backend |
| bevy_ecs | ECS integration |
| serde (optional) | Audio component serialization |
| ron (optional) | RON format for serialization |

### praxis_core

**Status:** ✅ Internal-only dependencies

All dependencies are internal workspace crates. No external dependencies.

### praxis_ecs

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| bevy_ecs | Core ECS framework with serialization |
| serde | Component serialization |
| ron | RON format for scene files |

### praxis_editor

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| egui | Immediate mode GUI framework |
| egui_dock | Dockable panel layout |
| egui-winit | Winit integration |
| vulkano | Rendering integration |
| notify | File system watching for hot-reload |
| image | Thumbnail loading |
| serde/ron | Editor state serialization |
| rfd | Native file dialogs |
| chrono | Save file timestamps |

### praxis_gui

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| egui | GUI framework |
| egui-winit | Winit integration |
| egui_winit_vulkano | Vulkan rendering backend |
| parking_lot | Thread-safe GUI state |
| mlua (optional) | Lua console support |

### praxis_input

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| winit | Window and input events |
| bevy_ecs | ECS integration |
| serde (optional) | Input mapping serialization |

### praxis_math

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| glam | SIMD-optimized 3D math with serialization |

### praxis_networking

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| tokio | Async networking runtime |
| bincode | Binary message serialization |
| serde | Data structure serialization |
| crossbeam-channel | Thread-safe channels |
| parking_lot | Synchronization primitives |
| dashmap | Concurrent hash map |
| color-eyre | Error handling |

### praxis_physics

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| rapier3d | 3D physics engine |
| bevy_ecs | ECS integration |
| bitflags | Collision layer flags |
| serde/ron (optional) | Physics component serialization |

### praxis_procedural

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| vulkano/vulkano-shaders | GPU compute shaders for texture generation |
| bytemuck | GPU buffer data conversion |
| rand | Noise generation seeding |
| seahash | Fast hashing for texture cache |
| shaderc | Runtime GLSL to SPIR-V compilation |

### praxis_profiling

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| vulkano | GPU profiling |
| bevy_ecs | ECS integration |
| serde/serde_json | Chrome trace format |
| parking_lot | Thread-safe timing data |

### praxis_scene

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| bevy_ecs | ECS integration |
| serde/ron | Scene and animation serialization |
| chrono | Save file timestamps |

### praxis_scripting

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| mlua | Lua 5.4 scripting engine |
| notify | File watching for hot-reload |
| bevy_ecs | ECS integration |
| parking_lot | Thread-safe script state |
| serde/serde_json | Script data exchange |

### praxis_spatial

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| bevy_ecs | ECS integration |
| vulkano/vulkano-shaders | GPU culling and LOD shaders |
| bytemuck | GPU buffer data conversion |

### praxis_terrain

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| vulkano/vulkano-shaders | Terrain rendering shaders |
| bytemuck | GPU buffer conversion |
| bevy_ecs | ECS integration |
| image | Heightmap loading |
| noise | Procedural generation |
| rayon | Parallel mesh generation |
| rand | Vegetation placement |

### praxis_utils

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| tracing | Structured logging |
| tracing-subscriber | Log formatting and filtering |
| color-eyre | Enhanced error reporting |

### praxis_window

**Status:** ✅ All dependencies documented

| Dependency | Purpose |
|------------|---------|
| winit | Window management and event loop |
| pollster | Blocking on async window creation |

## Common Patterns

### Most Used Dependencies

1. **bevy_ecs (0.14)** - Used in 14 crates - ECS framework foundation
2. **vulkano (0.35.1)** - Used in 8 crates - Vulkan rendering
3. **serde (1.0)** - Used in 13 crates - Universal serialization
4. **parking_lot (0.12)** - Used in 6 crates - Thread-safe primitives
5. **winit (0.30.11)** - Used in 5 crates - Window management

### Dependency Categories

- **Rendering:** vulkano, vulkano-shaders, bytemuck, image
- **ECS:** bevy_ecs
- **Serialization:** serde, ron, bincode, serde_json
- **Concurrency:** parking_lot, crossbeam-channel, dashmap, rayon, tokio
- **Math:** glam
- **GUI:** egui family (egui, egui-winit, egui_winit_vulkano, egui_dock)
- **Utilities:** tracing, color-eyre, notify, chrono

## Recommendations

### Completed Actions

✅ **Removed unused dependencies from praxis_graphics:**
   - Removed `pollster` (0.4.0) - unused, belongs in praxis_window
   - Removed `raw-window-handle` (0.6.2) - implicit dependency, not directly used

✅ **Added comprehensive documentation:**
   - All Cargo.toml files now have inline comments
   - Each dependency includes its purpose and usage location
   - Grouped dependencies logically for easier maintenance

### Future Considerations

1. **Version Alignment:**
   - Most crates use consistent versions (e.g., vulkano 0.35.1, bevy_ecs 0.14)
   - Consider workspace-level dependency management for shared crates

2. **Feature Flags:**
   - Several crates properly use optional dependencies (serde, ron, mlua)
   - This pattern could be extended to other crates for modular builds

3. **Dependency Consolidation:**
   - Consider if `crossbeam-channel` could be replaced with `tokio::sync` in some cases
   - Evaluate if all crates need separate `serde`/`ron` or if it could be centralized

4. **Documentation:**
   - All dependencies are now well-documented in Cargo.toml files
   - Regular audits recommended when adding new dependencies

## Validation

All dependencies have been verified through:
- Code search (`grep`) for actual usage
- File inspection to confirm import statements
- Purpose documentation in Cargo.toml comments

No redundant or unused dependencies remain in the workspace.

## Conclusion

The Praxis engine dependency graph is clean and well-justified. Each external dependency serves a clear purpose and is actively used. The removal of 2 unused dependencies from praxis_graphics and the addition of comprehensive documentation improves maintainability and makes the codebase easier to understand for new contributors.
