# Render Statistics System

The render statistics system provides comprehensive tracking and analysis of per-frame rendering metrics in the Praxis engine.

## Overview

The system tracks the following metrics per frame:

- **Total Objects**: Number of objects submitted for rendering
- **Visible Objects**: Objects rendered after culling
- **Frustum Culled**: Objects outside the camera frustum
- **Occlusion Culled**: Objects hidden behind other geometry
- **Draw Calls**: Number of GPU draw calls issued
- **Descriptor Allocations**: Descriptor set allocations (cached vs new)
- **Active LOD Levels**: Distribution of objects across LOD levels
- **Streaming Queue Depth**: Meshes waiting to be streamed

## Architecture

### Core Components

#### `RenderStats`
Per-frame snapshot of rendering metrics. Created once per frame and recorded to history.

```rust
pub struct RenderStats {
    pub frame_number: u64,
    pub total_objects: usize,
    pub visible_objects: usize,
    pub frustum_culled: usize,
    pub occlusion_culled: usize,
    pub draw_calls: usize,
    pub descriptor_allocations: usize,
    pub active_lod_levels: Vec<(usize, usize)>,
    pub streaming_queue_depth: usize,
}
```

**Methods:**
- `culling_efficiency()`: Percentage of objects culled
- `visibility_ratio()`: Percentage of objects rendered
- `lod_distribution_percentages()`: LOD level distribution

#### `RenderStatsHistory`
Rolling history with statistical aggregation. Maintains a circular buffer of recent frames (default: 300 frames = ~5 seconds at 60 FPS).

```rust
pub struct RenderStatsHistory {
    frames: VecDeque<RenderStats>,
    max_frames: usize,
    // ... aggregation fields
}
```

**Key Methods:**
- `record(stats)`: Add a new frame's statistics
- `avg_visible_objects()`: Average across tracked frames
- `max_draw_calls()`: Peak draw calls recorded
- `avg_culling_efficiency()`: Average culling percentage
- `export_to_csv(path)`: Export to CSV file

#### `RenderStatsVisualizer`
Visualization data for GUI rendering. Extracts formatted data from history for charts and graphs.

```rust
pub struct RenderStatsVisualizer {
    pub visible_objects_graph: Vec<f32>,
    pub draw_calls_graph: Vec<f32>,
    pub culling_efficiency_graph: Vec<f32>,
    pub descriptor_allocations_graph: Vec<f32>,
    pub streaming_queue_graph: Vec<f32>,
    pub culling_breakdown: CullingBreakdown,
    pub summary: StatsSummary,
}
```

## Integration

### Automatic Collection

Stats collection is integrated into `RenderContext::render()` and enabled by default:

```rust
// Stats are collected automatically each frame
render_context.render(&cmds)?;

// Access current frame stats
let stats = render_context.render_stats();
println!("Visible objects: {}/{}", stats.visible_objects, stats.total_objects);
```

### Accessing Statistics

```rust
// Current frame
let stats = render_context.render_stats();

// Historical data
let history = render_context.render_stats_history();
println!("Average visible: {:.1}", history.avg_visible_objects());
println!("Peak draw calls: {}", history.max_draw_calls());
```

### Controlling Collection

```rust
// Disable for maximum performance
render_context.set_render_stats_enabled(false);

// Re-enable for profiling
render_context.set_render_stats_enabled(true);
```

### Exporting to CSV

```rust
// Export for external analysis
render_context.export_render_stats_csv("stats.csv")?;
```

CSV format:
```csv
frame_number,total_objects,visible_objects,frustum_culled,occlusion_culled,draw_calls,descriptor_allocations,streaming_queue_depth,culling_efficiency
1,1000,250,650,100,120,15,5,75.0
2,1000,245,655,100,118,15,4,75.5
```

## Performance Impact

When **enabled**:
- Minimal overhead: ~5-10 CPU cycles per tracked metric
- Memory: ~36 bytes per frame × history size (default: 10.8 KB)
- No GPU overhead

When **disabled**:
- Zero overhead: all tracking code is skipped via runtime checks

## Use Cases

### Performance Profiling

Track rendering efficiency over time:

```rust
let history = render_context.render_stats_history();

// Identify culling effectiveness
println!("Culling efficiency: {:.1}%", history.avg_culling_efficiency());

// Monitor draw call batching
println!("Draw calls: avg {:.1}, peak {}", 
    history.avg_draw_calls(),
    history.max_draw_calls()
);
```

### Optimization Validation

Verify optimization impact:

```rust
// Before optimization
let before_avg = history.avg_draw_calls();

// ... apply optimization ...

// After optimization (wait for stats to accumulate)
let after_avg = history.avg_draw_calls();
println!("Draw call reduction: {:.1}%", 
    (before_avg - after_avg) / before_avg * 100.0
);
```

### Automated Testing

Export stats for regression testing:

```rust
// Run test scenario
for _ in 0..300 {
    render_context.render(&test_scene)?;
}

// Export for comparison
render_context.export_render_stats_csv("test_baseline.csv")?;
```

### Real-time Monitoring

Display live stats in debug UI:

```rust
let stats = render_context.render_stats();
ui.label(format!("Visible: {}/{}", stats.visible_objects, stats.total_objects));
ui.label(format!("Draw Calls: {}", stats.draw_calls));
ui.label(format!("Culling: {:.1}%", stats.culling_efficiency()));
```

## Visualization

### Historical Graphs

```rust
let viz = RenderStatsVisualizer::from_history(history);

// Access graph data
let visible_objects_over_time = viz.visible_objects_graph;
let draw_calls_over_time = viz.draw_calls_graph;
let culling_efficiency_over_time = viz.culling_efficiency_graph;
```

### Statistical Summary

```rust
let summary = viz.summary;
println!("Average visible: {:.1}", summary.avg_visible);
println!("Peak visible: {}", summary.peak_visible);
println!("Average draw calls: {:.1}", summary.avg_draw_calls);
println!("Peak draw calls: {}", summary.peak_draw_calls);
```

### egui Integration

When the `egui` feature is enabled, `RenderStatsVisualizer` provides a `render_ui()` method:

```rust
#[cfg(feature = "egui")]
{
    let viz = RenderStatsVisualizer::from_history(history);
    viz.render_ui(&mut ui);
}
```

This displays:
- Summary statistics (averages, peaks)
- Line graphs for visible objects, draw calls, culling efficiency
- Formatted labels with units and percentages

## Example: Render Stats Demo

See `examples/render_stats_demo.rs` for a complete demonstration:

```bash
cargo run --example render_stats_demo
```

Features:
- Real-time stats display
- Large scene with 500+ objects
- Camera controls for exploration
- CSV export on keypress
- Toggle stats overlay

## Integration with Other Systems

### GPU Culling

When GPU culling is enabled, stats track actual GPU culling results:

```rust
render_context.enable_gpu_culling()?;

// Stats will show GPU-culled counts
let stats = render_context.render_stats();
println!("GPU frustum culled: {}", stats.frustum_culled);
```

### LOD System

LOD distribution is tracked automatically:

```rust
let stats = render_context.render_stats();
for (level, percentage) in stats.lod_distribution_percentages() {
    println!("LOD {}: {:.1}%", level, percentage);
}
```

### Mesh Streaming

Streaming queue depth tracks pending mesh loads:

```rust
let stats = render_context.render_stats();
println!("Meshes in streaming queue: {}", stats.streaming_queue_depth);
```

## Best Practices

### Performance

1. **Disable in Release Builds** (if not needed):
   ```rust
   #[cfg(debug_assertions)]
   render_context.set_render_stats_enabled(true);
   
   #[cfg(not(debug_assertions))]
   render_context.set_render_stats_enabled(false);
   ```

2. **Adjust History Size** based on needs:
   ```rust
   // Short-term monitoring (1 second)
   render_context.render_stats_history_mut().clear();
   *render_context.render_stats_history_mut() = RenderStatsHistory::new(60);
   
   // Long-term profiling (5 minutes)
   *render_context.render_stats_history_mut() = RenderStatsHistory::new(18000);
   ```

### Analysis

1. **Export Regularly** for trend analysis:
   ```rust
   // Export every 5 minutes
   if frame_count % (60 * 60 * 5) == 0 {
       render_context.export_render_stats_csv(
           &format!("stats_{}.csv", chrono::Local::now().format("%Y%m%d_%H%M%S"))
       )?;
   }
   ```

2. **Monitor Key Metrics**:
   - Culling efficiency: Should be > 70% for most scenes
   - Draw calls: Should decrease with batching optimizations
   - Descriptor allocations: Should be low (< 50) with descriptor pooling

### Debugging

1. **Compare Before/After**:
   ```rust
   // Save baseline
   let before_avg_draw_calls = history.avg_draw_calls();
   
   // Test change
   // ...
   
   // Compare
   let after_avg_draw_calls = history.avg_draw_calls();
   assert!(after_avg_draw_calls < before_avg_draw_calls, "Optimization failed");
   ```

2. **Identify Outliers**:
   ```rust
   if stats.draw_calls > history.max_draw_calls() {
       warn!("Abnormal draw call count: {}", stats.draw_calls);
   }
   ```

## Technical Details

### Collection Points

Stats are collected at these points in `RenderContext::render()`:

1. **Frame Start**: Initialize stats, record total objects
2. **Post-Culling**: Record visible objects and culled counts
3. **Post-Batching**: Record draw call count
4. **Frame End**: Record stats to history

### Memory Layout

```
RenderStats: 88 bytes
- frame_number: 8 bytes
- Counters: 7 × 8 bytes = 56 bytes
- LOD levels: Vec (24 bytes)

History (300 frames):
- Frames: 300 × 88 bytes = 26.4 KB
- Aggregations: ~100 bytes
Total: ~26.5 KB
```

### Thread Safety

- `RenderStats`: Not thread-safe (owned by RenderContext)
- `RenderStatsHistory`: Not thread-safe (owned by RenderContext)
- Access via `&RenderContext` or `&mut RenderContext` provides safety

## Future Enhancements

Potential improvements:

1. **GPU Timing**: Track GPU execution time per pass
2. **Memory Stats**: Track VRAM usage and allocations
3. **Shader Stats**: Track shader compilation and switches
4. **Pipeline Stats**: Track pipeline state changes
5. **Advanced Visualizations**: Flame graphs, heat maps
6. **Real-time Alerts**: Warn on performance degradation
7. **Comparative Analysis**: Compare multiple runs

## See Also

- `examples/render_stats_demo.rs`: Complete demonstration
- `crates/praxis_profiling/`: CPU/GPU profiling system
- `crates/praxis_graphics/src/gpu_culling.rs`: GPU culling integration
- `crates/praxis_graphics/src/lod.rs`: LOD system integration
