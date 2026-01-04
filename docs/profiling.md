# Profiling and Performance Analysis

Praxis provides a comprehensive profiling system for analyzing CPU, GPU, and memory performance.

## Overview

The profiling system includes:

- **Frame Time Breakdown**: Detailed per-frame timing with phase categorization
- **GPU Profiling**: Vulkan timestamp queries for accurate GPU timing
- **Memory Tracking**: Allocation tracking with leak detection
- **System Profiling**: ECS system execution time and bottleneck identification
- **Chrome Tracing**: Export to Chrome Trace Event Format for visualization

## Quick Start

```rust
use praxis_profiling::{Profiler, ProfilerConfig, ProfileScope};

// Create profiler
let config = ProfilerConfig::default();
let mut profiler = Profiler::new(config);

// Main loop
loop {
    profiler.begin_frame();

    {
        let _scope = ProfileScope::new("update");
        // Your update code
    }

    profiler.end_frame();

    // Get statistics
    let stats = profiler.statistics();
    println!("FPS: {:.1}", stats.avg_fps);
}
```

## Frame Time Breakdown

### Phases

The profiler categorizes frame time into phases:

- `SystemUpdate`: ECS system updates
- `Physics`: Physics simulation
- `RenderPrep`: Rendering preparation
- `Rendering`: GPU rendering
- `PostProcess`: Post-processing effects
- `Gui`: GUI rendering
- `Present`: Present and swap
- `Other`: Unclassified time

### Custom Phase Mapping

Register custom phase mappings for your scopes:

```rust
profiler.register_phase_mapping(
    "my_physics_system".to_string(),
    FramePhase::Physics
);
```

### Frame Statistics

```rust
let stats = profiler.frame_statistics();
println!("Average FPS: {:.1}", stats.avg_fps());
println!("Min FPS: {:.1}", stats.min_fps());
println!("Max FPS: {:.1}", stats.max_fps());

// Get per-phase statistics
for (phase, duration) in &stats.avg_phase_times {
    println!("{}: {:.2}ms", phase.name(), duration.as_secs_f64() * 1000.0);
}
```

## GPU Profiling

### Setup

```rust
use praxis_profiling::GpuProfiler;

let gpu_profiler = GpuProfiler::new(
    device.clone(),
    queue.clone(),
    128, // max queries per frame
    3,   // buffered frames
)?;

profiler.setup_gpu_profiler(gpu_profiler);
```

### Usage in Rendering

```rust
use vulkano::sync::PipelineStage;

let mut builder = AutoCommandBufferBuilder::primary(...)?;

if let Some(gpu_profiler) = profiler.gpu_profiler() {
    let mut gpu_profiler = gpu_profiler.lock();
    
    // Reset queries at start of command buffer
    gpu_profiler.reset_queries(&mut builder)?;

    // Profile a render pass
    if let Some((pool, start, end)) = gpu_profiler.begin_query("main_pass") {
        GpuProfiler::write_timestamp(
            &mut builder,
            pool.clone(),
            start,
            PipelineStage::TopOfPipe,
        )?;

        // Your rendering commands here

        GpuProfiler::write_timestamp(
            &mut builder,
            pool,
            end,
            PipelineStage::BottomOfPipe,
        )?;
    }
}

// Collect results after GPU work completes
let gpu_results = gpu_profiler.lock().collect_results()?;
for timestamp in gpu_results {
    println!("{}: {:.2}ms", 
        timestamp.name, 
        timestamp.duration().as_secs_f64() * 1000.0
    );
}
```

### RAII GPU Scopes

```rust
unsafe {
    let _scope = GpuProfileScope::new(&mut builder, &mut gpu_profiler, "shadow_pass");
    // Shadow rendering commands
}
```

## Memory Tracking

### Basic Tracking

```rust
let tracker = profiler.memory_tracker();

// Track allocation
let id = tracker.track_allocation(
    1024 * 1024,
    "vertex_buffer".to_string(),
    "Rendering".to_string(),
);

// Use the memory...

// Track deallocation
tracker.track_deallocation(id);
```

### RAII Guard

```rust
use praxis_profiling::AllocationGuard;

{
    let _guard = AllocationGuard::new(
        tracker.clone(),
        1024 * 1024,
        "temp_buffer".to_string(),
        "Rendering".to_string(),
    );
    // Memory is tracked while guard is alive
} // Automatically deallocated
```

### Statistics

```rust
let stats = tracker.statistics();
println!("Current allocated: {} MB", 
    stats.current_allocated as f64 / (1024.0 * 1024.0));
println!("Peak allocated: {} MB", 
    stats.peak_allocated as f64 / (1024.0 * 1024.0));
println!("Total allocations: {}", stats.allocation_count);

// Per-category breakdown
for (category, bytes) in &stats.bytes_by_category {
    println!("  {}: {} MB", category, *bytes as f64 / (1024.0 * 1024.0));
}
```

### Leak Detection

```rust
let leak_detector = profiler.leak_detector();

// Create checkpoint
leak_detector.checkpoint();

// Run your code...

// Detect leaks (allocations older than 1 second)
let leaks = leak_detector.detect_leaks(Duration::from_secs(1));
for (id, alloc) in leaks {
    println!("Potential leak:");
    println!("  Size: {} bytes", alloc.size);
    println!("  Location: {}", alloc.location);
    println!("  Category: {}", alloc.category);
    println!("  Age: {:.2}s", alloc.timestamp.elapsed().as_secs_f64());
}
```

## System Profiling

### Manual Profiling

```rust
let system_profiler = profiler.system_profiler();

system_profiler.begin_system("physics_update");
// System code...
system_profiler.end_system("physics_update");
```

### RAII Guard

```rust
use praxis_profiling::system_profiler::SystemProfileScope;

{
    let _guard = SystemProfileScope::new(&system_profiler, "render_system");
    // System code...
}
```

### Statistics

```rust
// Get all system statistics
let stats = system_profiler.system_statistics();
for stat in stats {
    println!("{}: avg {:.2}ms ({:.1}%)", 
        stat.name,
        stat.avg_time.as_secs_f64() * 1000.0,
        stat.frame_percentage
    );
}

// Get top slowest systems
let slowest = system_profiler.top_slowest_systems(5);
```

### Bottleneck Detection

```rust
let bottlenecks = system_profiler.identify_bottlenecks();
for bottleneck in bottlenecks {
    println!("⚠ Bottleneck: {}", bottleneck.name);
    println!("  Type: {:?}", bottleneck.bottleneck_type);
    println!("  Time: {:.2}ms ({:.1}%)", 
        bottleneck.avg_time.as_secs_f64() * 1000.0,
        bottleneck.percentage
    );
    println!("  Severity: {:.0}%", bottleneck.severity * 100.0);
    println!("  Recommendation: {}", bottleneck.recommendation);
}
```

## Chrome Tracing Export

### Basic Export

```rust
// Start tracing
profiler.begin_trace_export();

// Run your application with profiling...

// Stop and save
profiler.end_trace_export("trace.json")?;
```

### Viewing Traces

#### Chrome Tracing

1. Open Chrome or Chromium browser
2. Navigate to `chrome://tracing`
3. Click "Load" and select your trace file
4. Use mouse to zoom and pan
5. Click on events to see details

#### Perfetto

1. Visit https://ui.perfetto.dev/
2. Click "Open trace file"
3. Select your trace file
4. More powerful analysis than chrome://tracing

### Trace Contents

The trace includes:

- **CPU Scopes**: All `ProfileScope` timings with nesting
- **GPU Queries**: GPU timestamp query results (if enabled)
- **Memory Counters**: Memory allocation over time
- **Frame Markers**: Visual frame boundaries
- **Thread Information**: Per-thread timelines

### Example Trace Analysis

```
Frame 100 (16.7ms)
├─ physics_update (2.3ms)
│  ├─ collision_detection (1.1ms)
│  └─ integrate_velocities (0.9ms)
├─ render_prep (1.2ms)
│  ├─ cull_frustum (0.5ms)
│  └─ update_buffers (0.6ms)
└─ render (11.2ms)
   ├─ shadow_pass (3.1ms)
   ├─ main_pass (6.8ms)
   └─ post_process (1.3ms)
```

## Visualization

### Frame Time Graph

```rust
use praxis_profiling::FrameTimeGraph;

let mut graph = FrameTimeGraph::new(300, 60.0);

// Each frame
graph.add_frame_time(frame_duration);

// Get data for plotting
let data = graph.data();
let avg = graph.average();
let min = graph.min();
let max = graph.max();
```

### Phase Pie Chart

```rust
use praxis_profiling::PhasePieChart;

if let Some(breakdown) = profiler.current_frame_breakdown() {
    let pie_chart = PhasePieChart::from_breakdown(&breakdown);
    
    for (phase, percentage, color) in &pie_chart.segments {
        println!("{}: {:.1}% (RGB: {:.2}, {:.2}, {:.2})",
            phase.name(),
            percentage,
            color.r, color.g, color.b
        );
    }
}
```

### System Bar Chart

```rust
use praxis_profiling::SystemBarChart;

let system_stats = system_profiler.system_statistics();
let bar_chart = SystemBarChart::from_system_stats(&system_stats, 10);

for (name, time_ms, percentage) in &bar_chart.entries {
    println!("{}: {:.2}ms ({:.1}%)", name, time_ms, percentage);
}
```

### Memory Graph

```rust
use praxis_profiling::MemoryGraph;

let mut graph = MemoryGraph::new(300);

// Each frame
let stats = tracker.statistics();
graph.add_sample(stats.current_allocated);

// Get data
let data_mb = graph.data_mb();
let current = graph.current_mb();
let max = graph.max_mb();
```

## Performance Overhead

The profiling system is designed for minimal overhead:

| Feature | Overhead |
|---------|----------|
| CPU Scope | ~50-100ns |
| GPU Query | <1μs (GPU side only) |
| Memory Tracking | ~100ns per alloc/dealloc |
| System Profiling | ~50ns per system |

### Reducing Overhead

For production builds:

```rust
// Conditional compilation
#[cfg(feature = "profiling")]
let _scope = ProfileScope::new("expensive_function");

// Or runtime check
if cfg!(debug_assertions) {
    profiler.begin_frame();
    // ...
    profiler.end_frame();
}
```

## Best Practices

### 1. Descriptive Names

```rust
// Good
let _scope = ProfileScope::new("physics_rigidbody_integration");

// Bad
let _scope = ProfileScope::new("update");
```

### 2. Hierarchical Scopes

```rust
{
    let _parent = ProfileScope::new("render");
    
    {
        let _child = ProfileScope::new("shadow_pass");
        // Shadow rendering
    }
    
    {
        let _child = ProfileScope::new("main_pass");
        // Main rendering
    }
}
```

### 3. Categorize Allocations

```rust
tracker.track_allocation(size, location, "Physics".to_string());
tracker.track_allocation(size, location, "Rendering".to_string());
tracker.track_allocation(size, location, "Audio".to_string());
```

### 4. Regular Profiling

- Profile during development, not just when problems occur
- Create performance budgets for systems
- Track performance over time
- Use bottleneck detection to prioritize optimization

### 5. Use Chrome Tracing

- Export traces for complex performance issues
- Share traces with team members
- Compare before/after optimization

## Integration with ECS

```rust
use praxis_profiling::{ProfilerResource, SystemProfilerResource};
use bevy_ecs::prelude::*;

fn setup(mut commands: Commands) {
    let profiler = Profiler::new(ProfilerConfig::default());
    let system_profiler = profiler.system_profiler().clone();
    
    commands.insert_resource(ProfilerResource::new(profiler));
    commands.insert_resource(SystemProfilerResource::new(system_profiler));
}

fn my_system(system_profiler: Res<SystemProfilerResource>) {
    let _guard = SystemProfileScope::new(
        system_profiler.profiler(),
        "my_system"
    );
    
    // System logic
}
```

## Example

See `examples/profiling_demo.rs` for a complete working example:

```bash
cargo run --example profiling_demo
```

## Troubleshooting

### GPU Queries Not Working

- Check that timestamp queries are supported: `gpu_profiler.is_supported()`
- Ensure queries are reset at the start of command buffer
- Wait for GPU to finish before collecting results

### High Memory Usage

- Check for unclosed allocation guards
- Run leak detection regularly
- Review allocation categories for unexpected allocations

### Bottlenecks Not Detected

- Adjust `bottleneck_threshold` in `ProfilerConfig`
- Ensure systems are being profiled with `SystemProfiler`
- Check that frame time is being set correctly

### Chrome Trace Too Large

- Reduce `max_frame_history`
- Profile shorter durations
- Disable unnecessary profiling features
