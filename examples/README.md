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

### Rendering

| Example | Description |
|---------|-------------|
| `deferred_demo` | Deferred rendering with G-buffer visualization |
| `hdr_demo` | HDR rendering with tone mapping operators |
| `shadow_demo` | Cascaded shadow maps with PCF filtering |
| `multi_mesh_demo` | Multiple meshes with transforms |
| `skybox_demo` | Cubemap skybox rendering |

### Animation

| Example | Description |
|---------|-------------|
| `skeletal_animation_demo` | Bone hierarchies and keyframe animation |
| `animation_blending_demo` | Cross-fading, blend trees, layers |
| `gltf_animation_loader_demo` | Loading animations from GLTF files |

### Physics

| Example | Description |
|---------|-------------|
| `physics_demo` | Rapier3D integration, collisions, forces |

### Audio

| Example | Description |
|---------|-------------|
| `audio_demo` | Spatial audio, distance attenuation, doppler |

### Editor

| Example | Description |
|---------|-------------|
| `editor_demo` | Full editor with panels and tools |
| `selection_demo` | Entity selection, raycast picking |
| `editor_camera_demo` | Orbit camera controller |
| `command_system_demo` | Undo/redo command pattern |
| `command_serialization_demo` | Command history serialization |

### Core Systems

| Example | Description |
|---------|-------------|
| `ecs_integration` | ECS basics with Praxis |
| `transform_propagation_demo` | Parent-child transform hierarchy |
| `input_integration` | Keyboard, mouse, gamepad input |
| `fps_camera_controller` | First-person camera controls |

### Assets

| Example | Description |
|---------|-------------|
| `obj_loader_demo` | Loading OBJ mesh files |
| `comprehensive_scene_demo` | Complete asset pipeline |
| `scene_demo` | Scene loading and saving |
| `environment_probe_demo` | Environment map reflections |

### GUI

| Example | Description |
|---------|-------------|
| `gui_demo` | Basic egui integration |

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

### Deferred Rendering Demo

Shows the deferred rendering pipeline:
- G-buffer visualization (albedo, normals, depth)
- Multiple light sources
- Efficient many-lights rendering

```bash
cargo run --example deferred_demo
```

### Editor Demo

Full editor interface:
- Dockable panels (hierarchy, inspector, console)
- Entity selection and manipulation
- Transform gizmos
- Undo/redo

```bash
cargo run --example editor_demo
```

## Documentation

For detailed explanations of the systems demonstrated:

- [Guides](../docs/guides/README.md) - How-to documentation
- [Concepts](../docs/concepts/README.md) - Theory and design
- [Reference](../docs/reference/README.md) - API documentation
