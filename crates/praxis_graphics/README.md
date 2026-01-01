# Praxis Graphics

Graphics system for the Praxis game engine, providing Vulkan-based rendering via vulkano.

## Features

- **Vulkan Rendering**: Modern graphics API with explicit control
- **Mesh Asset Management**: Load, store, and render multiple mesh types
- **Primitive Mesh Generation**: Built-in cube, pyramid, and quad meshes
- **Per-Mesh Buffers**: Dedicated vertex and index buffers for each mesh
- **Transform System**: Model-view-projection matrix pipeline

## Mesh System

The mesh system provides complete support for loading and rendering 3D geometry.

### Core Types

- **`MeshData`**: CPU-side mesh definition with vertices, colors, normals, UVs, and indices
- **`GpuMesh`**: GPU-side mesh containing Vulkan buffers
- **`MeshAssetManager`**: Central manager for loaded meshes

### Loading Meshes

```rust
use praxis_graphics::{RenderContext, colored_cube_mesh};

// Access the mesh manager through RenderContext
render_context
    .mesh_manager_mut()
    .load_mesh("cube", colored_cube_mesh())?;
```

### Primitive Meshes

Built-in primitive mesh generators:

```rust
use praxis_graphics::{colored_cube_mesh, solid_cube_mesh, quad_mesh, pyramid_mesh};

// Multi-colored cube
let cube = colored_cube_mesh();

// Single-color cube
let red_cube = solid_cube_mesh([1.0, 0.0, 0.0]);

// Ground plane
let ground = quad_mesh(10.0, [0.3, 0.3, 0.3]);

// Pyramid with custom colors
let pyramid = pyramid_mesh([0.8, 0.6, 0.2], [1.0, 0.0, 0.0]);
```

### Rendering with Multiple Meshes

Use `DrawCommand` and `MeshRenderCommands` to render different mesh types:

```rust
use praxis_graphics::{DrawCommand, MeshRenderCommands};
use praxis_math::{Mat4, Vec3};

let draw_commands = vec![
    DrawCommand {
        mesh_id: "cube".to_string(),
        model: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    },
    DrawCommand {
        mesh_id: "pyramid".to_string(),
        model: Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0)),
    },
];

let cmds = MeshRenderCommands {
    view,
    proj,
    draw_commands: &draw_commands,
};

render_context.render_meshes(&cmds)?;
```

## Vertex Format

The current vertex format (`Vertex3D`) supports:
- Position (3D coordinates)
- Color (RGB)

Future versions will add support for normals, UVs, and other attributes.

## Architecture

The graphics system is organized into modules:

- **`device`**: Vulkan instance and device management
- **`vertex`**: Vertex data structures
- **`pipeline`**: Graphics pipeline configuration
- **`shaders`**: GLSL shader compilation
- **`mesh`**: Mesh data structures and asset management
- **`primitives`**: Built-in primitive mesh generators

## See Also

- [Mesh System Documentation](../../docs/mesh_system.md) - Complete mesh system guide
- [examples/multi_mesh_demo.rs](../../examples/multi_mesh_demo.rs) - Working example with multiple meshes
