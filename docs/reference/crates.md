# Crates Reference

Overview of all workspace crates, their purposes, and dependencies.

## Core Crates

### praxis
Root crate that re-exports all subsystems. Use this as your dependency.

```toml
[dependencies]
praxis = { path = "." }
```

### praxis_core
Engine lifecycle and main loop coordination.

- **Entry point**: `praxis_core::run()`
- **Dependencies**: praxis_window, praxis_ecs, praxis_input, praxis_audio, praxis_utils

### praxis_utils
Shared utilities used across all crates.

- Logging via `tracing`
- Error handling via `color-eyre`
- Frame timing (`FrameTimer`)

## Graphics Crates

### praxis_graphics
Vulkan rendering via `vulkano`.

**Key types:**
- `RenderContext` - Main rendering interface
- `GpuMesh` / `MeshData` - Mesh handling
- `Texture` / `TextureManager` - Texture management
- `DeferredRenderer` - G-buffer rendering
- `ToneMapper` - HDR to LDR conversion

**Documentation:**
- [Rendering Guide](../guides/rendering.md)
- [HDR Guide](../guides/hdr-and-tonemapping.md)

### praxis_window
Window management via `winit`.

- Window creation and events
- Input event forwarding
- Resize handling

## Data Crates

### praxis_ecs
Entity-Component-System via `bevy_ecs`.

**Re-exports:**
- `Component`, `Entity`, `Bundle`
- `Query`, `Commands`
- `Res`, `ResMut`, `Resource`
- `Schedule`, `World`

**Documentation:**
- [ECS Concepts](../concepts/ecs-architecture.md)

### praxis_math
Math utilities via `glam`.

**Re-exports:**
- `Vec2`, `Vec3`, `Vec4`
- `Mat3`, `Mat4`
- `Quat`

### praxis_scene
Scene graph and spatial organization.

- `Transform` / `GlobalTransform`
- `Parent` / `Children` hierarchy
- `Skeleton` / `AnimationClip` / `AnimationPlayer`
- Scene serialization

### praxis_assets
Asset loading and management.

- OBJ loading via `tobj`
- GLTF loading via `gltf`
- Asset caching

## Input/Output Crates

### praxis_input
Input handling.

- Keyboard, mouse, gamepad (via `gilrs`)
- `InputState` resource

### praxis_audio
Audio via `kira`.

- `AudioManager` - Audio backend
- `AudioSource` - Spatial audio component
- `AudioListener` - Listener component

### praxis_gui
Debug GUI via `egui`.

- `egui-winit` integration
- `egui_vulkano` rendering

## Simulation Crates

### praxis_physics
Physics via `Rapier3D`.

- `PhysicsWorld` - Physics pipeline
- `RigidBody`, `Collider` components
- Collision events, spatial queries

## Tools

### praxis_editor
Editor tools.

- Selection system
- Undo/redo
- Transform gizmos
- Dockable panels

**Documentation:**
- [Editor Guide](../editor/README.md)

## Dependency Graph

```
praxis_core
├── praxis_window ─── praxis_graphics
│                         └── praxis_math
├── praxis_ecs
├── praxis_input
├── praxis_audio
└── praxis_utils

praxis_scene ─┬── praxis_ecs
              └── praxis_math

praxis_assets ─── praxis_math

praxis_physics ─┬── praxis_ecs
                └── praxis_math

praxis_editor ─┬── praxis_ecs
               ├── praxis_gui
               └── praxis_scene
```

## See Also

- [Project Structure](../getting-started/project-structure.md)
- [Architecture](../architecture.md)
