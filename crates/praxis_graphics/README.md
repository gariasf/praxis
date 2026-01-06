# Praxis Graphics

Graphics system for the Praxis game engine, providing Vulkan-based rendering via vulkano.

## Features

- **Vulkan Rendering**: Modern graphics API with explicit control
- **Mesh Asset Management**: Load, store, and render multiple mesh types
- **Primitive Mesh Generation**: Built-in cube, pyramid, and quad meshes
- **Per-Mesh Buffers**: Dedicated vertex and index buffers for each mesh
- **Dynamic Uniform Buffers**: Efficient per-object uniform data with ring buffer
- **Transform System**: Model-view-projection matrix pipeline
- **Material System**: PBR materials with textures and properties
- **Lighting**: Point lights, directional lights, and ambient lighting
- **Texture Management**: Texture loading and binding
- **Deferred Rendering**: Optional deferred rendering pipeline

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

Use `DrawCommand` and `RenderCommands` to render different mesh types:

```rust
use praxis_graphics::{DrawCommand, RenderCommands};
use praxis_math::{Mat4, Vec3};

let draw_commands = vec![
    DrawCommand {
        mesh_id: "cube".to_string(),
        model: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        texture_name: None,
        material_properties: None,
    },
    DrawCommand {
        mesh_id: "pyramid".to_string(),
        model: Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0)),
        texture_name: None,
        material_properties: None,
    },
];

let cmds = RenderCommands {
    view,
    proj,
    draw_commands: &draw_commands,
    lighting: None,
};

render_context.render(&cmds)?;
```

## Dynamic Uniform Buffers

The graphics system uses dynamic uniform buffers with a ring buffer architecture for efficient per-object rendering.

### Architecture

Instead of creating a new uniform buffer and descriptor set for each object every frame, the system uses a single large buffer with dynamic offsets:

```
┌─────────────────────────────────────────┐
│         Dynamic Uniform Buffer          │
├─────────────────────────────────────────┤
│ Frame 0 │ Frame 1 │ Frame 2 │ Frame 0..│
│  Obj 0  │  Obj 0  │  Obj 0  │  Obj 0   │
│  Obj 1  │  Obj 1  │  Obj 1  │  Obj 1   │
│  Obj 2  │  Obj 2  │  Obj 2  │  Obj 2   │
│   ...   │   ...   │   ...   │   ...    │
└─────────────────────────────────────────┘
```

### Benefits

- **Single descriptor set** bound once per frame
- **Per-object data** accessed via dynamic offsets
- **Persistent mapped buffer** for efficient CPU writes
- **Ring buffer** prevents CPU-GPU stalls
- **Automatic alignment** handling for device requirements

### Configuration

Key constants (configurable in `RenderContext::new()`):

```rust
const FRAMES_IN_FLIGHT: usize = 3;        // Ring buffer size
const MAX_OBJECTS_PER_FRAME: usize = 1024; // Max drawable objects
```

Adjust based on your needs:
- More frames in flight = smoother pacing but more memory
- More max objects = can draw more but uses more memory

### Render Flow

1. Advance ring buffer to next frame
2. Update view/projection buffer once per frame
3. Write all model matrices to ring buffer
4. For each object:
   - Calculate dynamic offset
   - Bind descriptor set with offset
   - Draw

## Vertex Format

The current vertex format (`Vertex3D`) supports:
- Position (3D coordinates)
- Color (RGB)
- Normal (for lighting)
- UV coordinates (for textures)

## Architecture

The graphics system is organized into modules:

- **`device`**: Vulkan instance and device management
- **`vertex`**: Vertex data structures
- **`pipeline`**: Graphics pipeline configuration
- **`shaders`**: GLSL shader compilation
- **`mesh`**: Mesh data structures and asset management
- **`primitives`**: Built-in primitive mesh generators
- **`uniform_buffer`**: Dynamic uniform buffer management
- **`texture`**: Texture loading and management
- **`material`**: Material system with PBR properties
- **`lighting`**: Lighting system

## Examples

Run the graphics demos:

```bash
# Multiple mesh rendering
cargo run --example multi_mesh_demo

# Material demonstration (includes PBR and post-processing)
cargo run --example material_demo

# Advanced lighting
cargo run --example advanced_lighting_demo

# Environment probes
cargo run --example environment_probe_demo
```

## Dependencies

- `vulkano` 0.35.1: Vulkan bindings and abstractions
- `vulkano-shaders`: Shader compilation
- `praxis_utils`: Error handling, logging
- `praxis_math`: Matrix and vector math
- `praxis_ecs`: ECS integration for rendering

## Performance Characteristics

**Memory Usage:**
```
FRAMES_IN_FLIGHT × MAX_OBJECTS × aligned_sizeof(ModelUniforms) + sizeof(ViewProjection)
```

With default settings (3 frames, 1024 max objects): ~3 MB properly aligned

**CPU Overhead:**
- Old approach: N_objects × (buffer_allocation + descriptor_set_allocation)
- New approach: 1 × view_proj_write + 1 × bulk_model_write + N_objects × offset_calculation

The new approach eliminates allocation overhead entirely.

## Device Compatibility

The implementation automatically queries and uses the device's `minUniformBufferOffsetAlignment` limit, ensuring compatibility across different GPUs. Typical values:
- NVIDIA: 256 bytes
- AMD: 256 bytes
- Intel: 256 bytes
- Mobile: 16-64 bytes

## Testing

For headless testing without GPU initialization, use `MockRenderContext`:

```rust
#[cfg(test)]
use praxis_graphics::MockRenderContext;

#[test]
fn test_game_logic() {
    let mut ctx = MockRenderContext::new();
    
    // All rendering operations are no-ops
    ctx.load_mesh("player", mesh_data).unwrap();
    ctx.load_texture("player_tex", path).unwrap();
    ctx.render(&commands).unwrap();
    
    // Suitable for testing game logic without graphics hardware
    assert_eq!(ctx.mesh_count(), 1);
}
```

The mock provides the same API surface as `RenderContext` but with all operations as no-ops, allowing tests to run in CI environments without GPU access.

## See Also

- [Mesh System Documentation](../../docs/mesh-system.md)
- [Rendering Guide](../../docs/guides/rendering.md)
- [HDR and Tonemapping](../../docs/guides/hdr-and-tonemapping.md)
- [Material System](../../docs/guides/materials.md)
- [Multi-mesh Demo](../../examples/multi_mesh_demo.rs)
