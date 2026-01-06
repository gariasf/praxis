# Mesh System Documentation

The Praxis mesh system provides a complete solution for loading, managing, and rendering 3D geometry. It consists of components in the ECS, asset management in the graphics system, and GPU buffer management.

## Architecture Overview

The mesh system is split across two main crates:

### praxis_ecs Components

**`Mesh`**: A component that stores mesh data directly on an entity. Useful for procedurally generated or dynamic meshes.

```rust
use praxis_ecs::{Mesh, Transform, World};

let mut world = World::new();

let vertices = vec![
    [0.0, 1.0, 0.0],   // Top
    [-1.0, -1.0, 0.0], // Bottom-left
    [1.0, -1.0, 0.0],  // Bottom-right
];
let indices = vec![0, 1, 2];

world.spawn((
    Transform::default(),
    Mesh::new(vertices, indices),
));
```

**`MeshHandle`**: A component that references a mesh by ID in the graphics system's asset manager. This is the preferred approach for shared meshes.

```rust
use praxis_ecs::{MeshHandle, Transform, World};

let mut world = World::new();

world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    MeshHandle::new("cube"),
));
```

### praxis_graphics Mesh Management

**`MeshData`**: CPU-side mesh definition with vertices, indices, and optional attributes (colors, normals, UVs).

```rust
use praxis_graphics::MeshData;

// Simple mesh with positions and indices
let mesh = MeshData::new(positions, indices);

// Mesh with colors
let mesh = MeshData::with_colors(positions, colors, indices);
```

**`GpuMesh`**: GPU-side mesh containing Vulkan buffers. Created automatically when uploading `MeshData`.

**`MeshAssetManager`**: Central manager for loaded meshes, accessible via `RenderContext`.

```rust
// Load a mesh
render_context
    .mesh_manager_mut()
    .load_mesh("cube", colored_cube_mesh())?;

// Check if a mesh exists
if render_context.mesh_manager().contains_mesh("cube") {
    // Get mesh reference
    let mesh = render_context.mesh_manager().get_mesh("cube");
}
```

## Primitive Mesh Functions

Praxis provides several built-in primitive mesh generators:

- **`colored_cube_mesh()`**: Multi-colored cube (each vertex has a different color)
- **`solid_cube_mesh(color)`**: Single-color cube
- **`quad_mesh(size, color)`**: Flat quad/plane facing up
- **`pyramid_mesh(base_color, tip_color)`**: 4-sided pyramid

```rust
use praxis_graphics::{colored_cube_mesh, pyramid_mesh, quad_mesh, solid_cube_mesh};

// Create various primitives
let cube = colored_cube_mesh();
let red_cube = solid_cube_mesh([1.0, 0.0, 0.0]);
let ground = quad_mesh(10.0, [0.3, 0.3, 0.3]);
let pyramid = pyramid_mesh([0.8, 0.6, 0.2], [1.0, 0.0, 0.0]);
```

## Rendering Pipeline

### Using DrawCommands

The modern rendering approach uses `DrawCommand` to specify both the mesh and transform:

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

### Complete Rendering Flow

1. **Load Meshes** (setup phase):
```rust
render_context.mesh_manager_mut().load_mesh("cube", colored_cube_mesh())?;
render_context.mesh_manager_mut().load_mesh("pyramid", pyramid_mesh([0.8, 0.6, 0.2], [1.0, 0.0, 0.0]))?;
```

2. **Spawn Entities** (setup phase):
```rust
world.spawn((Transform::from_xyz(0.0, 0.0, 0.0), MeshHandle::new("cube")));
world.spawn((Transform::from_xyz(2.0, 0.0, 0.0), MeshHandle::new("pyramid")));
```

3. **Query and Build Commands** (per frame):
```rust
let mut draw_commands = Vec::new();

for (transform, mesh_handle) in world.query::<(&Transform, &MeshHandle)>().iter() {
    draw_commands.push(DrawCommand {
        mesh_id: mesh_handle.id.clone(),
        model: transform.compute_matrix(),
        texture_name: None,
        material_properties: None,
    });
}
```

4. **Render** (per frame):
```rust
let cmds = RenderCommands {
    view,
    proj,
    draw_commands: &draw_commands,
    lighting: None,
};

render_context.render(&cmds)?;
```

## GPU Buffer Management

Each mesh maintains its own vertex and index buffers on the GPU:

- **Vertex Buffer**: Contains `Vertex3D` data (position + color)
- **Index Buffer**: Contains u16 indices defining triangles
- **Per-Mesh Buffers**: Each loaded mesh has dedicated buffers, avoiding conflicts

### Memory Efficiency

The system uses:
- `PREFER_DEVICE` memory type filter for optimal GPU access
- `HOST_SEQUENTIAL_WRITE` for efficient CPU → GPU uploads
- Shared mesh instances (via `MeshHandle`) to avoid duplicate GPU allocations

## Best Practices

1. **Use MeshHandle for Shared Geometry**: If multiple entities use the same mesh, load it once and reference it via `MeshHandle`.

2. **Use Mesh Component for Dynamic Geometry**: For procedurally generated or frequently changing meshes, store data directly in the `Mesh` component.

3. **Batch by Mesh Type**: The renderer currently binds vertex/index buffers per mesh, so grouping similar objects can improve performance.

4. **Pre-load Common Meshes**: Load frequently used meshes during initialization to avoid frame hitches.

5. **Clean Up Unused Meshes**: Use `mesh_manager_mut().remove_mesh(id)` to free GPU memory when meshes are no longer needed.

## Examples

See the following examples for complete demonstrations:

- **`examples/mesh_demo.rs`**: Architecture overview and basic usage
- **`examples/multi_mesh_demo.rs`**: Complete rendering example with multiple mesh types

Run examples with:
```bash
cargo run --example multi_mesh_demo
```

## Implementation Details

### Components Added

#### praxis_ecs

**`MeshHandle`** - References a mesh asset by ID
- String-based identifier
- Implements `From<&str>` and `From<String>`
- Preferred for shared static meshes

**`Mesh`** - Stores mesh data directly on entities
- Vertex positions (required)
- Optional colors, normals, UVs
- Indices for triangle definitions
- Methods: `new()`, `with_colors()`, `set_colors()`, `set_normals()`, `set_uvs()`
- Query methods: `vertex_count()`, `index_count()`, `triangle_count()`

#### praxis_graphics

**`MeshData`** - CPU-side mesh definition
- Fields: `positions`, `colors`, `normals`, `uvs`, `indices`
- Methods: `new()`, `with_colors()`, `to_vertices()`, `upload()`

**`GpuMesh`** - GPU-side mesh with Vulkan buffers
- Fields: `vertex_buffer`, `index_buffer`, `index_count`, `vertex_count`
- Method: `new()` - Creates buffers from vertex/index data

**`MeshAssetManager`** - Central mesh asset cache
- HashMap-based storage
- Methods: `new()`, `load_mesh()`, `get_mesh()`, `contains_mesh()`, `remove_mesh()`, `mesh_count()`, `clear()`

### Data Flow

```
1. CPU Side (ECS)
   Entity { Transform, MeshHandle("cube") }
   Entity { Transform, Mesh { vertices, indices } }

2. Asset Management
   MeshData -> MeshAssetManager.load_mesh() -> GpuMesh
   
3. GPU Side
   GpuMesh { vertex_buffer, index_buffer }

4. Rendering
   Query(Transform, MeshHandle) -> DrawCommand -> render()
```

### Memory Management

- **Per-Mesh Buffers**: Each mesh has dedicated vertex and index buffers
- **Shared Meshes**: Multiple entities can reference the same mesh via MeshHandle
- **Memory Type**: PREFER_DEVICE | HOST_SEQUENTIAL_WRITE for optimal performance
- **Cleanup**: Meshes removed via `remove_mesh()` or `clear()`

### Rendering Pipeline Implementation

1. **Setup Phase**
   - Load meshes into MeshAssetManager
   - Spawn entities with Transform + MeshHandle

2. **Frame Loop**
   - Query entities: `Query<(&Transform, &MeshHandle)>`
   - Build DrawCommands with mesh_id and model matrix
   - Call `render_context.render(&cmds)`

3. **GPU Execution**
   - For each DrawCommand:
     - Lookup mesh by ID
     - Create uniform buffer with MVP matrices
     - Bind mesh-specific vertex/index buffers
     - Submit draw call

## Future Enhancements

Planned improvements to the mesh system:

- **Asset Loading**: Support for loading meshes from file formats (OBJ, glTF)
- **Normal/UV Support**: Full pipeline for textured meshes with lighting
- **Mesh Instancing**: Efficient rendering of many copies of the same mesh
- **Dynamic Mesh Updates**: API for updating mesh data on the GPU
- **Mesh LOD**: Level-of-detail system for complex meshes
