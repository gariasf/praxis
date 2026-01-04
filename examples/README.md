# Praxis Examples

Runnable demos demonstrating Praxis engine features.

## Quick Start

```bash
# Build all examples
cargo build --examples

# Run a specific example
cargo run --example comprehensive_scene_demo
```

## Examples by Category

### Scene & Rendering Demos

| Example | Description |
|---------|-------------|
| `comprehensive_scene_demo` | Complete asset pipeline with OBJ loading, procedural textures, and FPS camera |
| `scene_demo` | Scene loading and saving with basic rendering |
| `multi_mesh_demo` | Multiple meshes with transforms |
| `material_demo` | Material system demonstration |
| `advanced_material_demo` | Material instancing, layers, parallax mapping, extended PBR |
| `environment_probe_demo` | Environment map reflections |
| `particles_demo` | GPU-accelerated particle system |
| `procedural_texture_demo` | Procedural texture generation with noise nodes and color ramps |
| `terrain_demo` | Heightmap terrain with LOD, texture splatting, and vegetation |

### Animation Demos

| Example | Description |
|---------|-------------|
| `skeletal_animation_demo` | Bone hierarchies and keyframe animation |
| `animation_demo` | Interactive animation with blend transitions |
| `animation_blending_demo` | Cross-fading, blend trees, layers |
| `animation_advanced_demo` | IK, retargeting, additive blending, root motion |
| `gltf_animation_loader_demo` | Loading animations from GLTF files |

### Audio Demos

| Example | Description |
|---------|-------------|
| `audio_demo` | Spatial audio, distance attenuation, doppler |
| `audio_simple` | Basic audio system setup demonstration |

### Editor Demos

| Example | Description |
|---------|-------------|
| `editor_demo` | Full editor with panels and tools |
| `editor_camera_demo` | Orbit camera controller |
| `selection_demo` | Entity selection, raycast picking |
| `gui_demo` | Basic egui integration |
| `console_demo` | Console panel with log filtering and search |
| `menu_bar_demo` | Menu bar placeholder (design spec) |

### Editor Systems

| Example | Description |
|---------|-------------|
| `command_system_demo` | Undo/redo command pattern |
| `command_serialization_demo` | Command history serialization |
| `undo_redo_system_demo` | Complete undo/redo system with history limits |
| `scene_serialization_demo` | Scene serialization with versioning and editor data |

### Input Demos

| Example | Description |
|---------|-------------|
| `input_integration` | Keyboard, mouse, gamepad input |
| `fps_camera_controller` | First-person camera controls |

### Core ECS & Transform Demos

| Example | Description |
|---------|-------------|
| `ecs_integration` | ECS basics with Praxis |
| `transform_propagation_demo` | Parent-child transform hierarchy |

### Optimization & Performance Demos

| Example | Description |
|---------|-------------|
| `lod_demo` | Level of Detail system with distance-based mesh switching |
| `spatial_partitioning_demo` | Octree and BVH spatial partitioning |
| `spatial_optimization_demo` | Frustum culling, octree, BVH queries, LOD system |
| `profiling_demo` | CPU/GPU profiling, memory tracking, Chrome trace export |
| `profiling_advanced_demo` | Advanced profiling with visualization data generation |

### Scripting Demos

| Example | Description |
|---------|-------------|
| `scripting_demo` | Lua scripting with hot-reload and sandboxing |
| `scripting_advanced_demo` | Scripting with ECS systems and performance profiling |

### Networking Demos

| Example | Description |
|---------|-------------|
| `networking_demo` | Client-server architecture, entity replication, interpolation |

## Featured Examples

### Comprehensive Scene Demo ★

The most complete example demonstrating the full asset pipeline:
- OBJ mesh loading
- Procedural texture generation
- ECS-based scene management
- FPS camera controller

```bash
cargo run --example comprehensive_scene_demo
```

**Controls:** WASD (move), Mouse (look), Shift (sprint), ESC (exit)

### Editor Demo

Full editor interface:
- Dockable panels (hierarchy, inspector, console)
- Entity selection and manipulation
- Transform gizmos
- Undo/redo

```bash
cargo run --example editor_demo
```

### Animation Demo

Interactive character animation:
- Skeletal animation with 10-bone humanoid
- Smooth cross-fade transitions
- 1D blend tree for speed-based blending
- Real-time parameter adjustment

```bash
cargo run --example animation_demo
```

**Controls:** 1-4 (switch animations), Arrow Up/Down (adjust blend), ESC (exit)

### Terrain Demo

Large-scale terrain rendering:
- Heightmap-based terrain (512x512)
- 4-level LOD system
- Texture splatting (grass, rock, snow)
- GPU-instanced vegetation

```bash
cargo run --example terrain_demo
```

## Documentation

For detailed explanations of the systems demonstrated:

- [Guides](../docs/guides/README.md) - How-to documentation
- [Concepts](../docs/concepts/README.md) - Theory and design
- [Reference](../docs/reference/README.md) - API documentation
