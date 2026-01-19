# Optimization Panel

The **Optimization Panel** provides a comprehensive interface for configuring rendering optimizations with real-time performance comparison.

## Features

### Preset Management

The panel includes four optimization presets for quick configuration:

- **Low**: All optimizations disabled (baseline for debugging)
- **Medium**: Basic optimizations (multi-draw indirect, GPU culling, descriptor caching)
- **High**: Advanced optimizations (adds GPU LOD, Hi-Z occlusion, backface/distance culling)
- **Ultra**: All optimizations enabled (maximum performance)

### Individual Optimization Toggles

Configure each optimization independently:

#### Core Optimizations
- **Multi-Draw Indirect**: Batch multiple draw calls into a single indirect draw
- **GPU Culling**: Compute shader-based frustum and occlusion culling
- **GPU LOD Selection**: GPU-driven level-of-detail selection
- **Descriptor Caching**: Reuse descriptor sets across frames

#### Advanced Optimizations
- **Hi-Z Occlusion**: Hierarchical Z-buffer occlusion culling
- **Mesh Streaming**: Background async loading of mesh data

#### GPU Culling Strategies
- **Backface Culling**: Cull objects facing away from camera
- **Small Object Culling**: Cull objects below screen-space threshold
- **Distance Culling**: Cull objects beyond max render distance

### Performance Comparison

The panel provides before/after performance comparison:

1. **Capture Before**: Take a snapshot of current rendering metrics
2. **Toggle Optimizations**: Enable/disable optimizations as desired
3. **Capture After**: Take a snapshot of new rendering metrics
4. **View Delta**: See the performance impact with color-coded improvements

Metrics compared:
- Draw calls
- Visible objects
- Culling efficiency
- Descriptor allocations

### Live Statistics

Real-time monitoring of rendering performance:
- Frame number
- Total objects in scene
- Visible objects after culling
- Objects culled by frustum
- Objects culled by occlusion
- Draw calls issued
- Descriptor allocations
- Culling efficiency percentage
- Streaming queue depth

### Performance Graphs

Optional live graphs showing:
- Draw calls over time
- Visible objects over time
- Performance trends (last 60 frames)

## Usage

### Integration into Editor

The optimization panel is automatically integrated into the editor as a dockable panel:

```rust
use praxis_editor::EditorState;

let mut editor = EditorState::new();

// Access the optimization panel
let optimization_panel = editor.optimization_panel_mut();

// Update with render stats each frame
optimization_panel.update_stats(render_stats);
```

### Standalone Usage

You can also use the panel standalone:

```rust
use praxis_editor::OptimizationPanel;
use praxis_graphics::RenderStats;

let mut panel = OptimizationPanel::new();

// In your render loop:
panel.update_stats(stats);

// Render the panel
egui::Window::new("Optimization")
    .show(ctx, |ui| {
        panel.ui(ui, None, None);
    });

// Access configuration
if let Some(config) = panel.config() {
    if config.has_changed() {
        // Apply to render context
        render_context.set_optimization_config(config.clone());
    }
}
```

### Preset Configuration

Apply presets programmatically:

```rust
use praxis_editor::{OptimizationPanel, OptimizationPreset};

let mut panel = OptimizationPanel::new();

// Apply a preset
if let Some(config) = panel.config_mut() {
    OptimizationPreset::Ultra.apply_to(config);
}

// Detect current preset
let current = OptimizationPreset::detect_from(panel.config().unwrap());
println!("Current preset: {}", current.name());
```

## Menu Integration

The optimization panel can be toggled from the View menu:
- **View → Optimization**: Toggle panel visibility

## Best Practices

1. **Baseline Comparison**: Start with Low preset to establish baseline performance
2. **Incremental Testing**: Toggle one optimization at a time to measure individual impact
3. **Capture Timing**: Wait for scene to stabilize before capturing before/after snapshots
4. **Monitor Trends**: Use live graphs to identify performance patterns
5. **Document Findings**: Note which optimizations provide the most benefit for your use case

## Example Workflow

1. Load a representative test scene
2. Set preset to **Low** (all optimizations off)
3. Click **📸 Capture Before** to record baseline metrics
4. Set preset to **Ultra** (all optimizations on)
5. Click **✓ Capture After** to record optimized metrics
6. Review the **Performance Delta** section to see improvements
7. Toggle individual optimizations to fine-tune for your scene
8. Monitor the **Live Statistics** to ensure stable performance

## Performance Impact Examples

Typical improvements with optimizations enabled:

- **Draw Calls**: 40-60% reduction with multi-draw indirect
- **Visible Objects**: 50-80% reduction with GPU culling
- **Culling Efficiency**: 70-90% with all culling strategies enabled
- **Descriptor Allocations**: 30-50% reduction with caching

Results vary based on scene complexity, camera position, and hardware.

## See Also

- `examples/optimization_panel_demo.rs` - Interactive demonstration
- `crates/praxis_graphics/src/utilities/optimization_config.rs` - Configuration implementation
- `crates/praxis_graphics/src/utilities/render_stats.rs` - Statistics tracking
