# Praxis Profiling

Comprehensive profiling and performance analysis tools for the Praxis game engine.

## Features

### Frame Time Breakdown
- Detailed breakdown of CPU time per frame
- Phase-based categorization (Physics, Rendering, GUI, etc.)
- Rolling statistics with min/max/average tracking
- Per-scope timing with hierarchical nesting

### GPU Profiling
- Vulkan timestamp queries for accurate GPU timing
- Double-buffered query pools for multiple frames
- Per-pass and per-draw-call timing
- Automatic synchronization with CPU timeline

### Memory Tracking
- Allocation tracking with categorization
- Peak memory usage detection
- Per-category allocation statistics
- Memory leak detection with checkpointing

### System Profiling
- ECS system execution time tracking
- Bottleneck identification with severity scoring
- Query size and frequency analysis
- Performance recommendations

### Chrome Tracing Export
- Export to Chrome Trace Event Format
- View in chrome://tracing or Perfetto
- Combined CPU and GPU timeline
- Memory counters and frame markers

## Quick Start

```rust
use praxis_profiling::{Profiler, ProfilerConfig, ProfileScope};

// Create profiler
let config = ProfilerConfig::default();
let mut profiler = Profiler::new(config);

// Start tracing (optional)
profiler.begin_trace_export();

// Main loop
loop {
    profiler.begin_frame();

    {
        let _scope = ProfileScope::new("physics_update");
        // Your physics code here
    }

    {
        let _scope = ProfileScope::new("render");
        // Your rendering code here
    }

    profiler.end_frame();

    // Get statistics
    let stats = profiler.statistics();
    println!("FPS: {:.1}, CPU: {:.2}ms", stats.avg_fps, stats.cpu_time_ms);
}

// Export trace
profiler.end_trace_export("trace.json")?;
```

## GPU Profiling

```rust
use praxis_profiling::GpuProfiler;
use vulkano::device::{Device, Queue};

// Setup GPU profiler
let gpu_profiler = GpuProfiler::new(
    device.clone(),
    queue.clone(),
    128, // max queries per frame
    3,   // buffered frames
)?;

profiler.setup_gpu_profiler(gpu_profiler);

// In your rendering code
let mut builder = AutoCommandBufferBuilder::primary(...);

if let Some(gpu_profiler) = profiler.gpu_profiler() {
    let mut gpu_profiler = gpu_profiler.lock();
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
```

## Memory Tracking

```rust
use praxis_profiling::AllocationTracker;

let tracker = profiler.memory_tracker();

// Track allocation
let id = tracker.track_allocation(
    1024 * 1024,
    "vertex_buffer".to_string(),
    "Rendering".to_string(),
);

// ... use the memory ...

// Track deallocation
tracker.track_deallocation(id);

// Get statistics
let stats = tracker.statistics();
println!("Allocated: {} bytes", stats.current_allocated);
println!("Peak: {} bytes", stats.peak_allocated);
```

## Leak Detection

```rust
let leak_detector = profiler.leak_detector();

// Create checkpoint
leak_detector.checkpoint();

// ... run your code ...

// Detect leaks (allocations older than 1 second)
let leaks = leak_detector.detect_leaks(Duration::from_secs(1));
for (id, alloc) in leaks {
    println!("Potential leak: {} bytes at {}", alloc.size, alloc.location);
}
```

## System Profiling

```rust
let system_profiler = profiler.system_profiler();

// Profile a system
system_profiler.begin_system("physics_update");
// ... system code ...
system_profiler.end_system("physics_update");

// Or use RAII guard
{
    let _guard = SystemProfileScope::new(&system_profiler, "render_system");
    // ... system code ...
}

// Identify bottlenecks
let bottlenecks = system_profiler.identify_bottlenecks();
for bottleneck in bottlenecks {
    println!("Bottleneck: {} ({:.1}%)", 
        bottleneck.name, 
        bottleneck.percentage
    );
    println!("  {}", bottleneck.recommendation);
}
```

## Chrome Tracing

The profiler can export data to Chrome Trace Event Format for visualization:

1. Start trace export: `profiler.begin_trace_export()`
2. Run your application with profiling enabled
3. Stop and save: `profiler.end_trace_export("trace.json")`
4. Open the trace file:
   - Chrome: Navigate to `chrome://tracing` and load the JSON file
   - Perfetto: Visit https://ui.perfetto.dev/ and load the file

The trace will show:
- CPU scopes with nesting and timing
- GPU query results (if enabled)
- Memory allocation counters
- Frame markers
- Thread information

## Performance Overhead

The profiling system is designed to have minimal overhead:

- CPU scopes: ~50-100ns per scope
- GPU queries: No CPU overhead, small GPU overhead
- Memory tracking: ~100ns per allocation/deallocation
- System profiling: ~50ns per system

For production builds, you can disable profiling by not calling profiling functions or by using feature flags.

## Best Practices

1. **Use descriptive scope names**: `ProfileScope::new("physics_rigidbody_update")` is better than `ProfileScope::new("update")`

2. **Categorize allocations**: Use meaningful category names for memory tracking to identify where memory is being used

3. **Profile regularly**: Run profiling sessions periodically during development to catch performance regressions early

4. **Focus on bottlenecks**: Use the bottleneck identification to prioritize optimization work

5. **Export traces for analysis**: Chrome tracing provides a powerful visualization tool for understanding performance

6. **Monitor memory trends**: Use leak detection to catch memory leaks during development

## Integration with ECS

The profiler integrates seamlessly with Bevy ECS:

```rust
fn my_system(/* ... */) {
    let _scope = ProfileScope::new("my_system");
    // System logic here
}

// Or use the system profiler directly
fn my_system(profiler: Res<SystemProfiler>) {
    let _guard = SystemProfileScope::new(&profiler, "my_system");
    // System logic here
}
```

## Example

See `examples/profiling_demo.rs` for a complete example demonstrating all features.

```bash
cargo run --example profiling_demo
```

This will:
- Profile 10 simulated frames
- Track memory allocations
- Identify bottlenecks
- Export a Chrome trace to `profiling_trace.json`
- Demonstrate leak detection
