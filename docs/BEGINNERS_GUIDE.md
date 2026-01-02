# Beginner's Guide to Praxis Engine

This guide provides comprehensive explanations of the core concepts and architectural patterns used throughout the Praxis game engine. It's designed for developers new to game engines, Vulkan/graphics programming, or Rust game development.

## Table of Contents

1. [Rendering Pipeline Flow](#rendering-pipeline-flow)
2. [Uniform Buffers and Descriptor Sets](#uniform-buffers-and-descriptor-sets)
3. [ECS Data Flow](#ecs-data-flow)
4. [Memory Management Patterns](#memory-management-patterns)
5. [Vulkan/Vulkano Abstractions](#vulkanvulkano-abstractions)
6. [Dynamic Uniform Buffer Ring System](#dynamic-uniform-buffer-ring-system)
7. [Transform Hierarchy Propagation](#transform-hierarchy-propagation)
8. [Rust-Specific Patterns](#rust-specific-patterns)

---

## Rendering Pipeline Flow

The rendering pipeline in Praxis follows a predictable flow from application initialization to frame presentation.

### High-Level Flow

```text
┌─────────────────────────────────────────────────────────────┐
│                    Application Start                         │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  praxis_utils::init()                                        │
│  - Setup logging (tracing)                                   │
│  - Initialize error reporting (color-eyre)                   │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  praxis_ecs::init()                                          │
│  - Initialize ECS system (bevy_ecs)                          │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  praxis_input::init()                                        │
│  - Initialize input handling                                 │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  praxis_window::run()                                        │
│  - Create winit event loop                                   │
│  - Create window (1920x1080)                                 │
│  - Enter event loop (ControlFlow::Poll)                      │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  ApplicationHandler::resumed()                               │
│  - Async: RenderContext::new()                               │
│    └─► VulkanDevice::new()                                   │
│    └─► Create swapchain                                      │
│    └─► Create render pass                                    │
│    └─► Create graphics pipeline                              │
│    └─► Create framebuffers                                   │
│    └─► Initialize mesh/texture managers                      │
│    └─► Create default white texture                          │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
                ┌───────────────────────┐
                │   Main Event Loop     │
                │   (ControlFlow::Poll) │
                └───────────┬───────────┘
                            │
                            ▼
        ┌───────────────────────────────────┐
        │     WindowEvent::RedrawRequested  │
        └───────────────────────────────────┘
                            │
                            ▼
        ┌───────────────────────────────────────────────┐
        │          Per-Frame Rendering Flow             │
        │                                               │
        │  1. Check if window is minimized              │
        │     └─► If yes: skip rendering                │
        │                                               │
        │  2. Handle swapchain recreation               │
        │     └─► If resize pending: recreate           │
        │                                               │
        │  3. Acquire next swapchain image              │
        │     └─► Get image index + future              │
        │                                               │
        │  4. Build per-object data                     │
        │     └─► Create uniform buffers                │
        │     └─► Create descriptor sets                │
        │                                               │
        │  5. Record command buffer                     │
        │     └─► Begin render pass                     │
        │     └─► Bind pipeline                         │
        │     └─► For each object:                      │
        │         ├─► Bind vertex/index buffers         │
        │         ├─► Bind descriptor set               │
        │         └─► Draw indexed                      │
        │     └─► End render pass                       │
        │                                               │
        │  6. Submit to GPU                             │
        │     └─► Join with acquire future              │
        │     └─► Execute command buffer                │
        │                                               │
        │  7. Present image                             │
        │     └─► Swapchain present                     │
        │     └─► Signal fence and flush                │
        │                                               │
        │  8. Store future for next frame               │
        │     └─► Save for synchronization              │
        └───────────────────┬───────────────────────────┘
                            │
                            ▼
                  (Loop back to event loop)
```

### Detailed Command Buffer Recording

When recording commands for a single frame, the flow looks like this:

```text
GPU Command Recording
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│ AutoCommandBufferBuilder::primary()                         │
│ - Create command buffer for this frame                      │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ begin_render_pass()                                          │
│ - Specify framebuffer (swapchain image)                     │
│ - Set clear values (background color)                       │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ bind_pipeline_graphics()                                     │
│ - Bind the graphics pipeline (shaders + state)              │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ set_viewport()                                               │
│ - Define viewport transformation                            │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
        ┌───────────────────────────────────┐
        │    For each object to render:     │
        └───────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ bind_vertex_buffers()                                        │
│ - Bind mesh vertex buffer to binding 0                      │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ bind_index_buffer()                                          │
│ - Bind mesh index buffer                                    │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ bind_descriptor_sets()                                       │
│ - Bind uniform buffers (model, view, proj matrices)         │
│ - Bind texture sampler                                       │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ draw_indexed()                                               │
│ - Issue draw call for this object                           │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        └──────┐
                               │ (Repeat for next object)
                               │
                        ┌──────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ end_render_pass()                                            │
│ - Finish recording render commands                          │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ build()                                                      │
│ - Finalize command buffer                                   │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
            (Submit to GPU queue)
```

### Frame Synchronization

Vulkan requires careful synchronization to prevent race conditions:

```text
Frame Synchronization
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

CPU Timeline:    Frame N-1    │    Frame N      │    Frame N+1
                              │                 │
GPU Timeline:    Frame N-2    │    Frame N-1    │    Frame N
                              │                 │
                              │                 │
              ┌───────────────┼─────────────────┼──────────────┐
              │  Swapchain    │   Acquire Image │              │
              │  Images       │   ◄─────────────┤              │
              │               │                 │              │
              │  ┌─────┐      │   ┌─────┐       │   ┌─────┐   │
              │  │ Img │      │   │ Img │       │   │ Img │   │
              │  │  0  │◄─────┼───┤  1  │◄──────┼───┤  2  │   │
              │  └─────┘      │   └─────┘       │   └─────┘   │
              │    Presenting │     Rendering   │   Available  │
              └───────────────┼─────────────────┼──────────────┘
                              │                 │
                              │                 │
              ┌───────────────┼─────────────────┼──────────────┐
              │  GpuFuture    │  .join()        │  .then_*()   │
              │  Chain        │                 │              │
              │               │  previous_frame │              │
              │               │       +         │              │
              │               │  acquire_future │              │
              │               │       ▼         │              │
              │               │  then_execute() │              │
              │               │       ▼         │              │
              │               │  then_present() │              │
              │               │       ▼         │              │
              │               │  then_flush()   │              │
              └───────────────┼─────────────────┼──────────────┘
```

---

## Uniform Buffers and Descriptor Sets

Uniform buffers are the primary way to pass data from the CPU to GPU shaders. Understanding them is crucial for graphics programming.

### What is a Uniform Buffer?

A uniform buffer is a GPU-accessible memory buffer containing read-only data for shaders. The data is "uniform" because it doesn't change per-vertex or per-fragment—it's constant for an entire draw call.

```text
Uniform Buffer Concept
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

CPU Side                         GPU Side
┌──────────────────────┐       ┌──────────────────────┐
│   Application        │       │   Vertex Shader      │
│   ────────────       │       │   ─────────────      │
│                      │       │                      │
│   model: Mat4        │       │   layout(set=0,      │
│   view: Mat4         │◄──────┼───binding=0)         │
│   proj: Mat4         │       │   uniform Uniforms { │
│                      │       │     mat4 model;      │
│   Uniforms struct    │       │     mat4 view;       │
│                      │       │     mat4 proj;       │
└──────────────────────┘       │   };                 │
         │                     │                      │
         │ Write to            └──────────────────────┘
         ▼ buffer                       ▲
┌──────────────────────┐                │
│   Vulkan Buffer      │                │ Read from
│   (GPU Memory)       │────────────────┘ buffer
│                      │
│   [model matrix]     │ 64 bytes (16 floats)
│   [view matrix]      │ 64 bytes
│   [proj matrix]      │ 64 bytes
└──────────────────────┘
```

### Descriptor Sets: The Bridge

Descriptor sets are Vulkan's way of connecting buffers (and other resources) to shader bindings.

```text
Descriptor Set Architecture
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│                   Descriptor Set Layout                      │
│  (Defined at pipeline creation - describes what to expect)  │
│                                                              │
│   Set 0:                                                     │
│     Binding 0: Uniform Buffer (Uniforms struct)              │
│     Binding 1: Combined Image Sampler (Texture)              │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          │ Used to create ▼
                          │
┌─────────────────────────────────────────────────────────────┐
│                     Descriptor Set                           │
│  (Actual instance - binds specific resources)               │
│                                                              │
│   Set 0:                                                     │
│     Binding 0: ───► [Uniform Buffer at 0x12345678]           │
│                     (Contains model/view/proj matrices)      │
│     Binding 1: ───► [Texture + Sampler at 0xABCDEF00]       │
│                     (Wall texture with linear filtering)     │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          │ Bound during
                          │ draw call
                          ▼
                   ┌──────────────┐
                   │    Shader    │
                   │   Execution  │
                   └──────────────┘
```

### Current Praxis Implementation

In the current implementation, Praxis creates one descriptor set per object per frame:

```rust
// From RenderContext::render_meshes()
for draw_cmd in cmds.draw_commands.iter() {
    // 1. Create uniform buffer with model/view/proj matrices
    let uniforms = Uniforms {
        model: draw_cmd.model.to_cols_array_2d(),
        view: cmds.view.to_cols_array_2d(),
        proj: cmds.proj.to_cols_array_2d(),
    };

    let buffer = Buffer::from_data(
        self.memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        uniforms,
    )?;

    // 2. Create descriptor set binding the buffer
    let set = DescriptorSet::new(
        self.descriptor_set_allocator.clone(),
        self.descriptor_set_layout.clone(),
        [
            WriteDescriptorSet::buffer(0, buffer.clone()),
            WriteDescriptorSet::image_view_sampler(
                1,
                texture.view.clone(),
                texture.sampler.clone(),
            ),
        ],
        [],
    )?;
}
```

**Note**: This approach is simple but not optimal. See the [Dynamic Uniform Buffer Ring System](#dynamic-uniform-buffer-ring-system) section for the more efficient approach.

### Data Layout: std140

GLSL shaders expect uniform data in a specific layout. Praxis uses **std140**, which has strict alignment rules:

```text
std140 Layout Rules
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Type          Size      Alignment    Example
────────────────────────────────────────────────────────────
float         4 bytes   4 bytes      offset 0, 4, 8, 12...
vec2          8 bytes   8 bytes      offset 0, 8, 16, 24...
vec3         12 bytes  16 bytes      offset 0, 16, 32...
vec4         16 bytes  16 bytes      offset 0, 16, 32...
mat4         64 bytes  16 bytes      offset 0, 64, 128...

Array rule: Each element aligned to vec4 (16 bytes)

Struct example:
layout(std140) uniform Uniforms {
    mat4 model;   // offset 0,  size 64 bytes
    mat4 view;    // offset 64, size 64 bytes
    mat4 proj;    // offset 128, size 64 bytes
};
// Total size: 192 bytes (aligned to 16-byte boundary)
```

In Rust, we ensure correct layout using `#[repr(C)]`:

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    model: [[f32; 4]; 4],  // Column-major 4x4 matrix
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}
```

---

## ECS Data Flow

The Entity-Component-System (ECS) architecture is central to Praxis. Understanding how data flows through the ECS helps you write efficient systems.

### ECS Core Concepts

```text
ECS Architecture
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│                         WORLD                                │
│  (Container for all entities and components)                │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Entity Storage                           │  │
│  │  Entity 0, Entity 1, Entity 2, ...                    │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Component Storage                        │  │
│  │  (Organized by component type, not by entity)        │  │
│  │                                                       │  │
│  │  Transform:  [E0: (0,0,0)] [E1: (1,2,3)] ...        │  │
│  │  MeshHandle: [E0: "cube"]  [E2: "sphere"] ...       │  │
│  │  Camera:     [E3: {...}]                             │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘

                            ▲
                            │ Query
                            │
┌─────────────────────────────────────────────────────────────┐
│                        SYSTEMS                               │
│  (Functions that operate on components)                     │
│                                                              │
│  fn render_system(                                           │
│      query: Query<(&Transform, &MeshHandle)>                │
│  ) {                                                         │
│      for (transform, mesh) in query.iter() {                │
│          // Only iterates entities with BOTH components     │
│          draw(transform, mesh);                              │
│      }                                                       │
│  }                                                           │
└─────────────────────────────────────────────────────────────┘
```

### Component Storage

Components in bevy_ecs are stored in **archetypes** for cache efficiency:

```text
Archetype Storage
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Archetype: (Transform, MeshHandle, GlobalTransform)
┌─────────────────────────────────────────────────────────────┐
│  Entity IDs:     [E0] [E1] [E2] [E3] ...                    │
│  Transform:      [T0] [T1] [T2] [T3] ...                    │
│  MeshHandle:     [M0] [M1] [M2] [M3] ...                    │
│  GlobalTransform:[G0] [G1] [G2] [G3] ...                    │
└─────────────────────────────────────────────────────────────┘
All data for this archetype is stored contiguously in memory.
Iterating over entities of this archetype is cache-friendly!


Archetype: (Transform, Camera, PerspectiveProjection)
┌─────────────────────────────────────────────────────────────┐
│  Entity IDs:            [E10] [E11] ...                      │
│  Transform:             [T10] [T11] ...                      │
│  Camera:                [C10] [C11] ...                      │
│  PerspectiveProjection: [P10] [P11] ...                      │
└─────────────────────────────────────────────────────────────┘
```

### Query Flow

When you query for components, the ECS efficiently iterates only matching archetypes:

```text
Query Execution
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Query: Query<(&Transform, &MeshHandle)>
  │
  ├─► Scan all archetypes
  │   
  ├─► Does archetype contain Transform? ────► No ─► Skip
  │                    │
  │                    ▼ Yes
  │   
  └─► Does archetype contain MeshHandle? ───► No ─► Skip
                       │
                       ▼ Yes
  
      Match! Iterate this archetype:
      ┌────────────────────────────────────┐
      │ for (transform, mesh) in arch {    │
      │     // Process each entity         │
      │ }                                  │
      └────────────────────────────────────┘

Result: Only entities with BOTH components are processed.
        No heap allocations, no lookups, just linear iteration!
```

### Rendering Data Flow Example

Here's how data flows from ECS components to the GPU:

```text
ECS to GPU Data Flow
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. Game Logic Updates ECS
   ┌──────────────────────────────────┐
   │ World contains:                  │
   │   Entity 0: Transform, MeshHandle│
   │   Entity 1: Transform, MeshHandle│
   │   Entity 2: Camera, Transform    │
   └──────────┬───────────────────────┘
              │
              ▼
2. System Queries Components
   ┌──────────────────────────────────┐
   │ fn rendering_system(             │
   │     objects: Query<(             │
   │         &Transform,              │
   │         &GlobalTransform,        │
   │         &MeshHandle              │
   │     )>,                          │
   │     camera: Query<(              │
   │         &Camera,                 │
   │         &CameraMatrices          │
   │     )>                           │
   │ )                                │
   └──────────┬───────────────────────┘
              │
              ▼
3. Build Draw Commands
   ┌──────────────────────────────────┐
   │ let draw_cmds: Vec<DrawCommand>  │
   │ for (transform, .., mesh) in .. {│
   │     draw_cmds.push(DrawCommand { │
   │         mesh_id: mesh.id,        │
   │         model: global_transform  │
   │     });                          │
   │ }                                │
   └──────────┬───────────────────────┘
              │
              ▼
4. Submit to Renderer
   ┌──────────────────────────────────┐
   │ render_context.render_meshes(    │
   │     &MeshRenderCommands {        │
   │         view: camera_view,       │
   │         proj: camera_proj,       │
   │         draw_commands: &draw_cmds│
   │     }                            │
   │ )                                │
   └──────────┬───────────────────────┘
              │
              ▼
5. GPU Rendering
   ┌──────────────────────────────────┐
   │ For each draw command:           │
   │   - Create uniform buffer        │
   │   - Bind mesh buffers            │
   │   - Bind descriptor set          │
   │   - Draw indexed                 │
   └──────────────────────────────────┘
```

### Component Derive Macro

Praxis uses `bevy_ecs`'s `Component` derive macro to mark structs as components:

```rust
use bevy_ecs::component::Component;

#[derive(Component)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
```

**What the macro does**:
- Implements the `Component` trait
- Registers the type with the ECS
- Enables storage optimization based on type size
- Allows the component to be queried

---

## Memory Management Patterns

Praxis uses several memory management patterns to ensure safety and performance.

### Vulkan Memory Types

Vulkan exposes different types of memory with different properties:

```text
Vulkan Memory Types
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│  DEVICE_LOCAL                                                │
│  - Located in GPU VRAM                                       │
│  - Fastest for GPU access                                    │
│  - CPU cannot directly access                                │
│  - Used for: Vertex buffers, index buffers, textures        │
│                                                              │
│  ┌───────────────────────────────────────┐                  │
│  │ GPU  ◄── Very Fast ──► VRAM           │                  │
│  │                                       │                  │
│  │ CPU  ◄── No Access ──X VRAM           │                  │
│  └───────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  HOST_VISIBLE + HOST_COHERENT                                │
│  - CPU can map and write directly                            │
│  - Slower for GPU access                                     │
│  - Used for: Uniform buffers (updated every frame)          │
│                                                              │
│  ┌───────────────────────────────────────┐                  │
│  │ CPU  ◄── Fast ──► RAM                 │                  │
│  │                    │                   │                  │
│  │ GPU  ◄── Slower ───┘                   │                  │
│  └───────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  PREFER_DEVICE + HOST_SEQUENTIAL_WRITE                       │
│  - Resizable BAR / Smart Access Memory                       │
│  - Best of both worlds (when available)                      │
│  - Fast GPU access + CPU mappable                            │
│                                                              │
│  ┌───────────────────────────────────────┐                  │
│  │ CPU  ◄── Fast ──► VRAM (via BAR)      │                  │
│  │                                       │                  │
│  │ GPU  ◄── Fast ──► VRAM                │                  │
│  └───────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

### Buffer Creation Pattern

Praxis uses Vulkano's `Buffer` API with careful memory type selection:

```rust
// Example: Creating a vertex buffer
let vertex_buffer = Buffer::from_iter(
    memory_allocator.clone(),
    BufferCreateInfo {
        usage: BufferUsage::VERTEX_BUFFER,  // How will this be used?
        ..Default::default()
    },
    AllocationCreateInfo {
        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,  // Where to allocate?
        ..Default::default()
    },
    vertices.iter().copied(),  // Data to upload
)?;
```

**Rationale**:
- `PREFER_DEVICE`: Try to get GPU-local memory (fastest)
- `HOST_SEQUENTIAL_WRITE`: But allow CPU writes (for convenience)
- Vulkano picks the best available memory type

### Arc and Reference Counting

Praxis extensively uses `Arc` (Atomic Reference Counting) for shared ownership:

```text
Arc Reference Counting
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Problem: Multiple parts of the code need access to the same Vulkan resource.
         Who owns it? When should it be dropped?

Solution: Arc<T> - Atomic Reference Counted Smart Pointer

┌─────────────────────────────────────────────────────────────┐
│  Arc<Device>  (ref count: 1)                                 │
│  ┌────────┐                                                  │
│  │ Device │◄───── RenderContext owns it                      │
│  └────────┘                                                  │
└─────────────────────────────────────────────────────────────┘

           When we clone Arc<Device>:
           
┌─────────────────────────────────────────────────────────────┐
│  Arc<Device>  (ref count: 3)                                 │
│  ┌────────┐                                                  │
│  │ Device │◄───── RenderContext                              │
│  │        │◄───── Queue                                      │
│  │        │◄───── MemoryAllocator                            │
│  └────────┘                                                  │
└─────────────────────────────────────────────────────────────┘

When all owners drop their Arc, ref count → 0, Device is destroyed.
```

**Why Arc is used in Praxis**:
- Vulkan resources have dependencies (e.g., Buffer depends on Device)
- Arc allows sharing without lifetime complications
- Thread-safe (Vulkano can use resources across threads)

Example from `RenderContext`:
```rust
pub struct RenderContext {
    pub device: Arc<Device>,              // Shared with many objects
    pub graphics_queue: Arc<Queue>,       // Shares device
    memory_allocator: Arc<StandardMemoryAllocator>,  // Shares device
    // ... many more Arc fields
}
```

### Subbuffer: Slices of Buffers

`Subbuffer<T>` is Vulkano's way of representing a typed view into a buffer:

```text
Subbuffer Concept
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│                    Underlying Buffer                         │
│                  (Untyped GPU memory)                        │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ [bytes bytes bytes bytes bytes bytes bytes ......... ]│   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                          │
              ┌───────────┴───────────┐
              │                       │
              ▼                       ▼
┌──────────────────────────┐ ┌──────────────────────────┐
│ Subbuffer<[Vertex3D]>    │ │ Subbuffer<[u16]>         │
│                          │ │                          │
│ - Typed view             │ │ - Typed view             │
│ - Offset: 0              │ │ - Offset: 1024           │
│ - Length: 256 vertices   │ │ - Length: 512 indices    │
└──────────────────────────┘ └──────────────────────────┘
```

Benefits:
- Type safety: Can't accidentally use a vertex buffer as an index buffer
- Automatic size tracking
- Efficient: No copies, just metadata

---

## Vulkan/Vulkano Abstractions

Praxis uses `vulkano`, a safe Rust wrapper around Vulkan. Understanding the abstraction layers helps debug issues.

### Vulkan Abstraction Hierarchy

```text
Abstraction Layers
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│  Application Code (Praxis)                                   │
│  - RenderContext::render()                                   │
│  - High-level rendering APIs                                 │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│  Vulkano (Safe Rust Wrapper)                                 │
│  - Device, Queue, Buffer, Image, etc.                        │
│  - Builder patterns                                          │
│  - Automatic synchronization                                 │
│  - Compile-time shader validation                            │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│  Vulkan API (C API)                                          │
│  - vkCreateDevice, vkCreateBuffer, etc.                      │
│  - Raw handles (VkDevice, VkBuffer, etc.)                    │
│  - Manual memory management                                  │
│  - Explicit synchronization                                  │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│  GPU Driver                                                   │
│  - Translates Vulkan commands to GPU instructions            │
│  - Manages hardware resources                                │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│  GPU Hardware                                                 │
│  - Executes shaders                                          │
│  - Performs rasterization                                    │
└─────────────────────────────────────────────────────────────┘
```

### Key Vulkano Abstractions

#### Device and Queue

```rust
// Physical device = actual GPU hardware
let physical_device: Arc<PhysicalDevice> = ...;

// Logical device = our interface to the GPU
let device: Arc<Device> = Device::new(...)?;

// Queue = command submission pathway
let graphics_queue: Arc<Queue> = ...;
```

```text
Device and Queue Relationship
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│                    Physical Device                           │
│                  (Actual GPU hardware)                       │
│                                                              │
│  Properties:                                                 │
│    - Name: "NVIDIA GeForce RTX 4070"                         │
│    - Memory: 12 GB VRAM                                      │
│    - Queue families: Graphics, Compute, Transfer            │
└────────────────────┬────────────────────────────────────────┘
                     │ Create
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                    Logical Device                            │
│         (Application's interface to GPU)                     │
│                                                              │
│  Owns:                                                       │
│    - Memory allocator                                        │
│    - Command pools                                           │
│    - Descriptor pools                                        │
└────────────────────┬────────────────────────────────────────┘
                     │ Has
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                       Queues                                 │
│        (Submission pathways to GPU)                          │
│                                                              │
│  Graphics Queue:  Submit rendering commands                  │
│  Compute Queue:   Submit compute shaders                     │
│  Transfer Queue:  Submit memory transfer operations          │
└─────────────────────────────────────────────────────────────┘
```

#### Pipeline: The Complete Rendering Configuration

A graphics pipeline in Vulkan defines the entire rendering state:

```text
Graphics Pipeline State
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

GraphicsPipeline contains:

┌─────────────────────────────────────────────────────────────┐
│  1. Shaders                                                  │
│     - Vertex shader (vertex.glsl → SPIR-V)                   │
│     - Fragment shader (fragment.glsl → SPIR-V)               │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  2. Vertex Input State                                       │
│     - Vertex buffer bindings                                 │
│     - Attribute locations (position, color, normal, uv)      │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  3. Input Assembly State                                     │
│     - Topology: TriangleList, LineList, etc.                 │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  4. Viewport State                                           │
│     - Viewport dimensions                                    │
│     - Scissor rectangle                                      │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  5. Rasterization State                                      │
│     - Polygon mode: Fill, Line, Point                        │
│     - Cull mode: Front, Back, None                           │
│     - Front face: Clockwise, CounterClockwise                │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  6. Multisample State                                        │
│     - Sample count (for MSAA)                                │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  7. Depth/Stencil State                                      │
│     - Depth test enable                                      │
│     - Depth write enable                                     │
│     - Depth compare operation                                │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  8. Color Blend State                                        │
│     - Blend enable                                           │
│     - Blend factors                                          │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  9. Dynamic State                                            │
│     - Which states can change without recreating pipeline    │
│     - Example: Viewport, Scissor                             │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  10. Render Pass Compatibility                               │
│      - Which render pass this pipeline works with            │
└─────────────────────────────────────────────────────────────┘
```

Creating a pipeline is expensive, so Praxis creates them once during initialization.

---

## Dynamic Uniform Buffer Ring System

The `uniform_buffer.rs` module implements an efficient ring buffer system for per-frame uniform data. This is a significant optimization over per-object buffer creation.

### The Problem

The naive approach (current in Praxis) creates a new buffer for each object each frame:

```text
Naive Approach (Current Implementation)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Frame N:
  Object 0: Allocate buffer → Write uniforms → Create descriptor set
  Object 1: Allocate buffer → Write uniforms → Create descriptor set
  Object 2: Allocate buffer → Write uniforms → Create descriptor set
  ...
  Object 99: Allocate buffer → Write uniforms → Create descriptor set

  = 100 allocations per frame
  = 100 descriptor sets per frame
  = 6000 allocations per second at 60 FPS!
```

**Problems**:
- Many small allocations (fragmentation)
- Descriptor set allocation overhead
- CPU time spent on allocation/deallocation

### The Solution: Ring Buffer

The dynamic uniform buffer pre-allocates a large buffer divided into frames:

```text
Ring Buffer Architecture
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│              Dynamic Uniform Buffer                          │
│         (Single large persistent buffer)                     │
│                                                              │
│  ┌───────────────┬───────────────┬───────────────┐          │
│  │   Frame 0     │   Frame 1     │   Frame 2     │          │
│  │ (256 objects) │ (256 objects) │ (256 objects) │          │
│  ├───────────────┼───────────────┼───────────────┤          │
│  │               │               │               │          │
│  │  Obj 0: [M]   │  Obj 0: [M]   │  Obj 0: [M]   │          │
│  │  (aligned)    │  (aligned)    │  (aligned)    │          │
│  │               │               │               │          │
│  │  Obj 1: [M]   │  Obj 1: [M]   │  Obj 1: [M]   │          │
│  │  (aligned)    │  (aligned)    │  (aligned)    │          │
│  │               │               │               │          │
│  │  Obj 2: [M]   │  Obj 2: [M]   │  Obj 2: [M]   │          │
│  │  (aligned)    │  (aligned)    │  (aligned)    │          │
│  │               │               │               │          │
│  │     ...       │     ...       │     ...       │          │
│  │               │               │               │          │
│  └───────────────┴───────────────┴───────────────┘          │
│         ▲                                    │               │
│         │                                    │               │
│    GPU reading                          CPU writing          │
│    Frame N-2                            Frame N              │
└─────────────────────────────────────────────────────────────┘

Timeline:
─────────────────────────────────────────────────────────────
CPU Frame:  N-1      │      N       │      N+1      │
GPU Frame:  N-2      │      N-1     │      N        │
                     │              │               │
            No sync  │   No sync    │   No sync     │
            needed!  │   needed!    │   needed!     │
```

**Key insight**: By having 3 frames worth of buffer space, the CPU can write Frame N data while the GPU is still reading Frame N-2 data. No synchronization needed!

### Alignment Requirements

GPUs require uniform buffers to be aligned to specific boundaries:

```text
Uniform Buffer Alignment
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Minimum alignment (device-dependent, typically 256 bytes):

┌─────────────────────────────────────────────────────────────┐
│  Object 0 data     │  Padding │  Object 1 data     │ ...    │
│  (64 bytes)        │ (192 B)  │  (64 bytes)        │        │
│ [model matrix]     │ [unused] │ [model matrix]     │        │
└────────────────────┴──────────┴────────────────────┴────────┘
  ▲                                ▲
  │                                │
Offset 0                      Offset 256
(aligned)                     (aligned)

Why alignment matters:
- GPU hardware requirement for performance
- Allows efficient parallel access
- Violating alignment = undefined behavior!
```

The `DynamicUniformBuffer` automatically handles alignment:

```rust
impl DynamicUniformBuffer {
    fn align_up(size: usize, alignment: usize) -> usize {
        (size + alignment - 1) & !(alignment - 1)
    }

    pub fn new(..., max_objects_per_frame: usize) -> Result<Self> {
        let min_alignment = device
            .physical_device()
            .properties()
            .min_uniform_buffer_offset_alignment as usize;

        let object_size = std::mem::size_of::<ModelUniforms>();
        let aligned_object_size = Self::align_up(object_size, min_alignment);
        
        // Each object gets `aligned_object_size` bytes
        // ...
    }
}
```

### Usage Pattern

Using the dynamic uniform buffer is straightforward:

```rust
// Initialize once
let mut dyn_ubo = DynamicUniformBuffer::new(
    &device,
    memory_allocator,
    3,    // 3 frames in flight
    256,  // Max 256 objects per frame
)?;

// Each frame:
dyn_ubo.next_frame();  // Advance to next frame slot

let models: Vec<Mat4> = ...;  // Collect model matrices
dyn_ubo.write_models(&models)?;  // Write to buffer

// When drawing:
for i in 0..models.len() {
    let offset = dyn_ubo.get_dynamic_offset(i);
    // Use offset with VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC
}
```

---

## Transform Hierarchy Propagation

Praxis implements a scene graph where transforms can be hierarchical (parent-child relationships). Understanding how transforms propagate is crucial for working with the ECS.

### Transform Components

```rust
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

pub struct GlobalTransform {
    pub matrix: Mat4,
}

pub struct Parent(pub Entity);

pub struct Children(pub Vec<Entity>);
```

### Hierarchy Concept

```text
Transform Hierarchy
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Example: Character with weapon

┌──────────────────────────────────────────┐
│  Character Entity                        │
│  Transform:                              │
│    translation: (0, 0, 0)                │
│    rotation: 0° around Y                 │
│  GlobalTransform:                        │
│    matrix: I (identity)                  │
│  Children: [Hand Entity]                 │
└──────────────┬───────────────────────────┘
               │
               │ Parent-Child relationship
               │
               ▼
┌──────────────────────────────────────────┐
│  Hand Entity                             │
│  Transform:                              │
│    translation: (0.5, 0.8, 0)            │  ← Local offset
│    rotation: 0°                          │
│  Parent: Character Entity                │
│  GlobalTransform:                        │
│    matrix: Parent * Local                │
│           = (0,0,0) + (0.5,0.8,0)        │
│           = (0.5, 0.8, 0)                │
│  Children: [Weapon Entity]               │
└──────────────┬───────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────┐
│  Weapon Entity                           │
│  Transform:                              │
│    translation: (0, 0, 0.3)              │  ← Local offset
│    rotation: 0°                          │
│  Parent: Hand Entity                     │
│  GlobalTransform:                        │
│    matrix: Parent * Local                │
│           = (0.5,0.8,0) + (0,0,0.3)      │
│           = (0.5, 0.8, 0.3)              │
└──────────────────────────────────────────┘

Now if Character rotates 90° around Y axis:

Character GlobalTransform: rotation matrix
  │
  ├─► Hand GlobalTransform: Character * Hand local
  │     = (0,0,0) + rotated(0.5, 0.8, 0)
  │     ≈ (-0.5, 0.8, 0)  # Hand moved with character!
  │
  └─► Weapon GlobalTransform: Hand * Weapon local
        = Hand.matrix * Weapon.local
        ≈ (-0.5, 0.8, 0.3)  # Weapon also moved!
```

### Propagation Systems

Praxis uses multiple systems to maintain the hierarchy:

```text
Transform Propagation Pipeline
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│  1. sync_parent_child_relationships                          │
│                                                              │
│  When: Entity gets a Parent component                       │
│  Does: Add entity to parent's Children list                 │
│                                                              │
│  Query<(Entity, &Parent), Added<Parent>>                    │
│    └─► Find parent entity                                   │
│    └─► parent.Children.push(child_entity)                   │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  2. cleanup_removed_parents                                  │
│                                                              │
│  When: Parent component removed                             │
│  Does: Remove entity from old parent's Children list        │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  3. propagate_transforms                                     │
│                                                              │
│  When: Root entity's Transform changes                      │
│  Does: Update GlobalTransform for root + all descendants    │
│                                                              │
│  Query<                                                      │
│    (&Transform, &mut GlobalTransform, Option<&Children>),   │
│    (Changed<Transform>, Without<Parent>)  ← Root only       │
│  >                                                           │
│                                                              │
│  Pseudocode:                                                 │
│  for (transform, mut global, children) in roots {           │
│      global.matrix = transform.compute_matrix();            │
│      if let Some(children) = children {                     │
│          propagate_recursive(world, children, global);      │
│      }                                                       │
│  }                                                           │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  4. propagate_transforms_for_reparented                      │
│                                                              │
│  When: Entity's Parent changes                              │
│  Does: Immediately update GlobalTransform                   │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  5. propagate_transforms_for_changed_children                │
│                                                              │
│  When: Child's local Transform changes                      │
│  Does: Update its GlobalTransform and descendants           │
└─────────────────────────────────────────────────────────────┘
```

### Recursive Propagation

The key algorithm is recursive matrix multiplication:

```rust
fn propagate_recursive(
    world: &World,
    children: &[Entity],
    parent_global: &GlobalTransform,
) {
    for &child in children {
        let Ok((child_local, mut child_global, grandchildren)) 
            = world.get::<(&Transform, &mut GlobalTransform, Option<&Children>)>(child)
        else {
            continue;
        };

        // Child's global = Parent's global * Child's local
        child_global.matrix = parent_global.matrix * child_local.compute_matrix();

        // Recursively update grandchildren
        if let Some(grandchildren) = grandchildren {
            propagate_recursive(world, &grandchildren.0, &child_global);
        }
    }
}
```

**Mathematical detail**:

```text
Matrix Multiplication for Transforms
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Given:
  Parent global transform: P (4x4 matrix)
  Child local transform: C (4x4 matrix)

Child global transform = P × C

In homogeneous coordinates:
┌───┐   ┌─────┐   ┌───┐
│ X │   │ P   │   │ x │
│ Y │ = │   P │ × │ y │
│ Z │   │     │   │ z │
│ 1 │   │─────┘   │ 1 │
└───┘   └─────────┘ └───┘

This encodes:
- Translation: P's translation + rotated/scaled C's translation
- Rotation: P's rotation * C's rotation
- Scale: P's scale * C's scale
```

---

## Rust-Specific Patterns

Praxis uses several Rust patterns that might be unfamiliar to developers coming from other languages.

### Lifetimes in Render Context

Lifetimes are Rust's way of tracking how long references are valid. The render context doesn't need explicit lifetimes because it owns all its data:

```rust
pub struct RenderContext {
    pub device: Arc<Device>,  // Owned
    surface: Arc<Surface>,    // Owned
    swapchain: Arc<Swapchain>, // Owned
    // ... all owned
}
```

However, when passing data to render functions, we use references with lifetimes:

```rust
pub struct MeshRenderCommands<'a> {
    pub view: Mat4,  // Copied
    pub proj: Mat4,  // Copied
    pub draw_commands: &'a [DrawCommand],  // Borrowed with lifetime 'a
}
```

The `'a` lifetime says: "The reference `draw_commands` must live at least as long as this struct."

```text
Lifetime Example
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_frame() {
    let draw_commands = vec![...];  // ◄─── Created here
                                    //      Lifetime begins
    
    let cmds = MeshRenderCommands {
        view: camera_view,
        proj: camera_proj,
        draw_commands: &draw_commands,  // ◄─── Borrowed here
    };                                  //      Lifetime 'a
    
    render_context.render_meshes(&cmds)?;  // ◄─── Used here
                                            //      Lifetime 'a still valid
    
}  // ◄─── draw_commands dropped here
   //      Lifetime ends
   //      cmds can't exist beyond this point!
```

**Why this matters**: The compiler prevents use-after-free bugs. If `cmds` escaped the function, it would hold a dangling reference.

### Trait Objects for Asset Loading

Praxis uses trait objects to allow different types of asset loaders:

```rust
/// Generic trait for loading assets from files
pub trait AssetLoader<T> {
    fn load(&self, path: impl AsRef<Path>) -> Result<T>;
}

/// Concrete implementation for OBJ meshes
pub struct MeshLoader;

impl AssetLoader<MeshData> for MeshLoader {
    fn load(&self, path: impl AsRef<Path>) -> Result<MeshData> {
        // ... OBJ parsing logic
    }
}
```

This can be used with **static dispatch** (monomorphization):

```rust
fn load_mesh<L: AssetLoader<MeshData>>(loader: &L, path: &str) -> Result<MeshData> {
    loader.load(path)  // Compiler knows exact type at compile time
}
```

Or **dynamic dispatch** (trait objects):

```rust
fn load_asset(loader: &dyn AssetLoader<MeshData>, path: &str) -> Result<MeshData> {
    loader.load(path)  // Virtual call, determined at runtime
}
```

```text
Static vs Dynamic Dispatch
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Static Dispatch (Monomorphization):
────────────────────────────────────

fn load<L: AssetLoader<T>>(loader: &L) { ... }

Called with MeshLoader:
  Compiler generates: load__MeshLoader()
  - Direct function call
  - Can be inlined
  - Fast

Called with TextureLoader:
  Compiler generates: load__TextureLoader()
  - Different function
  - Also fast


Dynamic Dispatch (Trait Objects):
──────────────────────────────────

fn load(loader: &dyn AssetLoader<T>) { ... }

Runtime:
  loader ──► [vtable pointer] ──► [load function pointer] ──► MeshLoader::load
  
  - Indirect call through vtable
  - Cannot be inlined
  - Slightly slower
  - But allows heterogeneous collections!

Example:
let loaders: Vec<Box<dyn AssetLoader<MeshData>>> = vec![
    Box::new(ObjLoader),
    Box::new(GltfLoader),
    Box::new(FbxLoader),
];
```

**When Praxis uses each**:
- Static dispatch: Most of the time (better performance)
- Dynamic dispatch: Future plugin systems

### Component Derive Macro

The `Component` derive macro is a procedural macro that generates code at compile time:

```rust
#[derive(Component)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
```

**What it expands to** (simplified):

```rust
// Manually written equivalent:
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Component for Transform {
    type Storage = TableStorage;  // Dense storage for small types
    
    // ... other trait methods
}

// Register with type registry
inventory::submit! {
    ComponentInfo::new::<Transform>("Transform")
}
```

The derive macro:
1. Implements the `Component` trait
2. Chooses optimal storage strategy
3. Registers the type with ECS
4. Generates serialization code (if needed)

**Why use macros**:
- Reduces boilerplate
- Ensures consistency
- Compiler catches errors early
- Easy to add components

```rust
// Without macro:
impl Component for MyComponent {
    type Storage = TableStorage;
    const IS_ARCHETYPE_BOUND: bool = false;
    // ... many more methods
}

// With macro:
#[derive(Component)]
struct MyComponent { ... }  // Done!
```

---

## Lighting System Architecture

The lighting system in Praxis provides dynamic lighting support with both directional and point lights. Understanding how lighting data flows from ECS components to GPU shaders is essential for creating visually compelling scenes.

### Overview

The lighting system consists of three main layers:

1. **ECS Layer**: Light components (`DirectionalLight`, `PointLight`) attached to entities
2. **Collection Layer**: `gather_lighting_system` that queries lights and populates `LightingData` resource
3. **GPU Layer**: `LightingUniforms` structure uploaded to GPU for shader consumption

### High-Level Data Flow

```text
Lighting System Data Flow
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│                      ECS World                               │
│                                                              │
│  Entity 0: DirectionalLight, Transform                       │
│  Entity 1: PointLight, Transform                             │
│  Entity 2: PointLight, Transform, GlobalTransform           │
│  Entity 3: Mesh, Transform                                   │
│  ...                                                         │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           │ Queries entities with light components
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│          gather_lighting_system                              │
│                                                              │
│  Query<(&DirectionalLight, Option<&Transform>)>             │
│  Query<(&PointLight, Option<&GlobalTransform>,              │
│         Option<&Transform>)>                                 │
│                                                              │
│  For each DirectionalLight:                                  │
│    - Extract direction, color, intensity                     │
│    - Apply Transform rotation if present                     │
│    - Add to DirectionalLightInfo collection                  │
│                                                              │
│  For each PointLight:                                        │
│    - Extract color, intensity, range                         │
│    - Get position from GlobalTransform or Transform          │
│    - Add to PointLightInfo collection                        │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           │ Populates resource
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│               LightingData Resource                          │
│                                                              │
│  Vec<DirectionalLightInfo>:                                  │
│    [                                                         │
│      { direction: Vec3, color: Vec3, intensity: f32 },       │
│      { direction: Vec3, color: Vec3, intensity: f32 },       │
│      ...                                                     │
│    ]                                                         │
│                                                              │
│  Vec<PointLightInfo>:                                        │
│    [                                                         │
│      { position: Vec3, color: Vec3, intensity: f32,          │
│        range: f32 },                                         │
│      { position: Vec3, color: Vec3, intensity: f32,          │
│        range: f32 },                                         │
│      ...                                                     │
│    ]                                                         │
│                                                              │
│  ambient_color: Vec3                                         │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           │ Converted by render system
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│            LightingUniforms (std140 layout)                  │
│                                                              │
│  directional_lights: [DirectionalLightData; 8]               │
│    [                                                         │
│      { direction: [f32; 4], color: [f32; 4],                │
│        intensity: f32, _padding: [f32; 3] },                 │
│      ...                                                     │
│    ]                                                         │
│                                                              │
│  point_lights: [PointLightData; 16]                          │
│    [                                                         │
│      { position: [f32; 4], color: [f32; 4],                 │
│        intensity: f32, range: f32, _padding: [f32; 2] },     │
│      ...                                                     │
│    ]                                                         │
│                                                              │
│  ambient_color: [f32; 4]                                     │
│  directional_light_count: u32                                │
│  point_light_count: u32                                      │
│  _padding: [u32; 2]                                          │
│                                                              │
│  Total size: 1184 bytes                                      │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           │ Uploaded to GPU
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│           LightingUniformBuffer (GPU Memory)                 │
│                                                              │
│  Host-visible, coherent buffer                               │
│  Bound to descriptor set 0, binding 2                        │
│  Updated every frame with current lighting state             │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           │ Accessed by shaders
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│              Fragment Shader (GLSL)                          │
│                                                              │
│  layout(set=0, binding=2) uniform LightingData {             │
│      DirectionalLight directional_lights[8];                 │
│      PointLight point_lights[16];                            │
│      vec4 ambient_color;                                     │
│      uint directional_light_count;                           │
│      uint point_light_count;                                 │
│  } lighting;                                                 │
│                                                              │
│  // Compute lighting for each pixel:                         │
│  vec3 final_color = ambient_color.rgb * base_color;          │
│                                                              │
│  // Add directional lights                                   │
│  for (uint i = 0; i < lighting.directional_light_count; i++){│
│      final_color += compute_directional_light(              │
│          lighting.directional_lights[i], ...);               │
│  }                                                           │
│                                                              │
│  // Add point lights                                         │
│  for (uint i = 0; i < lighting.point_light_count; i++) {    │
│      final_color += compute_point_light(                    │
│          lighting.point_lights[i], ...);                     │
│  }                                                           │
└─────────────────────────────────────────────────────────────┘
```

### Component Definitions

#### DirectionalLight Component

Represents a light source at infinite distance (like the sun):

```rust
#[derive(Component)]
pub struct DirectionalLight {
    /// Direction the light is shining (normalized)
    pub direction: Vec3,
    
    /// RGB color of the light
    pub color: Vec3,
    
    /// Intensity multiplier
    pub intensity: f32,
}
```

**Key characteristics**:
- Position doesn't matter, only direction
- Affects all objects equally
- No attenuation with distance
- Useful for: sun, moon, ambient sky light

**Example usage**:
```rust
// Create a sun-like light
world.spawn(DirectionalLight::new(
    Vec3::new(0.3, -0.8, 0.5).normalize(),  // Direction
    Vec3::new(1.0, 0.95, 0.85),              // Warm white
    1.0,                                      // Full intensity
));

// Can be rotated with a Transform
world.spawn((
    DirectionalLight::new(/* ... */),
    Transform::from_rotation(Quat::from_rotation_y(angle)),
));
```

#### PointLight Component

Represents an omnidirectional light source with distance attenuation:

```rust
#[derive(Component)]
pub struct PointLight {
    /// RGB color of the light
    pub color: Vec3,
    
    /// Intensity at the source
    pub intensity: f32,
    
    /// Maximum range (beyond this, no effect)
    pub range: f32,
}
```

**Key characteristics**:
- Position matters (from Transform component)
- Radiates in all directions
- Attenuation based on distance
- Useful for: light bulbs, torches, explosions

**Example usage**:
```rust
// Create a point light at a specific location
world.spawn((
    Transform::from_xyz(0.0, 5.0, 0.0),     // Position
    PointLight::new(
        Vec3::new(1.0, 0.8, 0.6),            // Warm color
        25.0,                                 // High intensity
        15.0,                                 // 15-unit range
    ),
));

// Can be moved/animated by updating Transform
```

### The gather_lighting_system

This system bridges the ECS and rendering layers:

```text
gather_lighting_system Flow
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│  fn gather_lighting_system(                                  │
│      mut lighting_data: ResMut<LightingData>,                │
│      directional_lights: Query<(                             │
│          &DirectionalLight,                                  │
│          Option<&Transform>                                  │
│      )>,                                                     │
│      point_lights: Query<(                                   │
│          &PointLight,                                        │
│          Option<&GlobalTransform>,                           │
│          Option<&Transform>                                  │
│      )>,                                                     │
│  )                                                           │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
        ┌──────────────────────────────────┐
        │  1. Clear previous frame's data  │
        │     lighting_data.clear()        │
        └──────────────┬───────────────────┘
                       │
                       ▼
        ┌────────────────────────────────────────────┐
        │  2. Process Directional Lights             │
        │                                            │
        │  for (light, maybe_transform) in query {   │
        │      // Get direction (apply rotation)     │
        │      let world_dir = if let Some(t) = ... {│
        │          t.rotation * light.direction      │
        │      } else {                               │
        │          light.direction                   │
        │      };                                     │
        │                                            │
        │      // Add to collection                  │
        │      lighting_data.directional_lights.push(│
        │          DirectionalLightInfo {             │
        │              direction: world_dir,          │
        │              color: light.color,            │
        │              intensity: light.intensity,    │
        │          }                                  │
        │      );                                     │
        │  }                                          │
        └──────────────┬─────────────────────────────┘
                       │
                       ▼
        ┌────────────────────────────────────────────┐
        │  3. Process Point Lights                   │
        │                                            │
        │  for (light, global_t, local_t) in query { │
        │      // Get position (prefer global)       │
        │      let world_pos = if let Some(g) = ... {│
        │          g.translation()                   │
        │      } else if let Some(t) = local_t {     │
        │          t.translation                     │
        │      } else {                               │
        │          Vec3::ZERO                         │
        │      };                                     │
        │                                            │
        │      // Add to collection                  │
        │      lighting_data.point_lights.push(      │
        │          PointLightInfo {                   │
        │              position: world_pos,           │
        │              color: light.color,            │
        │              intensity: light.intensity,    │
        │              range: light.range,            │
        │          }                                  │
        │      );                                     │
        │  }                                          │
        └────────────────────────────────────────────┘
```

### Transform Handling

The system correctly handles hierarchical transforms:

```text
Transform Hierarchy for Point Lights
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Example: Light attached to a moving character's hand

┌──────────────────────────────────────┐
│  Character Entity                    │
│  Transform: position (10, 0, 5)      │
│  GlobalTransform: computed           │
└──────────┬───────────────────────────┘
           │ Parent
           │
           ▼
┌──────────────────────────────────────┐
│  Hand Entity                         │
│  Transform: local offset (0.5, 1, 0) │  ← Local to character
│  GlobalTransform: (10.5, 1, 5)       │  ← World space
│  Parent: Character                   │
└──────────┬───────────────────────────┘
           │ Parent
           │
           ▼
┌──────────────────────────────────────┐
│  Torch Entity                        │
│  Transform: local offset (0, 0, 0.3) │  ← Local to hand
│  GlobalTransform: (10.5, 1, 5.3)     │  ← World space
│  PointLight: red, intensity 20       │
│  Parent: Hand                        │
└──────────────────────────────────────┘

The gather_lighting_system uses GlobalTransform (10.5, 1, 5.3)
so the light position is correct in world space!

When character moves:
  Character moves → GlobalTransform updates propagate
  → Hand GlobalTransform updates
  → Torch GlobalTransform updates
  → Next frame, light position reflects new location
```

### GPU Memory Layout (std140)

The `LightingUniforms` struct uses std140 layout for GPU compatibility:

```text
std140 Memory Layout
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

DirectionalLightData (48 bytes each):
┌──────────────────────────────────────────────────────┐
│  Offset 0:  direction [f32; 4]   (16 bytes)          │
│            └─ xyz: actual direction, w: padding      │
│  Offset 16: color [f32; 4]       (16 bytes)          │
│            └─ rgb: color, a: padding                 │
│  Offset 32: intensity f32        (4 bytes)           │
│  Offset 36: _padding [f32; 3]    (12 bytes)          │
│            └─ Align to 48 bytes total                │
└──────────────────────────────────────────────────────┘

PointLightData (48 bytes each):
┌──────────────────────────────────────────────────────┐
│  Offset 0:  position [f32; 4]    (16 bytes)          │
│  Offset 16: color [f32; 4]       (16 bytes)          │
│  Offset 32: intensity f32        (4 bytes)           │
│  Offset 36: range f32            (4 bytes)           │
│  Offset 40: _padding [f32; 2]    (8 bytes)           │
└──────────────────────────────────────────────────────┘

Complete LightingUniforms (1184 bytes):
┌──────────────────────────────────────────────────────┐
│  Offset 0:    directional_lights [...; 8]            │
│               8 × 48 = 384 bytes                     │
│  Offset 384:  point_lights [...; 16]                 │
│               16 × 48 = 768 bytes                    │
│  Offset 1152: ambient_color [f32; 4]   (16 bytes)   │
│  Offset 1168: directional_light_count  (4 bytes)    │
│  Offset 1172: point_light_count        (4 bytes)    │
│  Offset 1176: _padding [u32; 2]        (8 bytes)    │
│  Total: 1184 bytes                                   │
└──────────────────────────────────────────────────────┘

Why padding?
  - vec3 in std140 takes 16 bytes (same as vec4)
  - Array elements must be aligned to 16 bytes
  - Struct size must be multiple of largest alignment
  - GPU hardware requires this for optimal access
```

### Shader Integration

The fragment shader uses the lighting data to compute final pixel colors:

```glsl
// Descriptor binding in fragment shader
layout(set = 0, binding = 2, std140) uniform LightingData {
    DirectionalLight directional_lights[8];
    PointLight point_lights[16];
    vec4 ambient_color;
    uint directional_light_count;
    uint point_light_count;
} lighting;

// Blinn-Phong lighting computation
vec3 compute_lighting(vec3 base_color, vec3 world_pos, vec3 normal) {
    // Start with ambient
    vec3 result = lighting.ambient_color.rgb * base_color;
    
    // Add directional lights
    for (uint i = 0; i < lighting.directional_light_count; i++) {
        DirectionalLight light = lighting.directional_lights[i];
        
        // Lambert diffuse
        float diff = max(dot(normal, -light.direction.xyz), 0.0);
        result += light.color.rgb * light.intensity * diff * base_color;
    }
    
    // Add point lights
    for (uint i = 0; i < lighting.point_light_count; i++) {
        PointLight light = lighting.point_lights[i];
        
        // Calculate distance and attenuation
        vec3 light_dir = light.position.xyz - world_pos;
        float distance = length(light_dir);
        
        if (distance < light.range) {
            light_dir = normalize(light_dir);
            
            // Distance attenuation
            float attenuation = 1.0 - (distance / light.range);
            attenuation = attenuation * attenuation; // Quadratic falloff
            
            // Lambert diffuse
            float diff = max(dot(normal, light_dir), 0.0);
            
            result += light.color.rgb * light.intensity * 
                      diff * attenuation * base_color;
        }
    }
    
    return result;
}
```

### Performance Considerations

#### Array Size Limits

```text
Light Count Trade-offs
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Current limits:
  MAX_DIRECTIONAL_LIGHTS = 8   (384 bytes)
  MAX_POINT_LIGHTS = 16         (768 bytes)
  Total buffer size = 1184 bytes

Why these limits?
  ✓ Fits comfortably in minimum UBO size (16KB)
  ✓ Reasonable for most scenes
  ✓ Balance between flexibility and performance
  
Typical usage:
  Outdoor scene:  1-2 directional + 0-4 point lights
  Indoor scene:   0-1 directional + 4-12 point lights
  Complex scene:  2-3 directional + 8-16 point lights

Performance impact:
  - Shader loops through active lights only (using counts)
  - Unused array slots have zero cost
  - Increasing limits requires shader recompilation
```

#### Per-Frame Update Cost

```text
Update Performance
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Each frame:
  1. gather_lighting_system runs (CPU)
     - Query DirectionalLight entities:    ~10-100 ns per light
     - Query PointLight entities:          ~10-100 ns per light
     - Vec allocations (clear + push):     ~1-2 μs total
     
  2. Convert to GPU format (CPU)
     - Copy to LightingUniforms:           ~100-500 ns
     - No allocations (reuse buffer)
     
  3. Upload to GPU
     - Write to mapped buffer:             ~100-500 ns
     - No actual GPU transfer (host-visible)
     
  Total CPU cost: ~5-10 μs per frame (negligible)
  
  GPU cost: Lighting computation in fragment shader
    - Per pixel, per light
    - Dominant cost is fragment shader execution
```

### Common Patterns

#### Dynamic Day-Night Cycle

```rust
fn day_night_system(
    time: Res<GameTime>,
    mut sun_light: Query<&mut DirectionalLight, With<Sun>>,
) {
    for mut light in sun_light.iter_mut() {
        // Rotate sun direction based on time
        let angle = time.elapsed_seconds() * 0.1;
        let rotation = Quat::from_rotation_x(angle);
        light.direction = rotation * Vec3::NEG_Y;
        
        // Adjust color and intensity based on time of day
        let t = (angle.sin() + 1.0) * 0.5; // 0 to 1
        light.color = Vec3::lerp(
            Vec3::new(0.3, 0.4, 0.6),  // Night (blue)
            Vec3::new(1.0, 0.95, 0.8), // Day (warm)
            t
        );
        light.intensity = t; // Dim at night
    }
}
```

#### Flickering Torch

```rust
fn torch_flicker_system(
    time: Res<GameTime>,
    mut torches: Query<&mut PointLight, With<Torch>>,
) {
    for mut light in torches.iter_mut() {
        // Random flicker effect
        let flicker = (time.elapsed_seconds() * 10.0).sin() * 0.1 + 1.0;
        light.intensity = 20.0 * flicker;
    }
}
```

#### Following Light (Flashlight)

```rust
fn flashlight_system(
    player: Query<&Transform, With<Player>>,
    mut flashlight: Query<&mut Transform, (With<PointLight>, With<Flashlight>)>,
) {
    if let Ok(player_transform) = player.get_single() {
        for mut light_transform in flashlight.iter_mut() {
            // Position light at player position
            light_transform.translation = player_transform.translation;
            light_transform.translation.y += 1.5; // Eye height
        }
    }
}
```

### Debugging Tips

1. **Visualize light positions**: Spawn debug spheres at point light locations
2. **Check light counts**: Log `LightingData` to verify lights are collected
3. **Verify transforms**: Ensure GlobalTransform is updated before gathering
4. **Shader debugging**: Use simple colors to isolate lighting issues
5. **Performance profiling**: Monitor fragment shader cost with many lights

---

## Further Reading

- **Vulkan Tutorial**: https://vulkan-tutorial.com/
- **Vulkano Documentation**: https://docs.rs/vulkano/
- **Bevy ECS Book**: https://bevyengine.org/learn/book/
- **Praxis Architecture Docs**: `docs/architecture.md`
- **Praxis Mesh System**: `docs/mesh_system.md`

---

## Tips for Contributors

1. **Start with ECS**: Understanding the ECS data flow makes everything else easier.
2. **Use logging**: Praxis uses `tracing`. Add `trace!()`, `debug!()`, `info!()` liberally.
3. **Read error messages**: Vulkan errors can be cryptic, but Praxis adds context with `color-eyre`.
4. **Check examples**: The `examples/` directory shows real usage patterns.
5. **Ask questions**: Open an issue or discussion if something's unclear!

---

This guide will be updated as Praxis evolves. Contributions and feedback welcome!
