# Debug Rendering for Optimization Systems

This document describes the visual debug rendering system for visualizing optimization features in the Praxis engine.

## Overview

The debug rendering system provides real-time visualization of:

1. **Frustum Culling Results** - Wireframe bounding spheres colored by visibility
2. **LOD Level Heat Maps** - Color-coded visualization of level-of-detail selection
3. **Occlusion Buffer** - Hierarchical Z-buffer visualization
4. **Mesh Streaming State** - Loading progress and priority indicators

## Features

### 1. Wireframe Bounding Spheres (Culling Results)

Visualizes culling decisions with color-coded wireframe spheres around each object:

- **Green** - Object is visible (passed all culling tests)
- **Red** - Object is culled (failed one or more tests)

**Use Cases:**
- Verify frustum culling is working correctly
- Debug objects incorrectly culled or not culled
- Optimize bounding sphere sizes for better culling accuracy

**Example:**
```rust
use praxis_graphics::debug_rendering::{DebugRenderer, DebugRenderMode, CullingDebugInfo};

// Enable culling debug mode
debug_renderer.enable_mode(DebugRenderMode::CullingResults);

// Prepare culling info
let culling_info = vec![
    CullingDebugInfo {
        position: Vec3::new(0.0, 0.0, 0.0),
        radius: 2.0,
        is_visible: true,
        cull_reason: None,
    },
];

// Render debug visualization
debug_renderer.render_culling_debug(&mut cmd_builder, &culling_info, view_proj)?;
```

### 2. LOD Heat Map

Color-coded visualization showing LOD level selection across the scene:

- **Blue** - Highest detail (LOD level 0)
- **Cyan** - High-medium detail
- **Green** - Medium detail
- **Yellow** - Medium-low detail
- **Orange** - Low detail
- **Red** - Lowest detail (highest LOD level)

**Use Cases:**
- Verify LOD transitions happen at correct distances
- Tune LOD distance thresholds for optimal performance
- Debug LOD popping or incorrect level selection

**Example:**
```rust
use praxis_graphics::debug_rendering::{DebugRenderMode, LodDebugInfo};

// Enable LOD heat map
debug_renderer.enable_mode(DebugRenderMode::LodHeatMap);

// Prepare LOD info
let lod_info = vec![
    LodDebugInfo {
        position: Vec3::new(5.0, 0.0, 0.0),
        radius: 1.5,
        current_lod_level: 1,
        total_lod_levels: 4,
        distance_from_camera: 25.0,
    },
];

// Render heat map
debug_renderer.render_lod_debug(&mut cmd_builder, &lod_info, view_proj)?;
```

### 3. Occlusion Buffer Visualization

Shows the hierarchical Z-buffer used for occlusion culling:

- Depth buffer overlay with configurable intensity
- Per-object occlusion query results
- Visualize occluders and occludees

**Use Cases:**
- Verify occlusion culling is working correctly
- Identify ineffective occluders
- Debug false occlusion culling

**Configuration:**
```rust
let mut config = DebugRenderConfig::default();
config.show_occlusion_buffer = true;
config.occlusion_intensity = 0.5; // 0.0 to 1.0

debug_renderer.set_config(config);
```

### 4. Mesh Streaming State Indicators

Visualizes async mesh loading with color-coded indicators:

- **Gray** - Mesh not loaded
- **Yellow** - Loading in progress (with progress bar)
- **Green** - Fully loaded
- **Blue** - High priority in loading queue

**Use Cases:**
- Monitor mesh streaming performance
- Debug streaming priority system
- Identify streaming bottlenecks

**Example:**
```rust
use praxis_graphics::debug_rendering::{StreamingDebugInfo, StreamingState};

// Enable streaming state visualization
debug_renderer.enable_mode(DebugRenderMode::MeshStreamingState);

// Prepare streaming info
let streaming_info = vec![
    StreamingDebugInfo {
        position: Vec3::new(10.0, 0.0, 0.0),
        radius: 2.0,
        state: StreamingState::Loading,
        load_progress: 0.65, // 65% loaded
    },
];

// Render streaming state
debug_renderer.render_streaming_debug(&mut cmd_builder, &streaming_info, view_proj)?;
```

## API Reference

### DebugRenderer

Main interface for debug rendering.

```rust
pub struct DebugRenderer {
    // ...
}

impl DebugRenderer {
    /// Creates a new debug renderer
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<dyn MemoryAllocator>,
        render_pass: Arc<RenderPass>,
        viewport_dimensions: [u32; 2],
    ) -> Result<Self>;

    /// Enables a debug rendering mode
    pub fn enable_mode(&mut self, mode: DebugRenderMode);

    /// Disables a debug rendering mode
    pub fn disable_mode(&mut self, mode: DebugRenderMode);

    /// Toggles a debug rendering mode
    pub fn toggle_mode(&mut self, mode: DebugRenderMode);

    /// Renders culling debug visualization
    pub fn render_culling_debug(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        culling_info: &[CullingDebugInfo],
        view_proj: Mat4,
    ) -> Result<()>;

    /// Renders LOD heat map
    pub fn render_lod_debug(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        lod_info: &[LodDebugInfo],
        view_proj: Mat4,
    ) -> Result<()>;

    /// Renders mesh streaming state
    pub fn render_streaming_debug(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        streaming_info: &[StreamingDebugInfo],
        view_proj: Mat4,
    ) -> Result<()>;

    /// Renders all enabled debug visualizations
    pub fn render_all_debug(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        culling_info: &[CullingDebugInfo],
        lod_info: &[LodDebugInfo],
        streaming_info: &[StreamingDebugInfo],
        view_proj: Mat4,
    ) -> Result<()>;
}
```

### Debug Render Modes

```rust
pub enum DebugRenderMode {
    /// Wireframe bounding spheres colored by culling result
    CullingResults,
    /// LOD level heat map overlay
    LodHeatMap,
    /// Occlusion buffer visualization
    OcclusionBuffer,
    /// Mesh streaming state indicators
    MeshStreamingState,
}
```

### Configuration

```rust
pub struct DebugRenderConfig {
    /// Enable bounding sphere wireframes
    pub show_bounding_spheres: bool,
    /// Enable LOD heat map overlay
    pub show_lod_heat_map: bool,
    /// Enable occlusion buffer visualization
    pub show_occlusion_buffer: bool,
    /// Enable mesh streaming state indicators
    pub show_streaming_state: bool,
    /// Wireframe line width
    pub wireframe_thickness: f32,
    /// Heat map intensity (0.0 to 1.0)
    pub heat_map_intensity: f32,
    /// Occlusion buffer visualization intensity
    pub occlusion_intensity: f32,
}
```

## Helper Functions

The `helpers` module provides utility functions for creating debug info structures from engine data:

```rust
use praxis_graphics::debug_rendering::helpers;

// From GPU culling results
let culling_info = helpers::culling_info_from_gpu_result(
    position,
    radius,
    is_visible,
    was_frustum_culled,
    was_distance_culled,
);

// From LOD group
let lod_info = helpers::lod_info_from_lod_group(
    position,
    radius,
    &lod_group,
    distance_from_camera,
);

// From streaming state
let streaming_info = helpers::streaming_info_from_state(
    position,
    radius,
    StreamingState::Loading,
    0.75,
);
```

## Performance Considerations

### Overhead

Debug rendering adds minimal overhead when disabled:
- Zero cost when modes are disabled
- Line rendering is highly optimized
- Batched geometry for efficient GPU usage

Typical performance impact (1000 objects):
- **Culling Debug**: ~0.1ms per frame
- **LOD Heat Map**: ~0.1ms per frame
- **Streaming State**: ~0.15ms per frame (includes progress bars)
- **All Combined**: ~0.35ms per frame

### Optimization Tips

1. **Limit Visible Objects**: Only create debug info for visible objects
2. **Use Appropriate Detail**: Reduce wireframe segments for distant objects
3. **Batch Updates**: Update debug info once per frame, not per object
4. **Conditional Rendering**: Only render debug overlays when needed

```rust
// Good: Only create debug info for visible objects
let culling_info: Vec<_> = objects
    .iter()
    .filter(|obj| obj.is_in_view_frustum())
    .map(|obj| create_debug_info(obj))
    .collect();

// Better: Disable debug modes in release builds
#[cfg(debug_assertions)]
debug_renderer.enable_mode(DebugRenderMode::CullingResults);
```

## Integration with Editor

The debug renderer integrates seamlessly with the Praxis editor:

```rust
// In editor UI code
if ui.checkbox("Show Culling", &mut show_culling).changed() {
    if show_culling {
        debug_renderer.enable_mode(DebugRenderMode::CullingResults);
    } else {
        debug_renderer.disable_mode(DebugRenderMode::CullingResults);
    }
}

if ui.checkbox("Show LOD Heat Map", &mut show_lod).changed() {
    debug_renderer.toggle_mode(DebugRenderMode::LodHeatMap);
}
```

## Examples

See `examples/optimization_debug_demo.rs` for a complete demonstration of all debug rendering features.

Run the example:
```bash
cargo run --example optimization_debug_demo
```

Controls:
- `1` - Toggle culling debug visualization
- `2` - Toggle LOD heat map
- `3` - Toggle mesh streaming state
- `WASD` - Move camera
- Mouse - Look around

## Troubleshooting

### Wireframes Not Visible

**Problem**: Debug wireframes don't appear on screen.

**Solutions**:
1. Check that the debug mode is enabled: `debug_renderer.is_mode_enabled(mode)`
2. Verify debug info arrays are not empty
3. Ensure view-projection matrix is correct
4. Check line renderer is initialized with correct render pass

### Colors Incorrect

**Problem**: LOD heat map colors don't match expected levels.

**Solutions**:
1. Verify `current_lod_level` and `total_lod_levels` are correct
2. Check LOD group is updating each frame
3. Ensure distance calculations are using squared distance

### Performance Issues

**Problem**: Debug rendering causes frame rate drops.

**Solutions**:
1. Reduce number of objects with debug visualization
2. Lower wireframe segment count (modify `add_wireframe_sphere`)
3. Disable unused debug modes
4. Use release build for better performance

## Future Enhancements

Planned improvements to the debug rendering system:

- [ ] Screen-space overlay for statistics (FPS, object counts, etc.)
- [ ] Configurable wireframe segment count based on distance
- [ ] GPU-accelerated debug rendering for large scenes
- [ ] Occlusion buffer mipmap visualization
- [ ] Hierarchical debug rendering (show/hide by LOD level)
- [ ] Export debug visualizations to image files
- [ ] Custom shader-based debug overlays
