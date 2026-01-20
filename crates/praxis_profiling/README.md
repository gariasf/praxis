# Praxis Profiling

Performance profiling and analysis tools for the Praxis game engine.

## Overview

Comprehensive CPU/GPU profiling with memory tracking, bottleneck identification, and Chrome trace export.

**Key Features:**
- Frame time breakdown with hierarchical scopes
- GPU profiling via Vulkan timestamp queries
- Memory allocation tracking and leak detection
- ECS system profiling with bottleneck identification
- Chrome Trace Event Format export
- Rendering statistics integration (culling efficiency, draw calls, LOD distribution)
- ~50-100ns overhead per scope

## Quick Start

```rust
use praxis_profiling::{Profiler, ProfilerConfig, ProfileScope};

let mut profiler = Profiler::new(ProfilerConfig::default());
profiler.begin_trace_export();

loop {
    profiler.begin_frame();
    
    {
        let _scope = ProfileScope::new("physics_update");
        // Physics code
    }
    
    {
        let _scope = ProfileScope::new("render");
        // Rendering code
    }
    
    profiler.end_frame();
}

profiler.end_trace_export("trace.json")?;
```

## GPU Profiling

```rust
use praxis_profiling::GpuProfiler;

let gpu_profiler = GpuProfiler::new(device.clone(), queue.clone(), 128, 3)?;
profiler.setup_gpu_profiler(gpu_profiler);

// In rendering code
if let Some((pool, start, end)) = gpu_profiler.begin_query("main_pass") {
    GpuProfiler::write_timestamp(&mut builder, pool.clone(), start, ...)?;
    // Rendering commands
    GpuProfiler::write_timestamp(&mut builder, pool, end, ...)?;
}
```

## Memory Tracking

```rust
let tracker = profiler.memory_tracker();

let id = tracker.track_allocation(1024 * 1024, "vertex_buffer".into(), "Rendering".into());
// Use memory
tracker.track_deallocation(id);

let stats = tracker.statistics();
println!("Current: {} bytes, Peak: {} bytes", 
    stats.current_allocated, stats.peak_allocated);
```

## Bottleneck Detection

```rust
let system_profiler = profiler.system_profiler();

{
    let _guard = SystemProfileScope::new(&system_profiler, "physics_system");
    // System code
}

let bottlenecks = system_profiler.identify_bottlenecks();
for bottleneck in bottlenecks {
    println!("{}: {:.1}% - {}", 
        bottleneck.name, bottleneck.percentage, bottleneck.recommendation);
}
```

## Rendering Statistics Integration

When the `graphics_integration` feature is enabled (default), rendering metrics are automatically exported to Chrome traces:

```rust
use praxis_profiling::Profiler;
use std::time::Instant;

let mut profiler = Profiler::new(ProfilerConfig::default());
profiler.begin_trace_export();

// In render loop
let render_stats = render_context.current_render_stats();
profiler.record_render_stats(&render_stats, Instant::now());

profiler.end_trace_export("trace.json")?;
```

**Exported Metrics:**
- Culling efficiency (% of objects culled)
- Draw call reduction (batching effectiveness)
- Visible object counts
- Frustum and occlusion culling breakdown
- LOD distribution across levels
- Mesh streaming queue depth

See [RENDER_STATS_INTEGRATION.md](RENDER_STATS_INTEGRATION.md) for details.

## Chrome Tracing

1. `profiler.begin_trace_export()`
2. Run application
3. `profiler.end_trace_export("trace.json")`
4. Open in chrome://tracing or ui.perfetto.dev

## Documentation

**Comprehensive Guide:**
- [Profiling Guide](../../docs/profiling.md) - Complete profiling guide

## Examples

```bash
# Comprehensive profiling demo with three progressive sections:
# 1. Basic Profiling - Core features and simple usage
# 2. Advanced Profiling - Visualization and detailed analysis
# 3. Production Patterns - Real-world integration and best practices
cargo run --example profiling_demo
```

## Dependencies

- `tracing`: Structured logging
- `vulkano`: GPU profiling (optional)
