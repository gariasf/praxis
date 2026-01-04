# Profiling System Implementation

This document describes the comprehensive profiling and performance analysis system implemented for the Praxis engine.

## Overview

The profiling system provides complete performance analysis capabilities including:

- **Frame Time Breakdown**: Detailed per-frame CPU timing with hierarchical scope tracking
- **GPU Profiling**: Vulkan timestamp queries for accurate GPU timing measurements
- **Memory Tracking**: Allocation tracking with categorization and leak detection
- **System Profiling**: ECS system execution time tracking and bottleneck identification
- **Chrome Tracing**: Export to Chrome Trace Event Format for detailed visualization

## Architecture

### Crate Structure

The profiling system is implemented in the `praxis_profiling` crate with the following modules:

```
praxis_profiling/
├── src/
│   ├── lib.rs                 # Public API exports
│   ├── profiler.rs            # Main profiler coordinator
│   ├── scope.rs               # CPU scope tracking
│   ├── frame_breakdown.rs     # Frame time analysis
│   ├── gpu_profiler.rs        # GPU timestamp queries
│   ├── memory_tracker.rs      # Memory allocation tracking
│   ├── system_profiler.rs     # ECS system profiling
│   ├── chrome_trace.rs        # Chrome trace export
│   ├── visualization.rs       # Visualization data structures
│   └── integration.rs         # ECS integration helpers
├── Cargo.toml
└── README.md
```

### Core Components

#### 1. Profiler (`profiler.rs`)

The main coordinator that ties all profiling subsystems together:

- Manages frame lifecycle (`begin_frame()`, `end_frame()`)
- Collects data from all profiling subsystems
- Updates statistics and detects bottlenecks
- Manages Chrome trace export

#### 2. ProfileScope (`scope.rs`)

RAII-based CPU timing scope:

- Automatically measures execution time
- Supports hierarchical nesting
- Thread-safe with per-thread scope stacks
- Minimal overhead (~50-100ns per scope)

#### 3. FrameBreakdown (`frame_breakdown.rs`)

Frame time analysis:

- Categorizes time into phases (Physics, Rendering, GUI, etc.)
- Tracks individual scope timings
- Calculates percentages and statistics
- Rolling statistics over multiple frames

#### 4. GpuProfiler (`gpu_profiler.rs`)

GPU timing using Vulkan timestamp queries:

- Double-buffered query pools for multi-frame latency
- Timestamp period conversion for nanosecond accuracy
- Non-blocking result collection
- Support for nested GPU scopes

#### 5. AllocationTracker (`memory_tracker.rs`)

Memory allocation tracking:

- Per-allocation metadata (size, location, category, timestamp)
- Statistics by category
- Peak usage tracking
- Leak detection with checkpointing

#### 6. SystemProfiler (`system_profiler.rs`)

ECS system performance analysis:

- Per-system execution time tracking
- Average, min, max timing statistics
- Bottleneck identification with severity scoring
- Performance recommendations

#### 7. ChromeTraceExporter (`chrome_trace.rs`)

Export to Chrome Trace Event Format:

- Complete event (duration) events for scopes
- Instant events for frame markers
- Counter events for memory tracking
- Metadata events for process information

#### 8. Visualization (`visualization.rs`)

Data structures for GUI visualization:

- Frame time graphs
- Phase pie charts
- System bar charts
- Memory usage graphs

## Implementation Details

### CPU Profiling

CPU profiling uses a global callback system:

1. When a `ProfileScope` is created, it allocates a unique ID
2. Start time is recorded
3. Parent scope ID is determined from thread-local stack
4. On drop, duration is calculated and callback is invoked
5. Profiler collects scope data and builds frame breakdown

**Thread Safety**: Each thread maintains its own scope stack in a global `Mutex`. The overhead is minimal because scopes are typically short-lived.

### GPU Profiling

GPU profiling uses Vulkan timestamp queries:

1. Create query pools (double-buffered for frame latency)
2. At frame start, switch to next query pool
3. Reset query pool in command buffer
4. Write timestamps at start/end of GPU work
5. After GPU completes, collect results with nanosecond precision
6. Convert using device timestamp period

**Synchronization**: Query results are collected from the previous frame's pool while the current frame executes, avoiding stalls.

### Memory Tracking

Memory tracking is opt-in and manual:

1. Application calls `track_allocation()` with size and metadata
2. Tracker stores allocation in HashMap with unique ID
3. Application calls `track_deallocation()` with ID when freed
4. Statistics are updated in real-time
5. Leak detector can compare snapshots over time

**Use Cases**:
- GPU buffer allocations
- Large temporary buffers
- Asset loading
- Custom allocators

### System Profiling

System profiling tracks ECS system execution:

1. Before system runs, call `begin_system(name)`
2. System executes
3. After system completes, call `end_system(name)`
4. Statistics are updated (rolling average, min, max)
5. Frame percentage is calculated
6. Bottlenecks are identified based on threshold

**Bottleneck Detection**: Systems taking more than the configured threshold (default 15%) of frame time are flagged as bottlenecks with severity scoring and recommendations.

### Chrome Tracing

Chrome traces include:

- **Duration Events**: CPU scopes with start time and duration
- **Complete Events**: GPU queries as complete events
- **Instant Events**: Frame markers for visual alignment
- **Counter Events**: Memory usage over time
- **Metadata Events**: Process and thread information

The trace can be viewed in:
- Chrome: `chrome://tracing`
- Perfetto: https://ui.perfetto.dev/

## Usage Examples

### Basic Profiling

```rust
use praxis_profiling::{Profiler, ProfilerConfig, ProfileScope};

let mut profiler = Profiler::new(ProfilerConfig::default());

loop {
    profiler.begin_frame();

    {
        let _scope = ProfileScope::new("update");
        // Update code
    }

    {
        let _scope = ProfileScope::new("render");
        // Render code
    }

    profiler.end_frame();

    let stats = profiler.statistics();
    println!("FPS: {:.1}", stats.avg_fps);
}
```

### GPU Profiling

```rust
use praxis_profiling::GpuProfiler;

let gpu_profiler = GpuProfiler::new(device, queue, 128, 3)?;
profiler.setup_gpu_profiler(gpu_profiler);

// In rendering code
let mut builder = AutoCommandBufferBuilder::primary(...)?;
gpu_profiler.lock().reset_queries(&mut builder)?;

if let Some((pool, start, end)) = gpu_profiler.lock().begin_query("main_pass") {
    GpuProfiler::write_timestamp(&mut builder, pool.clone(), start, PipelineStage::TopOfPipe)?;
    // Rendering commands
    GpuProfiler::write_timestamp(&mut builder, pool, end, PipelineStage::BottomOfPipe)?;
}
```

### Memory Tracking

```rust
let tracker = profiler.memory_tracker();
let id = tracker.track_allocation(1024 * 1024, "buffer".into(), "Rendering".into());
// Use memory
tracker.track_deallocation(id);
```

### Leak Detection

```rust
let leak_detector = profiler.leak_detector();
leak_detector.checkpoint();
// Run code that might leak
let leaks = leak_detector.detect_leaks(Duration::from_secs(1));
for (id, alloc) in leaks {
    println!("Leak: {} bytes at {}", alloc.size, alloc.location);
}
```

### System Profiling

```rust
let system_profiler = profiler.system_profiler();
system_profiler.begin_system("physics");
// System code
system_profiler.end_system("physics");

let bottlenecks = system_profiler.identify_bottlenecks();
for bottleneck in bottlenecks {
    println!("⚠ {}: {}", bottleneck.name, bottleneck.recommendation);
}
```

### Chrome Trace Export

```rust
profiler.begin_trace_export();
// Run application
profiler.end_trace_export("trace.json")?;
```

## Performance Characteristics

### Overhead

| Operation | Overhead |
|-----------|----------|
| CPU Scope | ~50-100ns |
| GPU Query | <1μs (GPU only) |
| Memory Track | ~100ns |
| System Profile | ~50ns |

### Memory Usage

- **CPU Scopes**: ~200 bytes per active scope
- **GPU Queries**: ~16 bytes per query
- **Memory Tracking**: ~128 bytes per tracked allocation
- **System Stats**: ~256 bytes per system
- **Chrome Trace**: ~200 bytes per event

### Scalability

The profiling system scales well:

- **1000 scopes/frame**: ~100μs overhead
- **100 GPU queries/frame**: ~1μs overhead
- **10000 tracked allocations**: ~1.2MB memory
- **100 systems**: ~25KB memory

## Integration with Engine

### ECS Integration

```rust
use praxis_profiling::{ProfilerResource, SystemProfilerResource};

fn setup(mut commands: Commands) {
    let profiler = Profiler::new(ProfilerConfig::default());
    commands.insert_resource(ProfilerResource::new(profiler));
}

fn my_system(profiler: Res<SystemProfilerResource>) {
    let _guard = SystemProfileScope::new(profiler.profiler(), "my_system");
    // System logic
}
```

### Rendering Integration

The GPU profiler integrates directly with Vulkan command buffers:

```rust
// Reset queries at command buffer start
gpu_profiler.reset_queries(&mut builder)?;

// Profile render passes
if let Some((pool, start, end)) = gpu_profiler.begin_query("shadow_pass") {
    GpuProfiler::write_timestamp(&mut builder, pool.clone(), start, TOP_OF_PIPE)?;
    // Shadow rendering
    GpuProfiler::write_timestamp(&mut builder, pool, end, BOTTOM_OF_PIPE)?;
}

// Collect results after submission
let results = gpu_profiler.collect_results()?;
```

## Testing

The profiling system includes:

- Unit tests for core functionality
- Integration tests with mock ECS
- Performance benchmarks
- Example applications

Run tests:
```bash
cargo test -p praxis_profiling
```

Run examples:
```bash
cargo run --example profiling_demo
cargo run --example profiling_advanced_demo
```

## Future Enhancements

Potential improvements:

1. **Statistical Analysis**: Percentiles (p50, p95, p99) for frame times
2. **Hot Path Detection**: Automatic identification of critical paths
3. **Regression Detection**: Compare profiles between runs
4. **Network Profiling**: Track network I/O and latency
5. **Asset Loading**: Profile asset loading times
6. **Multi-threaded Support**: Better support for parallel systems
7. **Real-time Visualization**: Live profiling display in editor
8. **Profile Comparison**: Diff tool for comparing profiles
9. **Budget System**: Set performance budgets and alert on violations
10. **Auto-optimization**: Suggestions for system reordering

## Documentation

- **User Guide**: `docs/profiling.md`
- **Crate README**: `crates/praxis_profiling/README.md`
- **API Docs**: Run `cargo doc --open -p praxis_profiling`
- **Examples**: `examples/profiling_demo.rs`, `examples/profiling_advanced_demo.rs`

## Dependencies

- `vulkano`: Vulkan timestamp queries
- `bevy_ecs`: ECS integration
- `serde`/`serde_json`: Chrome trace export
- `parking_lot`: Fast mutexes for thread safety

## Conclusion

The profiling system provides comprehensive performance analysis capabilities for the Praxis engine. It's designed to be:

- **Low Overhead**: Minimal impact on performance
- **Easy to Use**: Simple API with RAII patterns
- **Comprehensive**: CPU, GPU, memory, and system profiling
- **Actionable**: Bottleneck detection with recommendations
- **Visualizable**: Chrome trace export for detailed analysis

The system is production-ready and can be used during development or in release builds with minimal overhead.
