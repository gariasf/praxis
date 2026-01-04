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
9. [Lighting System Architecture](#lighting-system-architecture)
10. [Material System](#material-system)
11. [Animation System](#animation-system)
12. [Physics System](#physics-system)
13. [Shadow Mapping System](#shadow-mapping-system)
14. [Normal Mapping](#normal-mapping)
15. [Post-Processing Pipeline](#post-processing-pipeline)
16. [Particle System](#particle-system)

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

In the current implementation, Praxis uses a unified rendering API with `DrawCommand` and `RenderCommands`:

```rust
// From RenderContext::render()
let draw_commands = vec![
    DrawCommand {
        mesh_id: "cube".to_string(),
        model: Mat4::IDENTITY,
        texture_name: None, // Optional: use Some("texture_name") for custom texture
        material_properties: None, // Optional: use Some() for custom materials
    },
];

let cmds = RenderCommands {
    view: camera_view,
    proj: camera_proj,
    draw_commands: &draw_commands,
    lighting: None, // Optional: use Some() for dynamic lighting
};

render_context.render(&cmds)?;
```

**Key features of the unified API**:
- Single `render()` method handles all rendering
- `DrawCommand` specifies mesh, transform, optional texture, and optional material
- `RenderCommands` provides camera matrices, draw commands, and optional lighting
- Automatic material batching and descriptor set reuse for optimal performance

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

Here's how data flows from ECS components to the GPU using the unified rendering API:

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
   │         model: global_transform, │
   │         texture_name: None,      │
   │         material_properties: None│
   │     });                          │
   │ }                                │
   └──────────┬───────────────────────┘
              │
              ▼
4. Submit to Renderer
   ┌──────────────────────────────────┐
   │ render_context.render(           │
   │     &RenderCommands {            │
   │         view: camera_view,       │
   │         proj: camera_proj,       │
   │         draw_commands: &draw_cmds│
   │         lighting: None,          │
   │     }                            │
   │ )                                │
   └──────────┬───────────────────────┘
              │
              ▼
5. GPU Rendering
   ┌──────────────────────────────────┐
   │ For each draw command:           │
   │   - Bind mesh buffers            │
   │   - Bind descriptor sets         │
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
pub struct RenderCommands<'a> {
    pub view: Mat4,  // Copied
    pub proj: Mat4,  // Copied
    pub draw_commands: &'a [DrawCommand],  // Borrowed with lifetime 'a
    pub lighting: Option<&'a LightingUniforms>,
}
```

The `'a` lifetime says: "The reference `draw_commands` must live at least as long as this struct."

```text
Lifetime Example
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_frame() {
    let draw_commands = vec![...];  // ◄─── Created here
                                    //      Lifetime begins
    
    let cmds = RenderCommands {
        view: camera_view,
        proj: camera_proj,
        draw_commands: &draw_commands,  // ◄─── Borrowed here
        lighting: None,
    };                                  //      Lifetime 'a
    
    render_context.render(&cmds)?;  // ◄─── Used here
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
```

---

## Animation System

The animation system in Praxis brings characters and objects to life through skeletal animation. Understanding how animations work is essential for creating dynamic, believable game characters.

### What is Skeletal Animation?

Skeletal animation is a technique where a character is rigged with a hierarchy of bones, and animations deform the mesh by transforming these bones over time.

```text
Skeletal Animation Concept
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Character Mesh + Skeleton
┌─────────────────────────────────────────────────────────────┐
│                                                              │
│        ●  ← Head bone                                       │
│        │                                                     │
│    ┌───┴───┐                                                │
│    │       │    ← Arm bones                                 │
│    ●       ●                                                 │
│        │                                                     │
│        ●  ← Spine bone                                      │
│        │                                                     │
│    ┌───┴───┐                                                │
│    │       │    ← Leg bones                                 │
│    ●       ●                                                 │
│                                                              │
│  Each bone:                                                  │
│   - Has a position, rotation, and scale                     │
│   - Connected to parent bone (hierarchy)                    │
│   - Influences nearby mesh vertices                         │
│                                                              │
│  Animation changes bone transforms over time                │
│  Mesh deforms automatically to follow bones                 │
└─────────────────────────────────────────────────────────────┘
```

### Core Animation Components

Praxis uses several ECS components for animation:

```rust
// Core components
Skeleton          // Bone hierarchy and bind poses
AnimationPlayer   // Controls playback of animations
AnimatedPose      // Stores computed bone transforms

// Optional advanced component
AnimationBlender  // Advanced blending features
```

#### Skeleton Component

Defines the bone structure:

```text
Skeleton Hierarchy Example
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Root (Pelvis)
 │
 ├─► Spine
 │    │
 │    ├─► Left Arm → Left Hand
 │    └─► Right Arm → Right Hand
 │
 ├─► Left Leg → Left Foot
 └─► Right Leg → Right Foot

Key concepts:
- Bind Pose: Default "rest" position
- Parent-Child: Children move with parents
- Local Transform: Relative to parent
- World Transform: Absolute position
```

#### AnimationClip

Stores keyframe data:

```text
Animation Clip Structure
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

"Walk" Animation (2.0 seconds)
│
├─► Bone 0 (Root)
│    ├─► Translation: [(0.0s, 0,0,0), (1.0s, 1,0,0), (2.0s, 2,0,0)]
│    ├─► Rotation: [(0.0s, 0°), (1.0s, 5°), (2.0s, 0°)]
│    └─► Scale: [(0.0s, 1,1,1), ...]
│
├─► Bone 1 (Spine)
│    └─► ...
│
└─► Bone N
     └─► ...

Between keyframes: smooth interpolation
- Translation/Scale: Linear interpolation (LERP)
- Rotation: Spherical interpolation (SLERP)
```

### Animation Data Flow

```text
Per-Frame Animation Update
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. Update Time
   current_time += delta_time × speed
   │
   ▼
2. Sample Keyframes
   For each bone track:
     translation = interpolate(keyframes, current_time)
     rotation = interpolate(keyframes, current_time)
     scale = interpolate(keyframes, current_time)
   │
   ▼
3. Blend Animations (if multiple playing)
   final_pose = animation1 × weight1 + animation2 × weight2
   │
   ▼
4. Propagate Hierarchy
   For each bone (parent before child):
     world_transform = parent_transform × local_transform
   │
   ▼
5. Compute Skinning
   For each bone:
     skinning_matrix = world_transform × inverse_bind_matrix
   │
   ▼
6. GPU Skinning
   Vertex shader applies bone transforms to vertices
```

### Keyframe Interpolation

The system automatically interpolates between keyframes for smooth motion:

```text
Interpolation Example
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Translation Keyframes:
  t=0.0s:  Position (0, 0, 0)
  t=1.0s:  Position (10, 0, 0)

Query at t=0.5s:
  ┌─────────────────────────────────────────┐
  │ 1. Find surrounding keyframes:          │
  │    before = 0.0s                        │
  │    after  = 1.0s                        │
  │                                         │
  │ 2. Calculate blend weight:              │
  │    t = (0.5 - 0.0) / (1.0 - 0.0) = 0.5 │
  │                                         │
  │ 3. Interpolate:                         │
  │    result = lerp(                       │
  │      (0,0,0),                           │
  │      (10,0,0),                          │
  │      0.5                                │
  │    ) = (5, 0, 0)                        │
  └─────────────────────────────────────────┘

Timeline:
  0.0s      0.5s       1.0s
  (0,0,0) → (5,0,0) → (10,0,0)
            ↑
        Interpolated!
```

### Animation Blending

Multiple animations can play simultaneously with weights:

```text
Animation Blending
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Playing Two Animations:
  Walk (weight: 0.7)  → Bone rotation: 10°
  Run  (weight: 0.3)  → Bone rotation: 30°

Blended Result:
  final_rotation = 0.7 × 10° + 0.3 × 30°
                 = 7° + 9°
                 = 16°

Visual:
  Walk only (1.0, 0.0):  ████████████░░░░░░░░
  Blend (0.7, 0.3):      ████████████████░░░░
  Run only (0.0, 1.0):   ████████████████████
```

### Advanced Blending Features

The `AnimationBlender` component provides sophisticated blending:

#### 1. Cross-Fade Transitions

Smooth transitions between animations:

```text
Cross-Fade Example
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

blender.cross_fade("Idle", "Walk", 0.3); // 0.3 second transition

Time:     0.0      0.15     0.3
          │         │        │
Idle:     100%      50%      0%     ████████░░░░░░░░
Walk:     0%        50%      100%   ░░░░░░░░████████

Result: Smooth blend from idle to walk over 0.3 seconds
```

#### 2. 1D Blend Trees

Parameter-driven blending (e.g., based on speed):

```text
1D Blend Tree
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Speed Parameter: 0.0 to 1.0

  0.0         0.5         1.0
  Idle        Walk        Run
  │           │           │
  └───────────┴───────────┘

Set speed to 0.75:
  - Between Walk (0.5) and Run (1.0)
  - 50% Walk + 50% Run
  - Smooth transition as speed changes
```

#### 3. 2D Blend Trees

Two-parameter blending (e.g., directional movement):

```text
2D Blend Tree
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

      Forward
        (0,1)
          │
          │
Left ─────┼───── Right
 (-1,0)   │      (1,0)
          │
        Back
        (0,-1)

Input: (0.5, 0.5) - Moving forward-right
Result: Blend Forward + Right animations
Use case: 8-directional character movement
```

#### 4. Layered Animation

Multiple animation layers with bone masking:

```text
Layered Animation
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Layer 0 (Base): Walk
  Mask: All bones
  Result: ████████████████  Full body walks

Layer 1 (Upper): Wave
  Mask: Right arm only
  Result: ░░░░░░░░████░░░░  Only right arm waves

Combined: Character walks while waving right arm!

Use cases:
  - Walk + Upper body action (aim, wave, drink)
  - Run + Lower body (special footwork)
  - Idle + Facial animation
```

### GLTF Animation Loading

Praxis supports loading animations from GLTF files:

```rust
use praxis_assets::GltfLoader;

// Load GLTF file
let loader = GltfLoader::new();
let asset = loader.load_gltf("assets/models/character.gltf")?;

// Extract animations
let mut player = AnimationPlayer::new();
for animation in &asset.animations {
    let name = animation.name.clone().unwrap_or_default();
    player.add_clip(name, animation.clip.clone());
}

// Use skeleton
let skeleton = asset.skins[0].skeleton.clone();
let pose = AnimatedPose::new(skeleton.bone_count());

world.spawn((skeleton, player, pose));
```

### Animation Update System

Animations are updated each frame by a system:

```rust
fn animation_system(
    mut query: Query<(&Skeleton, &mut AnimationPlayer, &mut AnimatedPose)>
) {
    let delta_time = 0.016; // From timing system
    praxis_scene::update_animations(delta_time, &mut query);
}
```

### Transform Hierarchy Propagation

Bone transforms propagate through the hierarchy:

```text
Hierarchy Propagation
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Shoulder (Parent)
  Local: Rotation 45°
  World: Rotation 45°
  │
  └─► Elbow (Child)
       Local: Rotation 30° (relative to parent)
       World: Rotation 75° (45° + 30°)
       │
       └─► Hand (Grandchild)
            Local: Rotation 15°
            World: Rotation 90° (75° + 15°)

When shoulder rotates:
  - Elbow moves with shoulder (keeps same local transform)
  - Hand moves with elbow (keeps same local transform)
  - All world transforms update automatically

Formula for each bone:
  world_transform[bone] = world_transform[parent] × local_transform[bone]
```

### Performance Considerations

```text
Animation Performance
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Typical costs (60 FPS):

50-bone character, 1 animation:
  - Keyframe sampling: ~1-2 μs
  - Hierarchy propagation: ~1-2 μs
  - Total: ~5 μs per character

100 animated characters:
  - Animation update: ~0.5 ms
  - ~3% of 16.67ms frame budget
  - Very reasonable!

Optimization tips:
  ✓ Use bone masking to skip unused bones
  ✓ Reduce animation update rate for distant characters
  ✓ Use LOD: simpler skeletons at distance
  ✓ Cull offscreen characters

Scalability guidelines:
  ✓ <50 bones: Excellent
  ✓ 50-100 bones: Good
  ⚠ 100+ bones: Consider LOD
```

### Common Animation Patterns

```rust
// 1. Simple looping animation
player.add_clip("Walk", walk_clip);
player.play("Walk");
player.set_looping("Walk", true);

// 2. Animation speed control
player.set_speed("Walk", 2.0); // 2x speed

// 3. Multiple animations with weights
player.play("Walk");
player.set_weight("Walk", 0.7);
player.play("Idle");
player.set_weight("Idle", 0.3);

// 4. Cross-fade transition
blender.cross_fade("Idle", "Walk", 0.3);

// 5. Parameter-driven blending
blend_tree.set_parameter(0.75); // Speed-based blend
```

### Animation Workflow Summary

```text
Complete Animation Workflow
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. Asset Creation (Artist)
   Create character in 3D software (Blender, Maya, etc.)
   Rig with bones
   Animate
   Export as GLTF
   │
   ▼
2. Loading (Praxis)
   GltfLoader reads file
   Extracts Skeleton + AnimationClips
   │
   ▼
3. Setup (Game Code)
   Create AnimationPlayer
   Add clips to player
   Spawn entity with Skeleton + Player + Pose
   │
   ▼
4. Runtime (Game Loop)
   Animation system updates each frame
   Samples keyframes
   Blends animations
   Propagates hierarchy
   Computes skinning matrices
   │
   ▼
5. Rendering (GPU)
   Vertex shader applies bone transforms
   Mesh deforms to match animation
   Character moves smoothly!
```

For comprehensive details on the animation system, including blending algorithms and implementation details, see [Animation System Documentation](animation_system.md).

---

## Material System

The material system defines how surfaces appear in the rendered image, combining textures with physical properties for realistic rendering.

### What is a Material?

A material is a collection of properties that define how a surface responds to light. In Praxis, materials combine:

- **Textures**: Image data that defines the base appearance
- **Material Properties**: Physical parameters that control lighting behavior
- **Descriptor Sets**: GPU resources that bind textures and properties to shaders

### PBR Material Properties

Praxis uses **Physically-Based Rendering (PBR)** properties that simulate real-world material behavior:

```text
PBR Material Properties
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│  Base Color (RGBA)                                          │
│  - Tint multiplied with texture color                       │
│  - [1.0, 1.0, 1.0, 1.0] = white (no tint)                   │
│  - Example: [1.0, 0.8, 0.3, 1.0] = golden tint             │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  Metallic [0.0 to 1.0]                                      │
│  - Controls metal-like behavior                             │
│  - 0.0 = Dielectric (plastic, wood, stone)                  │
│  - 1.0 = Metallic (gold, silver, copper)                    │
│                                                             │
│  Dielectric (0.0):                                          │
│    • Diffuse reflections dominate                           │
│    • White specular highlights                              │
│    • Base color in diffuse component                        │
│                                                             │
│  Metallic (1.0):                                            │
│    • Specular reflections dominate                          │
│    • Colored specular (from base color)                     │
│    • No diffuse component                                   │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  Roughness [0.0 to 1.0]                                     │
│  - Controls surface smoothness                              │
│  - 0.0 = Perfectly smooth (mirror-like)                     │
│  - 1.0 = Completely rough (matte)                           │
│                                                             │
│  Smooth (0.0):                                              │
│    • Sharp, focused reflections                             │
│    • Clear mirror reflections                               │
│    • Tight specular highlights                              │
│                                                             │
│  Rough (1.0):                                               │
│    • Scattered, blurred reflections                         │
│    • No clear reflections                                   │
│    • Broad, soft highlights                                 │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  Emissive Strength [0.0+]                                   │
│  - Self-illumination intensity                              │
│  - 0.0 = Normal object (affected only by lights)            │
│  - 1.0+ = Glowing object (adds constant color)              │
│                                                             │
│  Use cases:                                                 │
│    • Light sources (lamps, signs)                           │
│    • Neon signs and displays                                │
│    • Glowing effects (magic, sci-fi)                        │
│    • UI elements that should always be visible             │
└─────────────────────────────────────────────────────────────┘
```

### Material Data Structure

The `MaterialProperties` struct is designed for efficient GPU upload:

```rust
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialProperties {
    pub base_color: [f32; 4],      // 16 bytes
    pub metallic: f32,              // 4 bytes
    pub roughness: f32,             // 4 bytes
    pub emissive_strength: f32,     // 4 bytes
    _padding: f32,                  // 4 bytes (alignment)
}
// Total: 32 bytes (aligned to 16-byte boundary)
```

The `#[repr(C)]` attribute ensures memory layout matches GPU expectations. The `Pod` and `Zeroable` traits from bytemuck enable safe byte-level operations for GPU transfer.

### Material Usage Patterns

#### 1. Per-Object Material Properties

Attach properties directly to entities:

```rust
world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    MeshHandle::new("cube"),
    TextureHandle::new("metal"),
    MaterialPropertiesComponent(
        MaterialProperties::new()
            .with_metallic(0.9)
            .with_roughness(0.2)
    ),
));
```

**When to use**:
- Each object needs unique material properties
- Dynamic material changes per object
- Material properties vary with game state

#### 2. Shared Materials via MaterialManager

Create named materials for reuse:

```rust
// During setup
render_context
    .material_manager_mut()
    .create_material_with_properties(
        "polished_gold",
        gold_texture,
        MaterialProperties::new()
            .with_base_color([1.0, 0.8, 0.3, 1.0])
            .with_metallic(1.0)
            .with_roughness(0.1),
    );

// In entity spawning
world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    MeshHandle::new("cube"),
    MaterialHandle::new("polished_gold"),
));
```

**When to use**:
- Many objects share the same material
- Material properties don't change
- Want to update material properties globally

### Descriptor Set Management

Materials use descriptor sets to efficiently bind GPU resources:

```text
Descriptor Set Architecture for Materials
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│                      Per-Frame Setup                         │
│                                                              │
│  Graphics Pipeline has two descriptor set layouts:          │
│    Set 0: Per-object data (transforms, texture, lighting)   │
│    Set 1: Material properties                               │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   Material Batching                          │
│                                                              │
│  Renderer sorts draw commands by material properties:       │
│    1. Group by texture name                                 │
│    2. Group by material properties hash                     │
│                                                              │
│  Result: Objects with identical materials are adjacent      │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Efficient Descriptor Set Creation               │
│                                                              │
│  For each unique material in sorted list:                   │
│    ┌────────────────────────────────────────┐               │
│    │  Material Properties                   │               │
│    │  (metallic, roughness, emissive, etc.) │               │
│    └──────────────┬─────────────────────────┘               │
│                   │ Write to GPU buffer                     │
│                   ▼                                         │
│    ┌────────────────────────────────────────┐               │
│    │  Material Uniform Buffer               │               │
│    │  (32 bytes, host-visible)              │               │
│    └──────────────┬─────────────────────────┘               │
│                   │ Bound to descriptor set                 │
│                   ▼                                         │
│    ┌────────────────────────────────────────┐               │
│    │  Descriptor Set (Set 1)                │               │
│    │  Binding 0: Material properties buffer │               │
│    └────────────────────────────────────────┘               │
│                                                              │
│  This descriptor set is reused for all objects with the     │
│  same material properties!                                  │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      GPU Rendering                           │
│                                                              │
│  For each object in sorted list:                            │
│    1. Bind Set 0 (transforms + texture) - always changes    │
│    2. Bind Set 1 (material) - only if material changed      │
│    3. Draw object                                           │
│                                                              │
│  Example: 100 objects with 10 materials                     │
│    Without batching: 100 material descriptor sets           │
│    With batching: 10 material descriptor sets               │
│    Result: 90% reduction in descriptor set operations       │
└─────────────────────────────────────────────────────────────┘
```

### Performance Benefits

**Material batching provides significant performance gains**:

```text
Material Batching Performance
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Scenario: 500 objects with 25 different materials (20 objects per material)

WITHOUT Material Batching:
┌─────────────────────────────────────────────────────────────┐
│  Descriptor Sets Created: 500                               │
│  Material Binds: 500                                        │
│  Texture Cache Misses: High (random access)                 │
│  Frame Time: ~0.8ms                                         │
└─────────────────────────────────────────────────────────────┘

WITH Material Batching (Praxis Implementation):
┌─────────────────────────────────────────────────────────────┐
│  Descriptor Sets Created: 25                                │
│  Material Binds: 25                                         │
│  Texture Cache Misses: Low (sequential access)              │
│  Frame Time: ~0.3ms                                         │
│                                                              │
│  Improvement: 20x fewer descriptor sets                     │
│              20x fewer GPU binds                            │
│              62% faster frame time                          │
└─────────────────────────────────────────────────────────────┘
```

### Real-World Material Examples

```rust
// 1. POLISHED GOLD: High metallic + low roughness
MaterialProperties::new()
    .with_base_color([1.0, 0.8, 0.3, 1.0])  // Golden color
    .with_metallic(1.0)                      // Fully metallic
    .with_roughness(0.1)                     // Very smooth

// 2. BRUSHED ALUMINUM: High metallic + moderate roughness
MaterialProperties::new()
    .with_base_color([0.9, 0.9, 0.9, 1.0])  // Light gray
    .with_metallic(0.9)                      // Very metallic
    .with_roughness(0.4)                     // Brushed finish

// 3. ROUGH STONE: Low metallic + high roughness
MaterialProperties::new()
    .with_base_color([0.6, 0.6, 0.5, 1.0])  // Gray-brown
    .with_metallic(0.0)                      // Non-metallic
    .with_roughness(0.9)                     // Very rough

// 4. PLASTIC: Low metallic + low roughness
MaterialProperties::new()
    .with_base_color([0.2, 0.4, 0.8, 1.0])  // Blue
    .with_metallic(0.0)                      // Non-metallic
    .with_roughness(0.3)                     // Slightly glossy

// 5. NEON SIGN: Emissive + low roughness
MaterialProperties::new()
    .with_base_color([0.0, 1.0, 1.0, 1.0])  // Cyan
    .with_emissive_strength(3.0)             // Strong glow
    .with_metallic(0.0)
    .with_roughness(0.2)
```

### Rendering Pipeline with Materials

```text
Complete Rendering Pipeline with Materials
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│  1. Application: Build Draw Commands                        │
│     - Query ECS for (Transform, MeshHandle, Texture,        │
│       MaterialPropertiesComponent)                          │
│     - Create DrawCommandWithMaterial for each entity        │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  2. Renderer: Sort by Material                              │
│     - Primary key: Texture name                             │
│     - Secondary key: Material properties (as bytes)         │
│     - Result: Adjacent objects with same material           │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  3. Renderer: Create Descriptor Sets                        │
│     - Track current material state                          │
│     - Create new material descriptor set when changed       │
│     - Reuse previous set when material matches             │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  4. GPU: Record Command Buffer                              │
│     - Begin render pass                                     │
│     - Bind pipeline                                         │
│     - For each object:                                      │
│       * Bind mesh buffers                                   │
│       * Bind Set 0 (transforms + texture) - always          │
│       * Bind Set 1 (material) - only if changed            │
│       * Draw indexed                                        │
│     - End render pass                                       │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  5. GPU: Execute Shaders                                    │
│     - Vertex shader: Transform vertices                     │
│     - Fragment shader:                                      │
│       * Sample texture                                      │
│       * Read material properties from Set 1                 │
│       * Compute lighting with metallic/roughness           │
│       * Add emissive contribution                          │
│       * Output final color                                  │
└─────────────────────────────────────────────────────────────┘
```

### Diagram: PBR Lighting with Materials

```text
PBR Fragment Shader Computation
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Inputs:
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│ Texture Color   │  │ Material Props  │  │ Lighting Data   │
│ (from texture)  │  │ (from Set 1)    │  │ (from Set 0)    │
└────────┬────────┘  └────────┬────────┘  └────────┬────────┘
         │                    │                     │
         │                    │                     │
         └────────────┬───────┴──────┬──────────────┘
                      │              │
                      ▼              ▼
         ┌────────────────────────────────────┐
         │     Base Color Calculation         │
         │  base = texture * base_color_tint  │
         └────────────┬───────────────────────┘
                      │
                      ▼
         ┌────────────────────────────────────┐
         │     Ambient Contribution           │
         │  ambient = base * ambient_light    │
         └────────────┬───────────────────────┘
                      │
                      ▼
    ┌─────────────────────────────────────────────┐
    │     For Each Directional Light:             │
    │  1. Calculate light direction               │
    │  2. Compute diffuse (Lambert)               │
    │  3. Compute specular (Blinn-Phong)          │
    │  4. Mix based on metallic property:         │
    │     • Dielectric: diffuse + white specular  │
    │     • Metallic: colored specular only       │
    │  5. Apply roughness to specular spread      │
    └─────────────────┬───────────────────────────┘
                      │
                      ▼
    ┌─────────────────────────────────────────────┐
    │     For Each Point Light:                   │
    │  1. Calculate distance and direction        │
    │  2. Apply distance attenuation              │
    │  3. Compute diffuse and specular            │
    │  4. Mix based on metallic                   │
    │  5. Apply roughness                         │
    └─────────────────┬───────────────────────────┘
                      │
                      ▼
         ┌────────────────────────────────────┐
         │     Add Emissive Contribution      │
         │  emissive = base * emissive_strength│
         └────────────┬───────────────────────┘
                      │
                      ▼
         ┌────────────────────────────────────┐
         │     Final Color Output             │
         │  color = ambient + diffuse +       │
         │          specular + emissive       │
         └────────────────────────────────────┘
```

### Best Practices

1. **Material Reuse**: Create shared materials for common surfaces to maximize batching benefits
2. **Property Ranges**: Keep metallic and roughness in [0,1] range for physically accurate results
3. **Emissive Usage**: Use emissive for light sources and UI, not for general brightness adjustment
4. **Base Color**: Use texture colors for variety, base_color for tinting entire materials
5. **Batching Awareness**: Group similar materials to minimize GPU state changes
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

## Physics System

The physics system integrates Rapier3D physics engine with Praxis's ECS architecture, providing rigid body dynamics, collision detection, and realistic physical simulation. Understanding how physics data flows from components to the simulation is essential for creating interactive, physically-behaved objects.

### Overview

The physics system consists of three main layers:

1. **ECS Layer**: Physics components (`RigidBody`, `Collider`, `PhysicsVelocity`) attached to entities
2. **Simulation Layer**: Rapier3D physics engine that computes positions, velocities, and collisions
3. **Synchronization Layer**: Systems that keep ECS transforms in sync with physics bodies

### High-Level Data Flow

```text
Physics System Data Flow
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│                      ECS World                               │
│                                                              │
│  Entity 0: Transform, RigidBody::Dynamic, Collider::Sphere   │
│  Entity 1: Transform, RigidBody::Static, Collider::Cuboid   │
│  Entity 2: Transform, RigidBody::Kinematic, Collider::Box   │
│  ...                                                         │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           │ 1. Sync ECS → Physics
                           │    (sync_physics_transforms_system)
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│               Rapier Physics World                           │
│                                                              │
│  RigidBodySet:                                               │
│    Body 0: position, velocity, mass, forces                  │
│    Body 1: position, velocity, mass, forces                  │
│    ...                                                       │
│                                                              │
│  ColliderSet:                                                │
│    Collider 0: shape (sphere), attached to Body 0           │
│    Collider 1: shape (cuboid), attached to Body 1           │
│    ...                                                       │
│                                                              │
│  Physics Pipeline:                                           │
│    - Broad phase (spatial partitioning)                      │
│    - Narrow phase (precise collision detection)              │
│    - Island manager (grouping connected bodies)              │
│    - Constraint solver (resolve collisions & joints)         │
│    - Integration (update positions from velocities)          │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           │ 2. Run physics simulation
                           │    (physics_step_system)
                           │
                           ▼
                     Fixed Timestep
                     Loop (60Hz)
                           │
                           │ 3. Sync Physics → ECS
                           │    (sync_physics_transforms_system)
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│              Updated ECS Transforms                          │
│                                                              │
│  Dynamic bodies: Updated positions from physics              │
│  Static bodies: Unchanged (never move)                       │
│  Kinematic bodies: Unchanged (controlled by ECS)             │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           │ 4. Rendering & Game Logic
                           │
                           ▼
                    Render Frame
```

### Component Definitions

#### RigidBody Component

Defines how an entity participates in physics simulation:

```rust
#[derive(Component)]
pub enum RigidBody {
    /// Fully simulated body affected by forces
    Dynamic,
    
    /// Immovable body (infinite mass)
    Static,
    
    /// Moved by code/animation, not forces
    Kinematic,
}
```

**Physical meaning**:
- **Dynamic**: Newton's laws apply. F = ma. Subject to gravity, forces, and collisions.
- **Static**: Infinite mass. Unmovable. Used for terrain, walls, buildings.
- **Kinematic**: Velocity set directly. Moves other objects but isn't moved by them. Used for platforms, doors, player controllers.

```text
Rigid Body Types Comparison
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────┬──────────┬──────────┬──────────┬────────────┐
│   Property  │ Dynamic  │  Static  │Kinematic │   Usage    │
├─────────────┼──────────┼──────────┼──────────┼────────────┤
│ Gravity     │   Yes    │    No    │    No    │            │
│ Forces      │   Yes    │    No    │    No    │            │
│ Collisions  │   Yes    │   Yes    │   Yes    │            │
│ Moved by    │ Physics  │  Never   │   Code   │            │
│ Affects     │   All    │ Dynamic  │ Dynamic  │            │
│             │          │Kinematic │          │            │
│ Performance │ Medium   │   Fast   │  Medium  │            │
│ Use Case    │ Props    │ Terrain  │ Platforms│            │
│             │ Actors   │  Walls   │  Doors   │            │
│             │ Physics  │ Buildings│ Character│            │
│             │ Objects  │          │  Control │            │
└─────────────┴──────────┴──────────┴──────────┴────────────┘
```

#### Collider Component

Defines collision geometry for an entity:

```rust
#[derive(Component)]
pub enum Collider {
    Cuboid { hx: f32, hy: f32, hz: f32 },
    Sphere { radius: f32 },
    CapsuleY { half_height: f32, radius: f32 },
    CylinderY { half_height: f32, radius: f32 },
    // ... other shapes
}
```

**Shape characteristics**:

```text
Collision Shape Trade-offs
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌────────────┬────────────┬───────────┬──────────────────────┐
│   Shape    │Performance │ Accuracy  │    Best For          │
├────────────┼────────────┼───────────┼──────────────────────┤
│  Sphere    │  Fastest   │   Good    │ Balls, planets,      │
│            │            │           │ projectiles          │
│            │            │           │                      │
│  Cuboid    │  Fast      │   Good    │ Boxes, buildings,    │
│            │            │           │ platforms, walls     │
│            │            │           │                      │
│  Capsule   │  Fast      │   Great   │ Characters, pills,   │
│            │            │           │ standing objects     │
│            │            │           │                      │
│  Cylinder  │  Medium    │   Good    │ Wheels, barrels,     │
│            │            │           │ cylindrical objects  │
│            │            │           │                      │
│  Compound  │  Slow      │  Perfect  │ Complex shapes,      │
│            │            │           │ vehicles             │
└────────────┴────────────┴───────────┴──────────────────────┘

Dimensions: Half-Extents
━━━━━━━━━━━━━━━━━━━━━━━━
All dimensions are half-extents (distance from center):

Collider::cuboid(1.0, 2.0, 3.0)
  → Total size: 2.0 wide × 4.0 tall × 6.0 deep

Collider::sphere(0.5)
  → Total diameter: 1.0

Why half-extents? Makes math simpler and symmetric:
  - Center at origin
  - Bounds: [-hx to +hx, -hy to +hy, -hz to +hz]
  - Distance calculations use radius directly
```

#### PhysicsVelocity Component

Stores linear and angular velocity of a body:

```rust
#[derive(Component)]
pub struct PhysicsVelocity {
    /// Linear velocity (units/second)
    pub linear: Vec3,
    
    /// Angular velocity (radians/second)
    pub angular: Vec3,
}
```

**Physical meaning**:

```text
Velocity Interpretation
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Linear Velocity:
  Rate of change of position
  
  Vec3::new(5.0, 0.0, 0.0) = Moving 5 units/sec in +X direction
  Vec3::new(0.0, -9.8, 0.0) = Falling at 9.8 units/sec
  Vec3::ZERO = Not moving
  
  Speed = velocity.length()
  Direction = velocity.normalize()

Angular Velocity:
  Rate of change of orientation (axis-angle representation)
  
  Vec3::new(0.0, 3.14, 0.0) = Rotating around Y-axis at π rad/sec
                               (180° per second)
  Vec3::new(1.0, 0.0, 0.0) = Rotating around X-axis at 1 rad/sec
                              (~57° per second)
  
  Rotation speed = angular.length()
  Rotation axis = angular.normalize()
```

### Fixed Timestep Simulation

Physics simulation uses **fixed timestep integration** for deterministic behavior:

```text
Fixed Timestep Accumulator Pattern
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Problem: Frame rate varies (30fps, 60fps, 144fps, etc.)
         Physics needs constant dt for stability

Solution: Accumulate frame time, run physics at fixed rate

┌─────────────────────────────────────────────────────────────┐
│                 PhysicsTime Accumulator                      │
│                                                              │
│  accumulator += frame_delta_time                             │
│                                                              │
│  while accumulator >= fixed_timestep:                        │
│      run_one_physics_step()                                  │
│      accumulator -= fixed_timestep                           │
└─────────────────────────────────────────────────────────────┘

Timeline Example (60Hz physics, 1/60 = 0.0167s per step):

Frame 1: dt=16.7ms
  accumulator = 16.7ms
  Run 1 step (16.7ms)
  accumulator = 0ms

Frame 2: dt=16.7ms
  accumulator = 16.7ms
  Run 1 step (16.7ms)
  accumulator = 0ms

Frame 3: dt=33.4ms (slow frame!)
  accumulator = 33.4ms
  Run 2 steps (33.4ms)
  accumulator = 0ms

Frame 4: dt=8.3ms (fast frame)
  accumulator = 8.3ms
  Run 0 steps (not enough time)
  accumulator = 8.3ms (carry over)

Frame 5: dt=8.4ms
  accumulator = 16.7ms (8.3 + 8.4)
  Run 1 step (16.7ms)
  accumulator = 0ms
```

**Benefits**:
- **Determinism**: Same inputs → Same results
- **Stability**: Solver convergence guaranteed
- **Frame-rate independence**: Looks same at any FPS

**Trade-offs**:
- Slow frames may run multiple steps (performance spike)
- "Spiral of death" if physics can't keep up with real-time

### Transform Synchronization

The physics system maintains bidirectional synchronization:

```text
Bidirectional Transform Sync
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│                    Frame Timeline                            │
│                                                              │
│  1. User Input / Animation                                   │
│     └─► Modify ECS Transform components                      │
│                                                              │
│  2. PRE-PHYSICS SYNC (ECS → Physics)                         │
│     └─► sync_physics_transforms_system                       │
│         • Kinematic bodies: Copy Transform to Rapier         │
│         • Dynamic bodies: Copy if just spawned               │
│         • Static bodies: Copy if just spawned                │
│                                                              │
│  3. PHYSICS STEP                                             │
│     └─► physics_step_system                                  │
│         • Integrate forces (F = ma)                          │
│         • Detect collisions                                  │
│         • Resolve constraints                                │
│         • Update positions & velocities                      │
│                                                              │
│  4. POST-PHYSICS SYNC (Physics → ECS)                        │
│     └─► sync_physics_transforms_system                       │
│         • Dynamic bodies: Copy position from Rapier          │
│         • Kinematic bodies: Skip (controlled by ECS)         │
│         • Static bodies: Skip (never move)                   │
│                                                              │
│  5. Transform Propagation (if hierarchical)                  │
│     └─► Update GlobalTransform from hierarchy                │
│                                                              │
│  6. Rendering                                                │
│     └─► Draw objects at updated positions                    │
└─────────────────────────────────────────────────────────────┘
```

### Physics Pipeline Systems

The complete physics update requires these systems in order:

```rust
let mut schedule = Schedule::default();
schedule.add_systems((
    // 1. Clear previous frame's collision events
    clear_collision_event_receivers,
    
    // 2. Sync ECS changes to physics (kinematic movement, teleports)
    sync_physics_transforms_system,
    
    // 3. Run physics simulation (may run 0, 1, or multiple steps)
    physics_step_system,
    
    // 4. Sync physics results back to ECS (dynamic body movement)
    sync_physics_transforms_system,
    
    // 5. Distribute collision events to entities
    populate_collision_events,
).chain());
```

### Example Usage

```rust
// Static ground (never moves)
world.spawn((
    Transform::from_xyz(0.0, -1.0, 0.0),
    RigidBody::Static,
    Collider::cuboid(50.0, 0.5, 50.0),  // 100×1×100 platform
    Friction::new(0.5),
    Restitution::new(0.0),  // No bounce
));

// Dynamic falling cube (affected by gravity)
world.spawn((
    Transform::from_xyz(0.0, 10.0, 0.0),
    RigidBody::Dynamic,
    Collider::cuboid(1.0, 1.0, 1.0),  // 2×2×2 cube
    PhysicsVelocity::default(),
    Mass::new(1.0),
    Friction::new(0.6),
    Restitution::new(0.2),  // Slight bounce
));

// Bouncy sphere (high restitution)
world.spawn((
    Transform::from_xyz(0.0, 15.0, 0.0),
    RigidBody::Dynamic,
    Collider::sphere(0.5),  // Radius 0.5
    PhysicsVelocity::linear(Vec3::new(1.0, 0.0, 0.0)),  // Initial velocity
    Restitution::new(0.8),  // Very bouncy
    Friction::new(0.1),     // Low friction
));
```

For more details, see `examples/physics_demo.rs` which demonstrates falling cubes, bouncing spheres, and collision detection.

---

## Shadow Mapping System

Shadow mapping is one of the most important techniques for adding realism to 3D scenes. Understanding how shadows work helps you optimize performance and achieve the visual quality you need.

### What is Shadow Mapping?

Shadow mapping is a two-pass rendering technique that determines which parts of the scene are in shadow:

```text
Shadow Mapping Overview
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Pass 1: Shadow Pass (Render from Light's Perspective)
┌─────────────────────────────────────────────────────────────┐
│                    Light's View                              │
│                                                              │
│            ☀️ Light                                         │
│             │                                                │
│             │ Looking down                                   │
│             ▼                                                │
│        ┌────────┐                                            │
│        │ Object │                                            │
│        └────────┘                                            │
│                                                              │
│  Render scene to depth texture (shadow map):                │
│  - Each pixel stores distance from light                    │
│  - Only depth values matter (no colors)                     │
│  - This creates the "shadow map"                            │
└─────────────────────────────────────────────────────────────┘

Pass 2: Main Pass (Render from Camera's Perspective)
┌─────────────────────────────────────────────────────────────┐
│                  Camera's View                               │
│                                                              │
│  For each pixel:                                             │
│    1. Calculate distance from light                          │
│    2. Look up stored distance in shadow map                  │
│    3. Compare:                                               │
│       - If current distance > stored distance:               │
│         → Something else is closer to light                  │
│         → This pixel is IN SHADOW                            │
│       - If current distance ≈ stored distance:               │
│         → This pixel is DIRECTLY LIT                         │
│    4. Darken shadowed pixels                                 │
└─────────────────────────────────────────────────────────────┘

Visual Example:
                        ☀️ Light
                         │
                         ▼
                    ┌────────┐
                    │ Object │
                    └────────┘
                         │
                         │ Shadow
                         ▼
        ═══════════════════════ Ground

Shadow map stores: [distance to object, distance to ground, ...]
Main pass compares: "Is this ground pixel farther than object?"
                    → Yes → Pixel is in shadow!
```

### Cascaded Shadow Maps (CSM)

A single shadow map covering the entire scene would have poor resolution. CSM solves this by using multiple shadow maps at different distances:

```text
Cascaded Shadow Maps
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Camera View Frustum (side view):
                                    Far distance
                                    (500m)
                                    ▲
                                   /│\
                                  / │ \
                                 /  │  \
                                /   │   \
              Mid distance     /    │    \
              (100m)          /     │     \
              ▲              /      │      \
             /│\            /       │       \
            / │ \          /        │        \
           /  │  \        /         │         \
Near      /   │   \      /          │          \
distance /    │    \    /           │           \
(20m)   /     │     \  /            │            \
▲      /      │      \/             │             \
│     /       │      /\             │              \
│    /        │     /  \            │               \
│   /         │    /    \           │                \
│  /          │   /      \          │                 \
│ /           │  /        \         │                  \
│/            │ /          \        │                   \
📷──────────────────────────────────────────────────────▶
Camera        │              │               │

Cascade 0     Cascade 1      Cascade 2       Cascade 3
(0-20m)       (20-100m)      (100-500m)      (500m+)
High res      Medium res     Lower res       Lowest res
2048×2048     2048×2048      2048×2048       2048×2048

Same texture size, but covering different world space:
- Cascade 0: 1 pixel = ~0.01m (very detailed)
- Cascade 1: 1 pixel = ~0.05m (good detail)
- Cascade 2: 1 pixel = ~0.20m (moderate detail)
- Cascade 3: 1 pixel = ~1.0m (basic detail)

Result: Shadows look great near camera, acceptable far away!
```

### Shadow Map Resolution Trade-offs

```text
Resolution Impact
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

512×512 per cascade:
  Memory: 1 MB per cascade (3 cascades = 3 MB)
  Quality: Blocky shadows, visible aliasing
  Performance: Excellent (fast rendering, low memory)
  Use case: Mobile, low-end hardware

1024×1024 per cascade (default):
  Memory: 4 MB per cascade (3 cascades = 12 MB)
  Quality: Good shadows, minimal aliasing
  Performance: Good balance
  Use case: Desktop, general purpose

2048×2048 per cascade:
  Memory: 16 MB per cascade (3 cascades = 48 MB)
  Quality: Excellent shadows, smooth edges
  Performance: More expensive (4× render time vs 1024)
  Use case: High-end graphics, screenshots

4096×4096 per cascade:
  Memory: 64 MB per cascade (3 cascades = 192 MB)
  Quality: Ultra high quality
  Performance: Heavy (16× render time vs 1024)
  Use case: Cinematics, offline rendering
```

### PCF (Percentage Closer Filtering)

PCF softens shadow edges by sampling the shadow map multiple times:

```text
PCF Filtering
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Without PCF (1 sample):
  Shadow Map (zoomed in):
  ┌─┬─┬─┬─┬─┐
  │█│█│░│░│░│  █ = in shadow (depth fail)
  │█│█│░│░│░│  ░ = lit (depth pass)
  │█│█│░│░│░│
  │█│█│░│░│░│
  └─┴─┴─┴─┴─┘
  
  Result: Hard, pixelated edge
  ████░░░░
  ████░░░░  ← Jagged!
  ████░░░░

With PCF (9 samples = 3×3 grid):
  For each pixel, sample 9 nearby points:
  ┌───┬───┬───┐
  │ 1 │ 2 │ 3 │  Average results:
  │───┼───┼───│  4 shadowed + 5 lit = 0.44
  │ 4 │ X │ 5 │  
  │───┼───┼───│  Result: Smooth gradient
  │ 6 │ 7 │ 8 │  ████▓▓▒▒░░
  └───┴───┴───┘  ████▓▓▒▒░░  ← Smooth!
      └─ 9       ████▓▓▒▒░░

PCF Sample Count Trade-off:
  1 sample:  Hard shadows, best performance
  4 samples: Slight softening, ~2× cost
  9 samples: Smooth shadows, ~4× cost
  16 samples: Very smooth, ~8× cost
```

### Common Shadow Artifacts

#### Shadow Acne

```text
Shadow Acne Problem
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

What it looks like:
  Surface shows striped/dotted shadow pattern on itself
  ▓▒░▓▒░▓▒░  ← Self-shadowing artifacts
  ░▓▒░▓▒░▓
  ▓▒░▓▒░▓▒

Why it happens:
  Light → Surface
          │
    Shadow map stores depth with limited precision
          │
    When comparing, rounding errors make surface
    think it's behind itself!

Solution: Shadow Bias
  Add small offset to depth comparison
  
  bias = 0.001  ← Too small: still get acne
  bias = 0.005  ← Good balance (default)
  bias = 0.050  ← Too large: "Peter Panning" (see below)
```

#### Peter Panning

```text
Peter Panning Problem
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

What it looks like:
     ┌────┐
     │ OBJ│
     └────┘
        ↕️ Gap!
    ▓▓▓▓▓▓▓▓  ← Shadow floats above ground
  ════════════ Ground

Why it happens:
  Shadow bias too large pushes shadow away from object

Solution:
  Reduce bias value
  Use slope-scale bias (automatically adjusts based on surface angle)
```

### Shadow System Usage

```rust
use praxis_graphics::shadow::{ShadowMapManager, ShadowConfig};

// Create shadow manager with custom config
let config = ShadowConfig {
    shadow_map_size: 1024,
    cascade_count: 3,
    cascade_distances: [20.0, 100.0, 500.0, 1000.0],
    pcf_samples: 9,  // 3×3 filter for smooth shadows
    bias: 0.005,
};

let shadow_manager = ShadowMapManager::new(
    memory_allocator.clone(),
    config,
)?;

// Each frame: update with light direction
let light_dir = Vec3::new(0.3, -0.8, 0.5).normalize();
shadow_manager.update(light_dir, camera_view, camera_proj)?;

// Shadows are automatically used in rendering!
```

For more details, see [Shadow System Documentation](shadow_system.md).

---

## Normal Mapping

Normal mapping is a technique that adds surface detail without adding geometry. It works by perturbing the surface normal at each pixel to simulate bumps and crevices.

### What are Normals?

A normal is a vector perpendicular to a surface. It tells the lighting system which direction the surface is facing:

```text
Surface Normals
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Flat Surface:
         ▲ Normal
         │ (perpendicular to surface)
         │
    ─────┴───── Surface

All points have same normal → Looks flat


Curved Surface:
    ▲     ▲     ▲
     \    |    /   Each point has different normal
      \   |   /    → Looks curved
       \  |  /
        ╲│╱
      ══════════

Normals determine how light reflects off surface!
```

### Normal Maps Explained

A normal map is a special texture where RGB values encode normal vectors:

```text
Normal Map Encoding
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Color Channels → Normal Vector:
  R (Red):   X component of normal (-1 to +1)
  G (Green): Y component of normal (-1 to +1)
  B (Blue):  Z component of normal (-1 to +1)

Typical normal map appearance:
  Mostly blue-purple colors
  Blue means "pointing toward camera" (Z = +1)
  
  Example pixel colors:
  RGB(128, 128, 255) → Normal (0, 0, 1)    [straight up]
  RGB(255, 128, 128) → Normal (1, 0, 0)    [pointing right]
  RGB(128, 255, 128) → Normal (0, 1, 0)    [pointing up]
  RGB(200, 128, 180) → Normal (0.4, 0, -0.2) [slight bump]
```

### Tangent Space

Normal maps are stored in "tangent space" - a coordinate system local to each triangle:

```text
Tangent Space Coordinate System
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

World Space vs Tangent Space:

World Space (global):
         Y (up)
         │
         │
         └──── X
        /
       Z

Triangle in World Space:
         Normal ▲
                │
        ╱───────┤  ← Triangle
       ╱        │
      ╱         │

Tangent Space (local to triangle):
         N (Normal)
         │      B (Bitangent)
         │     ╱
         │    ╱
         │   ╱
         │  ╱
         │ ╱
         └──────── T (Tangent)

The tangent and bitangent align with the texture's U and V directions!

This means:
- Normal map can be reused on any surface
- (0, 0, 1) in tangent space always means "flat surface"
- Bumps rotate correctly with the surface
```

### How Normal Mapping Works

```text
Normal Mapping Pipeline
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Step 1: Vertex Shader
  Input per vertex:
  - Position
  - Normal (geometric)
  - Tangent (from mesh)
  - UV coordinates
  
  Output per vertex:
  - Normal, Tangent, Bitangent (interpolated)
  - UV coordinates

Step 2: Fragment Shader
  For each pixel:
  
  1. Sample normal map at UV coordinate
     normal_map_color = texture(normal_map, uv)
     
  2. Convert color [0,1] to normal [-1,1]
     tangent_normal = normal_map_color * 2.0 - 1.0
     
  3. Build TBN matrix (Tangent, Bitangent, Normal)
     TBN = mat3(T, B, N)
     
  4. Transform from tangent space to world space
     world_normal = TBN * tangent_normal
     
  5. Use world_normal for lighting calculations
     Instead of flat geometric normal!

Result:
  Without normal map:
    ──────────  ← Looks flat, single normal
  
  With normal map:
    ╱╲╱╲╱╲╱╲  ← Looks bumpy, varied normals
                   (but still same geometry!)
```

### Normal Map Benefits

```text
Geometry vs Normal Maps
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Brick Wall Example:

High-Poly Geometry (1 million triangles):
  Pros: Perfect accuracy
  Cons: 
    - 1,000,000 vertices to process
    - Massive memory usage
    - Slow rendering
    - Impractical for games

Low-Poly + Normal Map (1000 triangles):
  Pros:
    - 1,000 vertices to process (1000× faster!)
    - Small memory footprint
    - Fast rendering
    - Looks almost as good
  Cons:
    - Silhouette is still flat
    - Can't cast detailed shadows

Best Practice:
  Use normal maps for:
  - Small surface details (scratches, bumps, tiles)
  - Repeated patterns
  - Close-up detail
  
  Use geometry for:
  - Silhouettes
  - Large features
  - Anything that needs to cast shadows
```

### Using Normal Maps in Praxis

```rust
// Normal maps are loaded like regular textures
render_context
    .texture_manager_mut()
    .load_texture(
        "brick_normals",
        "assets/textures/brick_normal.png"
    )?;

// The graphics system automatically uses them if:
// 1. Material has normal_map_texture set
// 2. Mesh has tangent vectors
// 3. Shader supports normal mapping

let material = MaterialProperties {
    base_color_texture: Some("brick_color".to_string()),
    normal_map_texture: Some("brick_normals".to_string()),
    metallic: 0.0,
    roughness: 0.8,
};
```

---

## Post-Processing Pipeline

Post-processing applies effects to the final rendered image before display. It's the last stage of rendering and can dramatically change the look of your game.

### Post-Processing Flow

```text
Complete Rendering Pipeline with Post-Processing
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│  Step 1: 3D Scene Rendering                                  │
│  ────────────────────────────────────────────────────────    │
│  Render scene to offscreen texture (not screen!)             │
│                                                              │
│  Input: 3D geometry, textures, lights                        │
│  Output: Color texture (scene_color)                         │
│                                                              │
│  ┌──────────────────────────────────────────┐               │
│  │  ┌─────┐  ┌─────┐  ┌─────┐               │               │
│  │  │ Obj │  │ Obj │  │Light│               │               │
│  │  └─────┘  └─────┘  └─────┘               │               │
│  │         Rendered Scene                   │               │
│  └──────────────────────────────────────────┘               │
│              │                                               │
│              │ Stored in offscreen texture                   │
│              ▼                                               │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  Step 2: Post-Processing Chain                               │
│  ────────────────────────────────────────────────────────    │
│  Apply effects in sequence, ping-ponging between textures    │
│                                                              │
│  Pass 1: Bloom Extraction                                    │
│  ┌──────────┐                                                │
│  │scene_────│ → Extract Bright → ┌──────────┐               │
│  │ color    │    Pixels           │ bright   │               │
│  └──────────┘                     └──────────┘               │
│                                         │                    │
│  Pass 2: Blur Horizontal                │                    │
│  ┌──────────┐                           │                    │
│  │ bright   │ ◄──────────────────────────┘                   │
│  └──────────┘ → Blur H → ┌──────────┐                        │
│                           │blur_h    │                        │
│                           └──────────┘                        │
│                                │                             │
│  Pass 3: Blur Vertical         │                             │
│  ┌──────────┐                  │                             │
│  │ blur_h   │ ◄─────────────────┘                            │
│  └──────────┘ → Blur V → ┌──────────┐                        │
│                           │blur_final│                        │
│                           └──────────┘                        │
│                                │                             │
│  Pass 4: Combine               │                             │
│  ┌──────────┐                  │                             │
│  │scene_────│ ◄─────────┐      │                             │
│  │ color    │            │      │                             │
│  └──────────┘            │      │                             │
│  ┌──────────┐            │      │                             │
│  │blur_final│ ◄──────────┴──────┘                            │
│  └──────────┘                                                 │
│       │                                                       │
│       │ Combine                                               │
│       ▼                                                       │
│  ┌──────────┐                                                 │
│  │  final   │                                                 │
│  │  output  │                                                 │
│  └──────────┘                                                 │
└──────────────┬──────────────────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────┐
│  Step 3: Present to Screen                                   │
│  ────────────────────────────────────────────────────────    │
│  Copy final texture to swapchain image (visible on screen)   │
│                                                              │
│  🖥️ Display                                                 │
└─────────────────────────────────────────────────────────────┘
```

### Why Offscreen Rendering?

```text
Screen vs Offscreen Rendering
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Direct to Screen (No Post-Processing):
  3D Scene → Swapchain → Display
  
  ✓ Simple
  ✓ Fast
  ✗ Can't apply effects
  ✗ Can't read back results
  ✗ Limited to screen resolution

To Offscreen Texture (With Post-Processing):
  3D Scene → Texture A → Process → Texture B → Swapchain → Display
  
  ✓ Can apply effects
  ✓ Can read/write multiple times
  ✓ Can render at different resolution
  ✓ Can save frames to disk
  ✗ Extra memory for textures
  ✗ Slightly more complex

Memory Cost Example (1080p):
  Color texture: 1920 × 1080 × 4 bytes = 8.3 MB
  Depth texture: 1920 × 1080 × 4 bytes = 8.3 MB
  Intermediate: 1920 × 1080 × 4 bytes = 8.3 MB
  Total: ~25 MB (minimal on modern GPUs)
```

### Ping-Pong Rendering

Many post-processing effects require multiple passes. Ping-pong rendering efficiently handles this:

```text
Ping-Pong Pattern
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Two Render Targets: A and B

Frame Start:
  Input: Scene in Texture A
  
Pass 1: Blur Horizontal
  Read from: Texture A
  Write to:  Texture B
  
Pass 2: Blur Vertical
  Read from: Texture B  ← Now B has the latest image
  Write to:  Texture A  ← Reuse A (not needed anymore)
  
Pass 3: Sharpen
  Read from: Texture A  ← Now A has the latest image
  Write to:  Texture B  ← Reuse B
  
Final Pass: Copy to Screen
  Read from: Texture B
  Write to:  Swapchain
  
Memory: Only 2 textures needed regardless of pass count!
        Without ping-pong: would need N textures for N passes
```

### Common Post-Processing Effects

```text
Post-Processing Effect Gallery
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Bloom (Glow Effect):
  Original:    ████▓▓▒▒░░      💡
  With Bloom:  ▓▓▓▓▓▓▓▒▒░  ← Light bleeds
  
  Steps:
    1. Extract bright pixels (threshold)
    2. Blur heavily (spread light)
    3. Add back to original
  
  Cost: ~1-2ms @ 1080p (3-5 passes)

Color Grading:
  Original:  Regular colors
  Graded:    Adjusted mood (warm, cool, vintage, etc.)
  
  Implementation: LUT (Look-Up Table) texture
  Cost: ~0.1ms @ 1080p (single texture lookup)

Depth of Field:
  Original:  Everything sharp
  DOF:       Focus on subject, blur background
  
  Requires: Depth buffer from rendering
  Cost: ~2-4ms @ 1080p (distance-based blur)

Motion Blur:
  Original:  Crisp movement
  Motion:    Trails behind moving objects
  
  Requires: Velocity buffer (previous frame positions)
  Cost: ~1-2ms @ 1080p

Screen Space Ambient Occlusion (SSAO):
  Original:  Flat ambient lighting
  SSAO:      Darkened corners and crevices
  
  Requires: Depth buffer + normals
  Cost: ~2-3ms @ 1080p (multiple depth samples)

Tone Mapping:
  Original:  HDR values (can be > 1.0)
  Mapped:    Compressed to screen range [0, 1]
  
  Purpose: Convert high dynamic range to displayable
  Cost: ~0.1ms @ 1080p (per-pixel formula)
```

### Post-Processing Best Practices

```text
Optimization Guidelines
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. Order Matters
   Fast to Slow:
     ✓ Cheap effects first (color grading)
     ✓ Expensive effects last (heavy blur)
   
   Reason: If early effect makes pixel transparent,
           later effects don't need to process it

2. Resolution Scaling
   High-frequency detail (SSAO):  Full resolution
   Low-frequency effects (bloom): Half resolution
   
   Example:
     Scene render: 1920×1080
     Bloom extract: 960×540  (¼ pixels = 4× faster!)
     Bloom blur: 960×540
     Final combine: 1920×1080

3. Render Target Pooling
   Reuse textures between frames:
   
   Frame N:   Create texture A  (1.5ms)
              Create texture B  (1.5ms)
              Total: 3ms
   
   Frame N+1: Reuse texture A   (0.01ms)
              Reuse texture B   (0.01ms)
              Total: 0.02ms  (150× faster!)

4. Batch Effects
   Bad:  Submit each pass separately
         (CPU overhead per submission)
   
   Good: Record all passes in one command buffer
         (Single submission)

5. Test on Target Hardware
   Desktop GPU:  Can handle 10+ effects
   Mobile GPU:   May struggle with 3+ effects
   
   Always profile!
```

### Using Post-Processing in Praxis

```rust
use praxis_graphics::post_processing::{
    PostProcessChain, RenderTargetPool, GrayscalePass
};

// Setup (once)
let mut pool = RenderTargetPool::new(memory_allocator.clone(), render_pass.clone());
let mut chain = PostProcessChain::new(queue.clone());

// Add effects
chain.add_pass(Box::new(GrayscalePass::new(
    device.clone(),
    render_pass.clone(),
)?));

// Each frame
let input = pool.acquire([width, height])?;
let output = pool.acquire([width, height])?;

// Render scene to input texture
// ... (your 3D rendering code)

// Apply post-processing
chain.process(&input, &output, &mut pool)?;

// Present output to screen
// ...

pool.release(input);
pool.release(output);
```

For detailed information, see [Post-Processing System Documentation](post_processing_system.md).

---

## Particle System

The particle system brings dynamic visual effects to your game: fire, smoke, explosions, magic spells, sparks, and more. Understanding how particles work helps you create compelling visual feedback and atmosphere.

### What are Particles?

Particles are small, simple objects (usually billboarded quads) rendered in large quantities to create complex visual effects:

```text
Particle System Concept
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fire Effect = Hundreds of Small Particles

Each particle:
  ▪️ Small colored quad
  🔴 Position
  🎨 Color (changes over time)
  📏 Size (changes over time)
  🔄 Rotation
  ⏱️ Lifetime (dies after X seconds)
  🚀 Velocity (moves each frame)

Together they create:
    🔥🔥🔥
   🔥🔥🔥🔥
  🔥🔥🔥🔥🔥  ← Looks like continuous fire!
   🔥🔥🔥🔥      But actually hundreds of particles
    🔥🔥🔥       being spawned, updated, and dying
```

### Particle System Architecture

```text
Particle System Flow
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│                     Particle Emitter                         │
│                                                              │
│  Configuration:                                              │
│  - Emission rate: 50 particles/second                       │
│  - Lifetime: 2.0 seconds                                     │
│  - Initial velocity: (0, 5, 0) ± randomness                 │
│  - Color gradient: yellow → orange → red → black            │
│  - Size curve: small → large → small                        │
│  - Forces: gravity, wind, drag                              │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼ Spawns particles
┌─────────────────────────────────────────────────────────────┐
│                    Particle Pool (CPU)                       │
│  [10,000 particle slots]                                     │
│                                                              │
│  Active particles:                                           │
│  Particle 0: pos(1,2,3), vel(0,3,0), life=1.5s, color=🔴    │
│  Particle 1: pos(2,3,1), vel(1,2,0), life=0.8s, color=🟠    │
│  Particle 2: pos(1,4,2), vel(0,4,0), life=1.2s, color=🔴    │
│  ... (100 active)                                            │
│                                                              │
│  Inactive particles: [Particle 101..9999]                   │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼ Each frame: Update positions,
                           │            Apply forces,
                           │            Check lifetimes
┌─────────────────────────────────────────────────────────────┐
│                Instance Buffer (GPU)                         │
│  [Upload only active particles]                             │
│                                                              │
│  ParticleInstance 0: {pos, color, size, rotation}           │
│  ParticleInstance 1: {pos, color, size, rotation}           │
│  ParticleInstance 2: {pos, color, size, rotation}           │
│  ... (100 instances = 3.6 KB)                               │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼ GPU Instancing
┌─────────────────────────────────────────────────────────────┐
│                  GPU Rendering                               │
│                                                              │
│  Quad geometry (4 vertices, 6 indices) ×100 instances       │
│  = Single draw call renders all 100 particles!              │
│                                                              │
│  Vertex shader: Transform quad → billboard facing camera    │
│  Fragment shader: Sample texture, apply color               │
└─────────────────────────────────────────────────────────────┘
```

### Emitter Shapes

Particles can spawn from various shapes:

```text
Emitter Shape Visualization
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Point:
      ●  ← All particles spawn here
    🔴🔴🔴
   🔴🔴🔴🔴
  🔴🔴🔴🔴🔴

Sphere (radius):
      ╱●●●╲   Particles spawn on surface
     ● 🔴🔴 ●  or within volume
    ●  🔴🔴  ●
     ● 🔴🔴 ●
      ╲●●●╱

Box (extents):
    ┌───────┐
    │ 🔴🔴🔴 │  Particles spawn anywhere
    │🔴🔴🔴🔴│  within the box
    │ 🔴🔴🔴 │
    └───────┘

Circle (radius):
       🔴
    🔴     🔴    Particles spawn on
   🔴   ●   🔴   the ring edge
    🔴     🔴
       🔴

Cone (radius, angle):
       ●
      🔴🔴      Particles spawn within
     🔴🔴🔴🔴    the cone volume
    🔴🔴🔴🔴🔴
```

### Particle Properties

Each particle tracks multiple properties that change over time:

```text
Particle Lifecycle
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Time:     0.0s           0.5s          1.0s          1.5s    2.0s
          (spawn)                                             (death)

Position: (0,0,0) ----→ (0,2,0) ---→ (0,5,0) ---→ (0,7,0) → (0,8,0)
          ↑ Apply forces each frame: gravity, wind, drag

Velocity: (0,5,0) ----→ (0,4,0) ---→ (0,3,0) ---→ (0,2,0) → (0,1,0)
          ↑ Slows down due to drag force

Color:    🟡 yellow --→ 🟠 orange --→ 🔴 red ---→ ⚫ black → ⬛ dead
          [1,1,0,1]     [1,0.5,0,1]   [1,0,0,0.5] [0,0,0,0]
          ↑ Interpolated from color gradient

Size:     ▪️ 0.2 -----→ ● 0.8 -----→ ⬤ 1.2 ----→ ● 0.8 ---→ ▪️ 0.4
          ↑ Grows then shrinks (size curve)

Rotation: 0° --------→ 60° -------→ 120° ------→ 180° ---→ 240°
          ↑ Constant rotation speed (2 rad/s)

Lifetime: 2.0s ------→ 1.5s ------→ 1.0s ------→ 0.5s ---→ 0.0s
          ↑ Decrements each frame, dies when <= 0
```

### Physical Forces

Forces make particles move realistically:

```text
Force Types
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Gravity:
          🔴
        🔴       Constant downward force
      🔴         (or any direction)
    🔴
  🔴             vel += gravity * dt
             
  Example: Fire goes up → gravity pulls down → curved motion

Wind:
  🔴 🔴 🔴 →→→   Directional force with
 🔴 🔴 🔴 →→→    optional turbulence
🔴 🔴 🔴 →→→     (random variation)
             
  Example: Smoke drifts sideways with random fluctuation

Attraction:
     🔴🔴           Particles pulled toward
    🔴 ● 🔴         a point (like a magnet)
     🔴🔴       
  ↑ attractor
             
  Example: Magic spell gathering energy to center

Radial (Push/Pull):
     🔴←●→🔴        Particles pushed away from
       🔴          (or pulled toward) origin
       🔴      
             
  Example: Explosion - particles blast outward from center

Drag:
  🔴→→→          Air resistance
   🔴→→          slows particles down
    🔴→      
      🔴         vel *= (1 - drag * dt)
             
  Example: Sparks slow down quickly after initial burst
```

### Color and Size Over Lifetime

Gradients create dynamic visual changes:

```text
Fire Particle Example
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Lifetime:    0%        25%        50%        75%       100%
             │         │          │          │          │
Color:       🟡 ----→ 🟠 ------→ 🔴 ------→ 🔴 -----→ ⚫
             bright   orange     red        dark red   black
             yellow                                     (fade)

Size:        ▪️ -----→ ● ------→ ⬤ ------→ ● ------→ ▪️
             0.1       0.5        1.0        0.7        0.3
             (small)  (growing)  (peak)     (shrink)   (tiny)

Configuration:
  color_over_lifetime: [
    [1.0, 1.0, 0.2, 1.0],  // 0%: bright yellow
    [1.0, 0.5, 0.0, 0.9],  // 33%: orange
    [1.0, 0.0, 0.0, 0.6],  // 66%: red
    [0.2, 0.0, 0.0, 0.0],  // 100%: fade to black
  ]
  
  size_over_lifetime: [0.1, 0.5, 1.0, 0.7, 0.3]

Result: Particles start small and bright, grow and turn orange,
        peak in size as red, then shrink and fade to black
```

### GPU Instancing

Efficient rendering of thousands of particles:

```text
Instancing Explained
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Without Instancing (Bad):
  For each particle:
    1. Upload vertex data (4 vertices)
    2. Issue draw call
    3. GPU processes 4 vertices
  
  1000 particles = 1000 draw calls (SLOW!)
  CPU spends most time on draw call overhead

With Instancing (Good):
  Upload once:
    - Quad geometry (4 vertices, 6 indices)
    - Instance buffer (1000 particle transforms)
  
  Issue one draw call:
    glDrawElementsInstanced(6 indices, 1000 instances)
  
  GPU automatically:
    - Replicates quad 1000 times
    - Applies per-instance data to each
  
  1000 particles = 1 draw call (FAST!)
  CPU overhead is minimal

Per-Instance Data (36 bytes):
  vec3 position    (12 bytes)
  vec4 color       (16 bytes)
  float size       (4 bytes)
  float rotation   (4 bytes)

1000 particles = 36 KB uploaded per frame (tiny!)
```

### Usage Example

```rust
use praxis_graphics::{
    ParticleSystem, ParticleEmitterConfig,
    EmitterShape, ParticleForce
};
use praxis_math::Vec3;

// Create particle system
let mut particle_system = ParticleSystem::new(
    memory_allocator,
    command_buffer_allocator,
    queue,
)?;

// Configure fire emitter
let fire_config = ParticleEmitterConfig {
    shape: EmitterShape::Sphere { radius: 0.5 },
    emission_rate: 50.0,              // 50 particles/second
    max_particles: 500,                // Pool size
    particle_lifetime: 2.0,            // Lives 2 seconds
    lifetime_randomness: 0.3,          // ±0.3 seconds
    
    initial_velocity: Vec3::new(0.0, 3.0, 0.0),  // Upward
    velocity_randomness: 1.0,                     // ±1.0 variation
    
    initial_color: [1.0, 0.8, 0.2, 1.0],         // Bright yellow
    color_over_lifetime: Some(vec![
        [1.0, 0.8, 0.2, 1.0],  // Yellow start
        [1.0, 0.3, 0.0, 0.8],  // Orange middle
        [0.5, 0.0, 0.0, 0.3],  // Dark red
        [0.1, 0.0, 0.0, 0.0],  // Fade out
    ]),
    
    initial_size: 0.3,
    size_over_lifetime: Some(vec![0.1, 0.5, 0.8, 0.4]),
    size_randomness: 0.1,
    
    rotation_speed: 2.0,              // 2 radians/second
    rotation_speed_randomness: 1.0,    // ±1.0 variation
    
    forces: vec![
        ParticleForce::Gravity {
            strength: Vec3::new(0.0, 1.0, 0.0),  // Upward (hot air)
        },
        ParticleForce::Wind {
            direction: Vec3::new(1.0, 0.0, 0.0),
            strength: 0.5,
            turbulence: 0.3,          // Random wobble
        },
        ParticleForce::Drag {
            coefficient: 0.5,         // Air resistance
        },
    ],
    
    looping: true,                    // Continuous emission
    ..Default::default()
};

particle_system.add_emitter("campfire", fire_config);

// Position the emitter
if let Some(emitter) = particle_system.get_emitter_mut("campfire") {
    emitter.set_position(Vec3::new(0.0, 0.0, 0.0));
}

// Each frame:
let delta_time = 0.016;  // ~60 FPS
particle_system.update(delta_time);
particle_system.prepare_render()?;

// Rendering (integrate with your pipeline):
// let instances = particle_system.instance_buffer();
// let quad_verts = particle_system.quad_vertex_buffer();
// let quad_indices = particle_system.quad_index_buffer();
// Draw instanced with these buffers
```

### ECS Integration

Attach particle emitters to entities:

```rust
use praxis_ecs::{World, Transform, ParticleEmitter};

let mut world = World::new();

// Torch entity with particle emitter
world.spawn((
    Transform::from_xyz(5.0, 1.0, 0.0),
    ParticleEmitter::new("torch_fire"),
));

// The emitter follows the entity's transform
// Useful for: torches, engines, magic effects, etc.
```

### Performance Considerations

```text
Particle Count Guidelines
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Total Active Particles vs Frame Time:
  
  100 particles:   ~0.1ms  (negligible)
  1,000 particles: ~0.5ms  (good)
  5,000 particles: ~2ms    (acceptable for heavy effects)
  10,000 particles: ~5ms   (maximum per emitter)
  50,000 particles: ~25ms  (avoid! will drop frames)

Optimization Tips:

1. Emission Rate × Lifetime = Steady State Count
   - 50/sec × 2s = ~100 particles
   - 100/sec × 5s = ~500 particles
   - Plan accordingly!

2. Cull Distant Emitters
   - Don't update particles 100 meters away
   - Save CPU time for visible effects

3. Use Looping Wisely
   - Continuous effects: looping = true (fire, smoke)
   - One-shot effects: looping = false (explosion, splash)

4. Texture Atlases
   - Pack multiple sprites in one texture
   - Reduces texture swaps during rendering

5. Batch Similar Emitters
   - System renders all particles in one draw call
   - Very efficient!
```

### Common Effects Recipes

```rust
// Smoke
let smoke = ParticleEmitterConfig {
    shape: EmitterShape::Point,
    emission_rate: 20.0,
    particle_lifetime: 4.0,
    initial_velocity: Vec3::new(0.0, 1.0, 0.0),
    initial_color: [0.5, 0.5, 0.5, 0.5],
    color_over_lifetime: Some(vec![
        [0.5, 0.5, 0.5, 0.5],  // Gray
        [0.3, 0.3, 0.3, 0.1],  // Fading
        [0.2, 0.2, 0.2, 0.0],  // Transparent
    ]),
    size_over_lifetime: Some(vec![0.3, 1.0, 1.5]),
    forces: vec![
        ParticleForce::Wind {
            direction: Vec3::new(1.0, 0.5, 0.0),
            strength: 1.0,
            turbulence: 0.8,
        },
    ],
    ..Default::default()
};

// Explosion
let explosion = ParticleEmitterConfig {
    shape: EmitterShape::Sphere { radius: 0.2 },
    emission_rate: 200.0,
    particle_lifetime: 1.5,
    initial_velocity: Vec3::ZERO,
    velocity_randomness: 5.0,  // High randomness = radial burst
    initial_color: [1.0, 1.0, 0.5, 1.0],  // Bright yellow
    forces: vec![
        ParticleForce::Radial {
            origin: Vec3::ZERO,
            strength: 10.0,  // Push outward
        },
        ParticleForce::Gravity {
            strength: Vec3::new(0.0, -9.8, 0.0),  // Fall down
        },
    ],
    looping: false,     // One-shot effect
    duration: 0.2,      // Emit for 0.2 seconds then stop
    ..Default::default()
};

// Magic Sparkles
let sparkles = ParticleEmitterConfig {
    shape: EmitterShape::Sphere { radius: 1.0 },
    emission_rate: 30.0,
    particle_lifetime: 1.0,
    initial_velocity: Vec3::ZERO,
    velocity_randomness: 0.5,
    initial_color: [0.5, 0.5, 1.0, 1.0],  // Blue
    size_over_lifetime: Some(vec![0.1, 0.3, 0.1]),
    rotation_speed: 5.0,
    forces: vec![
        ParticleForce::Attraction {
            position: Vec3::ZERO,
            strength: 2.0,
            radius: 5.0,
        },
    ],
    ..Default::default()
};
```

### Debugging Particles

```text
Common Issues and Solutions
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Issue: Particles not appearing
  ✓ Check emission_rate > 0
  ✓ Check particle_lifetime > 0
  ✓ Check emitter.is_active() == true
  ✓ Check max_particles is sufficient
  ✓ Verify emitter position is in view

Issue: Particles disappear instantly
  ✓ Check lifetime isn't too short
  ✓ Verify forces aren't pushing particles away too fast
  ✓ Check color alpha doesn't fade to 0 immediately

Issue: Performance problems
  ✓ Count active particles (call total_active_particles())
  ✓ Reduce emission_rate
  ✓ Shorten particle_lifetime
  ✓ Use fewer emitters
  ✓ Reduce force complexity

Issue: Particles look wrong
  ✓ Verify color gradients are correct
  ✓ Check size curve values
  ✓ Test with simpler config first
  ✓ Visualize one emitter at a time
```

For complete details, see [Particle System Documentation](particle_system.md).

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
