# Crate README Index

Quick reference to all Praxis crate documentation.

## Core Systems

### Engine Foundation

**[praxis_core](../../crates/praxis_core/README.md)**  
Engine lifecycle and subsystem orchestration. Entry point for initializing the engine.

**[praxis_utils](../../crates/praxis_utils/README.md)**  
Logging, error handling, and timing utilities. Foundation for all other crates.

**[praxis_window](../../crates/praxis_window/README.md)**  
Cross-platform window management using winit. Handles window creation and events.

### ECS & Data

**[praxis_ecs](../../crates/praxis_ecs/README.md)**  
Entity-Component-System built on bevy_ecs. Transform hierarchy, cameras, and serialization.

**[praxis_math](../../crates/praxis_math/README.md)**  
SIMD-accelerated mathematics using glam. Vectors, matrices, quaternions, and geometric primitives.

**[praxis_scene](../../crates/praxis_scene/README.md)**  
Scene management, hierarchies, and skeletal animation. RON-based scene serialization.

## Graphics & Rendering

**[praxis_graphics](../../crates/praxis_graphics/README.md)**  
Vulkan-based rendering system. PBR materials, deferred rendering, post-processing, and GPU optimization.

**[praxis_procedural](../../crates/praxis_procedural/README.md)**  
GPU-accelerated procedural texture generation. Node-based texture graphs with runtime GLSL compilation.

**[praxis_terrain](../../crates/praxis_terrain/README.md)**  
Heightmap terrain with chunked LOD, texture splatting, and GPU-instanced vegetation.

## Optimization

**[praxis_spatial](../../crates/praxis_spatial/README.md)**  
Spatial optimization with octree, BVH, frustum culling, and LOD systems.

**[praxis_profiling](../../crates/praxis_profiling/README.md)**  
CPU/GPU profiling with memory tracking, bottleneck identification, and Chrome trace export.

## Asset Management

**[praxis_assets](../../crates/praxis_assets/README.md)**  
Asset loading for OBJ, GLTF meshes, textures, and animations. Async loading with progress tracking.

## Input & Interaction

**[praxis_input](../../crates/praxis_input/README.md)**  
Keyboard, mouse, and gamepad input with action mapping and rebindable controls.

**[praxis_gui](../../crates/praxis_gui/README.md)**  
Immediate mode GUI using egui. Debug panels, console, and editor integration.

**[praxis_editor](../../crates/praxis_editor/README.md)**  
Editor system with dockable panels, selection, undo/redo, and transform gizmos.

## Physics & Audio

**[praxis_physics](../../crates/praxis_physics/README.md)**  
Rapier3D physics integration. Rigid bodies, colliders, and ECS transform synchronization.

**[praxis_audio](../../crates/praxis_audio/README.md)**  
Audio playback with 3D spatial audio, distance attenuation, and doppler effect.

## Scripting & Networking

**[praxis_scripting](../../crates/praxis_scripting/README.md)**  
Lua 5.4 scripting with ECS access, hot-reload, sandboxing, and performance monitoring.

**[praxis_networking](../../crates/praxis_networking/README.md)**  
Client-server networking with entity replication, interpolation, and lag compensation.

---

## Quick Comparison

| Crate | Primary Use | API Status | Examples |
|-------|-------------|------------|----------|
| praxis_core | Engine startup | Stable | See others |
| praxis_utils | Error/logging | Stable | N/A |
| praxis_window | Window creation | Stable | See examples |
| praxis_ecs | Entity management | Stable | Many |
| praxis_math | Math operations | Stable | See others |
| praxis_scene | Scene graphs | Stable | 4 |
| praxis_graphics | Rendering | Evolving | 11+ |
| praxis_procedural | Proc textures | Evolving | 1 |
| praxis_terrain | Terrain rendering | Evolving | 1 |
| praxis_spatial | Optimization | Stable | 2 |
| praxis_profiling | Performance | Evolving | 2 |
| praxis_assets | Asset loading | Stable | 3 |
| praxis_input | Input handling | Stable | 1 |
| praxis_gui | GUI panels | Evolving | 2 |
| praxis_editor | Editor tools | Evolving | 4 |
| praxis_physics | Physics sim | Stable | See scene demos |
| praxis_audio | Audio playback | Stable | 2 |
| praxis_scripting | Lua scripts | Evolving | 3 |
| praxis_networking | Multiplayer | Evolving | 1 |

## Finding Documentation

### By Feature

**Animation:**
- [praxis_scene](../../crates/praxis_scene/README.md) - Skeletal animation, blending
- [praxis_graphics](../../crates/praxis_graphics/README.md) - GPU skeletal animation

**Materials:**
- [praxis_graphics](../../crates/praxis_graphics/README.md) - PBR materials, instancing
- [praxis_procedural](../../crates/praxis_procedural/README.md) - Procedural textures
- [praxis_terrain](../../crates/praxis_terrain/README.md) - Texture splatting

**Performance:**
- [praxis_spatial](../../crates/praxis_spatial/README.md) - Culling, LOD, spatial queries
- [praxis_graphics](../../crates/praxis_graphics/README.md) - GPU culling, mesh streaming
- [praxis_profiling](../../crates/praxis_profiling/README.md) - Performance analysis

**Multiplayer:**
- [praxis_networking](../../crates/praxis_networking/README.md) - Client-server, replication
- [praxis_scripting](../../crates/praxis_scripting/README.md) - Shared game logic

**Editor:**
- [praxis_editor](../../crates/praxis_editor/README.md) - Editor core
- [praxis_gui](../../crates/praxis_gui/README.md) - GUI panels
- [praxis_input](../../crates/praxis_input/README.md) - Input handling

### By Dependency

**Using bevy_ecs:**
- praxis_ecs (core)
- praxis_physics
- praxis_audio
- praxis_spatial
- praxis_scene
- praxis_terrain
- praxis_editor
- praxis_networking

**Using Vulkano:**
- praxis_graphics
- praxis_window
- praxis_procedural
- praxis_terrain
- praxis_spatial (occlusion)
- praxis_profiling (GPU)

**Using async/tokio:**
- praxis_assets
- praxis_networking

## See Also

- [Architecture Overview](../architecture.md) - How crates fit together
- [Crate Reference](./crates.md) - Detailed crate relationships
- [Beginner's Guide](../beginners-guide.md) - Learning path
- [Getting Started](../getting-started/README.md) - Installation and setup
- [Crate README Audit](../CRATE_README_AUDIT.md) - Audit results and maintenance
