# DynamicUniformBuffer Integration Plan (Refined)

**Status**: Ready for Implementation
**Priority**: HIGH
**Goal**: Eliminate per-object buffer allocations in the render loop by integrating the existing `DynamicUniformBuffer` module.

---

## Current State Analysis

### What Exists (Unused)
- **File**: `crates/praxis_graphics/src/uniform_buffer.rs` (258 lines)
- **NOT exported** in `lib.rs` - the module is complete but invisible
- Already has:
  - `ViewProjectionUniforms` - view and proj matrices (128 bytes)
  - `ModelUniforms` - model matrix (64 bytes)
  - `DynamicUniformBuffer` - complete ring buffer with proper alignment

### What's Currently Happening (Wasteful)
- **File**: `crates/praxis_graphics/src/lib.rs:804-817`
- Each object creates a NEW `Buffer::from_data()` call per frame
- 100 objects = 100 buffer allocations per frame
- Descriptor sets are also created per-object per-frame

### What's Missing
1. `camera_position` in `ViewProjectionUniforms` (needed for specular lighting)
2. Module export in `lib.rs`
3. Integration into `RenderContext` struct
4. Render loop refactor to use dynamic offsets

---

## PREREQUISITE: Fix physics_demo.rs

Before any work begins, `physics_demo.rs` must be fixed to compile. It uses removed API types.

**File**: `examples/physics_demo.rs`

**Changes**:
1. Line 30: Change `MeshRenderCommands` → `RenderCommands`
2. Lines 549-557: Update struct construction
3. Line 560: Change `render_meshes()` → `render()`

---

## TASK 1: Extend ViewProjectionUniforms for Camera Position

**File**: `crates/praxis_graphics/src/uniform_buffer.rs`

**Current** (lines 40-47):
```rust
pub struct ViewProjectionUniforms {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
}
```

**Change to**:
```rust
/// Uniforms for view, projection, and camera data (shared across all objects in a frame).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewProjectionUniforms {
    /// Camera view matrix (world → view).
    pub view: [[f32; 4]; 4],
    /// Camera projection matrix (view → clip).
    pub proj: [[f32; 4]; 4],
    /// Camera position in world space (for specular lighting).
    pub camera_position: [f32; 3],
    /// Padding for std140 alignment (vec3 must be 16-byte aligned).
    pub _padding: f32,
}
```

**Size Change**: 128 → 144 bytes

**Add method** to extract camera position from view matrix:
```rust
impl ViewProjectionUniforms {
    /// Creates uniforms from matrices, extracting camera position from view matrix inverse.
    pub fn new(view: Mat4, proj: Mat4) -> Self {
        // Camera position is the translation component of the inverted view matrix
        let view_inverse = view.inverse();
        let camera_pos = view_inverse.col(3).truncate();

        Self {
            view: view.to_cols_array_2d(),
            proj: proj.to_cols_array_2d(),
            camera_position: camera_pos.to_array(),
            _padding: 0.0,
        }
    }
}
```

---

## TASK 2: Update Shaders for Dynamic Camera Position

### Fragment Shader
**File**: `crates/praxis_graphics/src/shaders/triangle.frag`

**Step 1**: Add camera position to uniforms block. Find the existing Uniforms at set 0, binding 0 (referenced from vertex shader) and access it in fragment shader.

Actually, the current setup has uniforms only in vertex shader. We need to either:
- **Option A**: Pass camera position as varying from vertex shader (simpler)
- **Option B**: Share uniform buffer between vertex and fragment shaders (cleaner)

**Recommended: Option B** - Fragment shader already uses set 0 for other bindings.

**Remove** lines 318-320:
```glsl
// DELETE THESE LINES:
// Camera position in world space (temporary fixed position)
// TODO: Pass this via uniform buffer for dynamic camera
const vec3 CAMERA_POS = vec3(0.0, 5.0, 10.0);
```

**Add** uniform access (after other bindings, before line 317):
```glsl
// Camera uniforms (shared with vertex shader)
// This accesses the same uniform buffer as the vertex shader
layout(set = 0, binding = 0, std140) uniform Uniforms {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec3 camera_position;
    float _padding;
} uniforms;
```

**Change** line 443 from:
```glsl
vec3 view_dir = normalize(CAMERA_POS - v_world_pos);
```
To:
```glsl
vec3 view_dir = normalize(uniforms.camera_position - v_world_pos);
```

### Vertex Shader
**File**: `crates/praxis_graphics/src/shaders/triangle.vert`

**Update** Uniforms block (lines 144-148) to match:
```glsl
layout(set = 0, binding = 0, std140) uniform Uniforms {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec3 camera_position;
    float _padding;
} ubo;
```

---

## TASK 3: Update Rust Uniforms Struct

**File**: `crates/praxis_graphics/src/lib.rs`

**Current** (lines 188-192):
```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}
```

**Change to**:
```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    camera_position: [f32; 3],
    _padding: f32,
}
```

**Update usage** in render function (around line 798-802):
```rust
// Extract camera position from view matrix
let view_inverse = cmds.view.inverse();
let camera_pos = view_inverse.col(3).truncate();

let uniforms = Uniforms {
    model: draw_cmd.model.to_cols_array_2d(),
    view: cmds.view.to_cols_array_2d(),
    proj: cmds.proj.to_cols_array_2d(),
    camera_position: camera_pos.to_array(),
    _padding: 0.0,
};
```

**Note**: This is an intermediate step. After dynamic buffer integration, the uniforms will be split between ViewProjectionUniforms (once per frame) and ModelUniforms (per object).

---

## TASK 4: Export uniform_buffer Module

**File**: `crates/praxis_graphics/src/lib.rs`

**Add** module declaration (around line 15-20 with other modules):
```rust
pub mod uniform_buffer;
```

**Add** to public exports (around line 160-180):
```rust
pub use uniform_buffer::{DynamicUniformBuffer, ModelUniforms, ViewProjectionUniforms};
```

---

## TASK 5: Add DynamicUniformBuffer to RenderContext

**File**: `crates/praxis_graphics/src/lib.rs`

**Find** the `RenderContext` struct definition and add field:
```rust
/// Dynamic uniform buffer for per-object model matrices
dynamic_uniform_buffer: DynamicUniformBuffer,
```

**Update** `RenderContext::new()` to initialize:
```rust
// After memory_allocator is created, before returning:
let dynamic_uniform_buffer = DynamicUniformBuffer::new(
    &device,
    memory_allocator.clone(),
    3,    // frames in flight (matches swapchain image count)
    1024, // max objects per frame
)?;
```

**Add accessor method**:
```rust
/// Returns a reference to the dynamic uniform buffer.
pub fn dynamic_uniform_buffer(&self) -> &DynamicUniformBuffer {
    &self.dynamic_uniform_buffer
}
```

---

## TASK 6: Create Frame-Level ViewProjection Descriptor Set

The current system creates descriptor sets per-object. We need a separate descriptor set for view/projection that's bound once per frame.

**File**: `crates/praxis_graphics/src/lib.rs`

**Add to RenderContext**:
```rust
/// Descriptor set layout for per-frame view/projection uniforms
view_proj_descriptor_set_layout: Arc<DescriptorSetLayout>,
/// Buffer for per-frame view/projection uniforms
view_proj_buffer: Subbuffer<ViewProjectionUniforms>,
```

**Create layout** in `RenderContext::new()`:
```rust
let view_proj_descriptor_set_layout = DescriptorSetLayout::new(
    device.clone(),
    DescriptorSetLayoutCreateInfo {
        bindings: [(
            0,
            DescriptorSetLayoutBinding {
                stages: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                descriptor_type: DescriptorType::UniformBuffer,
                descriptor_count: 1,
                ..Default::default()
            },
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    },
)?;
```

---

## TASK 7: Refactor render() to Use Dynamic Offsets

This is the core change. The render function needs to:
1. Write view/proj once per frame
2. Write all model matrices to dynamic buffer
3. Use dynamic offsets when binding descriptor sets

**File**: `crates/praxis_graphics/src/lib.rs`

**Replace** the per-object buffer allocation loop with:

```rust
pub fn render(&mut self, cmds: &RenderCommands) -> Result<()> {
    // ... existing frame setup code ...

    // Advance to next frame in ring buffer
    self.dynamic_uniform_buffer.next_frame();

    // Extract camera position and create view/proj uniforms
    let view_inverse = cmds.view.inverse();
    let camera_pos = view_inverse.col(3).truncate();
    let view_proj = ViewProjectionUniforms {
        view: cmds.view.to_cols_array_2d(),
        proj: cmds.proj.to_cols_array_2d(),
        camera_position: camera_pos.to_array(),
        _padding: 0.0,
    };

    // Write view/proj buffer once per frame
    self.view_proj_buffer.write()?.clone_from(&view_proj);

    // Collect all model matrices
    let models: Vec<Mat4> = cmds.draw_commands
        .iter()
        .map(|cmd| cmd.model)
        .collect();

    // Write all models to dynamic buffer at once
    self.dynamic_uniform_buffer.write_models(&models)?;

    // ... sorting and material handling stays similar ...

    // In the draw loop, instead of creating new buffers:
    for (i, (original_index, draw_cmd)) in indexed_commands.iter().enumerate() {
        // Get dynamic offset for this object
        let dynamic_offset = self.dynamic_uniform_buffer.get_dynamic_offset(original_index);

        // Bind descriptor set with dynamic offset
        command_buffer_builder.bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            self.pipeline.layout().clone(),
            0,
            (
                self.transform_descriptor_set.clone(),
                self.material_set.clone(),
            ),
            [dynamic_offset], // Dynamic offset for model matrix
        )?;

        // Draw mesh
        // ...
    }
}
```

---

## TASK 8: Update Pipeline Layout for Dynamic Descriptors

**File**: `crates/praxis_graphics/src/pipeline.rs`

The descriptor set layout needs to be created with `DescriptorType::UniformBufferDynamic` instead of `DescriptorType::UniformBuffer` for the model matrix binding.

**Note**: Currently the pipeline layout is auto-derived from shaders. We may need to:
1. Modify shader to use separate bindings for view/proj and model
2. Or manually specify the layout

**Recommended approach**:
- Set 0, Binding 0: ViewProjectionUniforms (regular uniform, once per frame)
- Set 0, Binding 1: ModelUniforms (dynamic uniform, per object)
- Set 0, Binding 2: Texture sampler (unchanged)
- Set 0, Binding 3: Lighting buffer (unchanged)
- Set 1, Binding 0: MaterialProperties (unchanged)

This requires shader changes to split the uniforms.

---

## TASK 9: Split Shader Uniforms

### Vertex Shader
**File**: `crates/praxis_graphics/src/shaders/triangle.vert`

**Change** from single uniforms block to:
```glsl
// Per-frame uniforms (view, projection, camera)
layout(set = 0, binding = 0, std140) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
    float _padding;
} vp;

// Per-object uniforms (model matrix) - uses dynamic offset
layout(set = 0, binding = 1, std140) uniform Model {
    mat4 model;
} obj;
```

**Update** main function to use new names:
```glsl
void main() {
    vec4 world_pos = obj.model * vec4(position, 1.0);
    gl_Position = vp.proj * vp.view * world_pos;
    v_normal = mat3(obj.model) * normal;
    // ...
}
```

### Fragment Shader
**File**: `crates/praxis_graphics/src/shaders/triangle.frag`

**Add**:
```glsl
layout(set = 0, binding = 0, std140) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
    float _padding;
} vp;
```

**Change** line 443:
```glsl
vec3 view_dir = normalize(vp.camera_position - v_world_pos);
```

---

## TASK 10: Update All Examples

After the API changes, verify and update all examples:

1. `fps_camera_controller.rs` - Uses camera, verify specular works
2. `comprehensive_scene_demo.rs` - Multiple objects, good stress test
3. `dynamic_lighting_demo.rs` - Tests lighting with camera
4. `material_demo.rs` - Tests material properties
5. `physics_demo.rs` - Already needs fixes (PREREQUISITE)
6. `multi_mesh_demo.rs` - Multiple meshes
7. `obj_loader_demo.rs` - Loaded models

**Verification for each**:
```bash
cargo run --example <name>
```
- Move camera around
- Verify specular highlights follow camera
- No visual regressions

---

## TASK 11: Add Performance Metrics

**File**: `crates/praxis_graphics/src/lib.rs`

Add timing/counting for:
```rust
// In render():
let buffer_write_start = Instant::now();
self.dynamic_uniform_buffer.write_models(&models)?;
trace!("Wrote {} models in {:?}", models.len(), buffer_write_start.elapsed());
```

Create a demo that renders 1000+ objects to demonstrate the performance improvement.

---

## TASK 12: Documentation Updates

### Update CLAUDE.md
Remove references to per-object buffer allocation, document dynamic buffer usage.

### Update BEGINNERS_GUIDE.md
Fix outdated API references (identified in CODEBASE_ANALYSIS_REPORT.md).

### Update lib.rs module docs
Document the dynamic uniform buffer pattern.

---

## Verification Checklist

After all tasks complete:

```bash
# Build everything
cargo build --workspace

# Build all examples
cargo build --examples

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --all -- -D warnings

# Verify examples work correctly
cargo run --example fps_camera_controller  # Check specular highlights
cargo run --example material_demo          # Check materials
cargo run --example physics_demo           # Check physics (was broken)
cargo run --example comprehensive_scene_demo  # Stress test
```

---

## Implementation Order

1. **PREREQUISITE**: Fix physics_demo.rs
2. **TASK 3**: Update Rust Uniforms struct (quick fix for camera)
3. **TASK 2**: Update shaders for camera position
4. **TASK 1**: Extend ViewProjectionUniforms
5. **TASK 4**: Export uniform_buffer module
6. **TASK 9**: Split shader uniforms (view/proj vs model)
7. **TASK 5**: Add DynamicUniformBuffer to RenderContext
8. **TASK 6**: Create frame-level descriptor set
9. **TASK 8**: Update pipeline layout for dynamic descriptors
10. **TASK 7**: Refactor render() to use dynamic offsets
11. **TASK 10**: Update and verify all examples
12. **TASK 11**: Add performance metrics
13. **TASK 12**: Documentation updates

---

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Shader compilation errors | Test each shader change individually |
| Descriptor set binding errors | Use vulkano's validation layers |
| Dynamic offset alignment | Use `min_uniform_buffer_offset_alignment` from device |
| Memory layout mismatch | Verify std140 alignment in both Rust and GLSL |
| Performance regression | Add metrics before/after comparison |

---

## Expected Outcome

- **Before**: N buffer allocations per frame (N = object count)
- **After**: 1 ring buffer allocation at startup, zero per-frame allocations
- **Specular lighting**: Correctly follows camera position
- **All examples**: Compile and run correctly
