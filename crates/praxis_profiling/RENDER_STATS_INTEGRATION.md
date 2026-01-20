# Rendering Statistics Integration

This document explains the integration between `praxis_graphics::RenderStats` and the profiling system's Chrome trace export functionality.

## Overview

The profiling system can automatically export rendering statistics as counter events in Chrome trace format, allowing visualization of rendering performance metrics alongside CPU/GPU profiling data. This provides a comprehensive view of engine performance in a single timeline.

## Features

### Automatic Counter Export

When trace export is active, rendering statistics are automatically converted to counter events:

- **Culling Efficiency %**: Percentage of objects successfully culled (0-100)
- **Total Objects**: Total objects submitted for rendering
- **Visible Objects**: Objects rendered after culling
- **Frustum Culled**: Objects culled by frustum test
- **Occlusion Culled**: Objects culled by occlusion test
- **Draw Calls**: Draw calls issued to GPU
- **Draw Call Reduction**: Objects saved from rendering via culling/batching
- **Descriptor Allocations**: Descriptor sets allocated this frame
- **LOD Level N %**: Percentage of objects at each LOD level
- **Streaming Queue Depth**: Meshes waiting to be loaded

### Chrome Trace Visualization

All metrics appear as counter tracks in chrome://tracing, enabling:

1. **Correlation**: See how rendering metrics relate to CPU/GPU work
2. **Trend Analysis**: Track metrics over time (e.g., culling efficiency across frames)
3. **Bottleneck Identification**: Identify frames with performance issues
4. **System Behavior**: Understand LOD system transitions and streaming patterns

## Usage

### Basic Integration

```rust
use praxis_profiling::{Profiler, ProfilerConfig};
use std::time::Instant;

// Create profiler
let mut profiler = Profiler::new(ProfilerConfig::default());

// Start trace export
profiler.begin_trace_export();

// Main loop
for frame in 0..300 {
    profiler.begin_frame();
    
    // ... rendering code ...
    
    // Record rendering statistics
    let render_stats = render_context.current_render_stats();
    profiler.record_render_stats(&render_stats, Instant::now());
    
    profiler.end_frame();
}

// Save trace file
profiler.end_trace_export("trace.json")?;
```

### Loading in Chrome

1. Open Chrome or Chromium browser
2. Navigate to `chrome://tracing`
3. Click "Load" and select `trace.json`
4. Use the timeline viewer to analyze performance:
   - Zoom in/out with mouse wheel
   - Pan with click and drag
   - Select events to see details
   - Use WASD keys for navigation

### Counter Track Layout

Counter events are organized into categories:

- **Rendering**: Main rendering metrics (culling, objects, draw calls)
- **Rendering/LOD**: LOD distribution percentages

These appear as separate tracks in the timeline, making it easy to correlate different aspects of rendering performance.

## Metrics Explained

### Culling Efficiency

**Formula**: `(frustum_culled + occlusion_culled) / total_objects * 100`

**Interpretation**:
- 0%: No culling (all objects rendered)
- 50%: Half of objects culled
- 90%+: Excellent culling efficiency

**Expected Values**:
- Indoor scenes: 60-80% (many objects occluded)
- Outdoor scenes: 40-60% (distant objects culled)
- Optimized scenes: 70-90%+

### Draw Call Reduction

**Formula**: `total_objects - draw_calls`

**Interpretation**:
- Shows effectiveness of batching and instancing
- Higher values indicate better draw call optimization
- Ratio to total_objects shows batching efficiency

**Expected Values**:
- No batching: ~0 (one draw call per object)
- Material batching: 20-50% reduction
- Instancing: 80-95% reduction

### LOD Distribution

**Interpretation**:
- Shows which LOD levels are active
- Level 0: Highest detail (near camera)
- Level N: Lowest detail (far from camera)
- Good distribution: Gradual falloff from high to low detail

**Expected Patterns**:
- Stationary camera: Stable distribution
- Moving camera: Shifting percentages as objects transition
- LOD bias changes: Immediate shift to different levels

### Streaming Queue Depth

**Interpretation**:
- Number of meshes waiting to load
- 0: All meshes loaded
- Non-zero: Streaming system is active

**Expected Values**:
- Stable scene: 0 (all loaded)
- Camera movement: 1-10 (loading new meshes)
- Fast camera: 10+ (aggressive streaming)

## Advanced Usage

### Custom Counter Export

For specialized use cases, you can manually export counters:

```rust
use praxis_profiling::ChromeTraceExporter;
use std::time::Instant;

let mut exporter = ChromeTraceExporter::new();

// Add custom rendering metrics
exporter.add_counter(
    "Custom Metric".to_string(),
    "Rendering".to_string(),
    Instant::now(),
    42.0,
);

// Add multiple counters at once
exporter.add_counters(
    vec![
        ("Metric 1".to_string(), "Custom".to_string(), 1.0),
        ("Metric 2".to_string(), "Custom".to_string(), 2.0),
    ],
    Instant::now(),
);
```

### Programmatic Analysis

You can also use the conversion utilities directly:

```rust
use praxis_profiling::conversion::render_stats_to_counters;

let render_stats = /* ... */;
let counters = render_stats_to_counters(&render_stats);

// Process counters programmatically
for (name, category, value) in counters {
    println!("{} [{}]: {}", name, category, value);
}
```

## Feature Flags

The integration is controlled by the `graphics_integration` feature flag:

- **Enabled (default)**: Full integration with `praxis_graphics`
- **Disabled**: Profiling works without graphics dependency

To disable:

```toml
[dependencies]
praxis_profiling = { version = "0.1.0", default-features = false }
```

## Performance Considerations

### Overhead

- **Trace Export Disabled**: Zero overhead (no code executed)
- **Trace Export Enabled**: ~50-100ns per counter event
- **Per Frame**: ~1-2μs for all rendering counters

### Recommendations

1. **Development**: Always enable for performance investigation
2. **Release**: Conditionally enable based on profiling mode
3. **Shipping**: Disable trace export entirely

### Memory Usage

- Each counter event: ~200 bytes
- 300 frames × 11 counters: ~660KB
- Trace file size: ~1-5MB for typical sessions

## Troubleshooting

### Counters Not Appearing

**Problem**: Counter tracks don't show in chrome://tracing

**Solutions**:
- Verify `begin_trace_export()` was called before recording stats
- Check that `record_render_stats()` is called each frame
- Ensure `end_trace_export()` was called to write the file

### Incorrect Values

**Problem**: Counter values seem wrong

**Solutions**:
- Verify `RenderStats` is being populated correctly
- Check timestamp matches frame timing
- Ensure stats are recorded after rendering, not before

### Missing LOD Metrics

**Problem**: LOD level counters don't appear

**Solutions**:
- Verify `active_lod_levels` is populated in RenderStats
- Check that LOD system is enabled in your scene
- Ensure objects have LOD data

## Examples

See these examples for complete integration:

- `examples/profiling_demo.rs`: Basic profiling with rendering stats
- `examples/performance_profiling_comprehensive.rs`: Full-featured profiling
- `examples/render_stats_demo.rs`: Rendering statistics visualization

## See Also

- [Chrome Tracing Documentation](chrome_trace.rs)
- [RenderStats API](../praxis_graphics/src/utilities/render_stats.rs)
- [Profiler Documentation](profiler.rs)
