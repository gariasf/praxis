# Profiling API Reference

API reference for CPU/GPU performance profiling and analysis.

## Core Types

### Profiler

Main profiling interface.

```rust
pub struct Profiler { /* ... */ }
```

**Methods:**
- `new(config: ProfilerConfig) -> Self`
- `begin_frame()` - Start new frame
- `end_frame()` - Finish frame, compute metrics
- `begin_scope(name: &str) -> ScopeId` - Start timed scope
- `end_scope(scope_id: ScopeId)` - End scope
- `frame_time() -> f32` - Last frame time in ms
- `average_frame_time() -> f32` - Average over window
- `frame_count() -> u64`
- `begin_trace_export()` - Start recording Chrome trace
- `end_trace_export(path: &str) -> Result<()>` - Write trace file
- `setup_gpu_profiler(gpu: GpuProfiler)`
- `memory_tracker() -> &MemoryTracker`
- `system_profiler() -> &SystemProfiler`

### ProfilerConfig

Configuration for profiler.

```rust
pub struct ProfilerConfig {
    pub enabled: bool,
    pub history_size: usize,          // Frame history for averaging
    pub slow_frame_threshold_ms: f32, // Warning threshold
    pub trace_export: bool,           // Enable trace recording
}
```

**Methods:**
- `default()` - Standard configuration
- `with_history(size: usize)`
- `disabled()` - All profiling disabled

### ProfileScope

RAII scope for automatic timing.

```rust
pub struct ProfileScope<'a> { /* ... */ }
```

**Usage:**
```rust
{
    let _scope = ProfileScope::new("physics_update");
    // Code to profile
} // Automatically ends when dropped
```

## Frame Profiling

### FrameMetrics

Statistics for a single frame.

```rust
pub struct FrameMetrics {
    pub frame_number: u64,
    pub total_time_ms: f32,
    pub scopes: Vec<ScopeMetrics>,
}
```

### ScopeMetrics

Timing data for a profiling scope.

```rust
pub struct ScopeMetrics {
    pub name: String,
    pub start_time: f64,
    pub duration_ms: f32,
    pub parent: Option<usize>,
    pub depth: usize,
}
```

## GPU Profiling

### GpuProfiler

GPU timestamp queries for render profiling.

```rust
pub struct GpuProfiler { /* ... */ }
```

**Methods:**
- `new(device, queue, max_queries: u32, frames_in_flight: u32) -> Result<Self>`
- `begin_query(name: &str) -> Option<(QueryPool, u32, u32)>` - Returns (pool, start_idx, end_idx)
- `write_timestamp(builder, pool, index, ...) -> Result<()>`
- `collect_results() -> Vec<GpuScopeResult>`

**Static Methods:**
- `GpuProfiler::write_timestamp(builder, pool, index, stage)` - Write timestamp in command buffer

### GpuScopeResult

GPU timing result.

```rust
pub struct GpuScopeResult {
    pub name: String,
    pub duration_ms: f32,
}
```

## Memory Profiling

### MemoryTracker

Tracks memory allocations and deallocations.

```rust
pub struct MemoryTracker { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `track_allocation(size: usize, name: String, category: String) -> AllocationId`
- `track_deallocation(id: AllocationId)`
- `statistics() -> MemoryStatistics`
- `allocations_by_category(category: &str) -> Vec<&Allocation>`
- `detect_leaks() -> Vec<&Allocation>`

### MemoryStatistics

Memory usage statistics.

```rust
pub struct MemoryStatistics {
    pub current_allocated: usize,
    pub peak_allocated: usize,
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub allocation_count: usize,
}
```

### Allocation

Individual memory allocation record.

```rust
pub struct Allocation {
    pub id: AllocationId,
    pub size: usize,
    pub name: String,
    pub category: String,
    pub timestamp: Instant,
}
```

## System Profiling

### SystemProfiler

Profiles ECS system execution.

```rust
pub struct SystemProfiler { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `record_system(name: &str, duration_ms: f32)`
- `frame_results() -> &[SystemMetrics]`
- `identify_bottlenecks() -> Vec<BottleneckInfo>`
- `reset_frame()`

### SystemMetrics

Execution metrics for a system.

```rust
pub struct SystemMetrics {
    pub name: String,
    pub duration_ms: f32,
    pub call_count: u32,
}
```

### BottleneckInfo

Bottleneck analysis result.

```rust
pub struct BottleneckInfo {
    pub name: String,
    pub duration_ms: f32,
    pub percentage: f32,
    pub recommendation: String,
}
```

### SystemProfileScope

RAII scope for system profiling.

```rust
pub struct SystemProfileScope<'a> { /* ... */ }
```

**Usage:**
```rust
fn my_system(profiler: Res<SystemProfiler>) {
    let _guard = SystemProfileScope::new(&profiler, "my_system");
    // System code
}
```

## Chrome Tracing

### TraceEvent

Event for Chrome Trace Event Format.

```rust
pub struct TraceEvent {
    pub name: String,
    pub category: String,
    pub timestamp_us: u64,
    pub duration_us: u64,
    pub process_id: u32,
    pub thread_id: u32,
}
```

**Methods:**
- `begin(name, cat, ts)` - Begin event
- `end(name, cat, ts)` - End event
- `complete(name, cat, ts, dur)` - Complete event
- `instant(name, cat, ts)` - Instant event

## Common Patterns

### Basic Frame Profiling

```rust
use praxis_profiling::{Profiler, ProfilerConfig, ProfileScope};

let mut profiler = Profiler::new(ProfilerConfig::default());
world.insert_resource(profiler);

// In game loop
fn frame_system(mut profiler: ResMut<Profiler>) {
    profiler.begin_frame();
    
    {
        let _scope = ProfileScope::new("game_logic");
        // Game logic
    }
    
    {
        let _scope = ProfileScope::new("render");
        // Rendering
    }
    
    profiler.end_frame();
    
    // Check performance
    if profiler.frame_time() > 16.67 {
        warn!("Slow frame: {:.2}ms", profiler.frame_time());
    }
}
```

### GPU Profiling Setup

```rust
use praxis_profiling::GpuProfiler;

// Initialize
let gpu_profiler = GpuProfiler::new(
    device.clone(),
    queue.clone(),
    128,  // max queries
    3,    // frames in flight
)?;

profiler.setup_gpu_profiler(gpu_profiler);

// In rendering code
if let Some((pool, start, end)) = profiler.gpu_profiler().begin_query("main_pass") {
    GpuProfiler::write_timestamp(
        &mut builder,
        pool.clone(),
        start,
        PipelineStage::TopOfPipe,
    )?;
    
    // Rendering commands
    
    GpuProfiler::write_timestamp(
        &mut builder,
        pool,
        end,
        PipelineStage::BottomOfPipe,
    )?;
}

// Collect results
let results = profiler.gpu_profiler().collect_results();
for result in results {
    info!("{}: {:.2}ms", result.name, result.duration_ms);
}
```

### Memory Tracking

```rust
use praxis_profiling::MemoryTracker;

let tracker = profiler.memory_tracker();

// Track allocation
let id = tracker.track_allocation(
    1024 * 1024,
    "vertex_buffer".to_string(),
    "Rendering".to_string(),
);

// Use memory...

// Track deallocation
tracker.track_deallocation(id);

// Check statistics
let stats = tracker.statistics();
println!("Current: {} MB, Peak: {} MB",
    stats.current_allocated / (1024 * 1024),
    stats.peak_allocated / (1024 * 1024)
);

// Detect leaks
let leaks = tracker.detect_leaks();
if !leaks.is_empty() {
    warn!("Found {} memory leaks", leaks.len());
    for leak in leaks {
        warn!("  {}: {} bytes", leak.name, leak.size);
    }
}
```

### System Bottleneck Detection

```rust
use praxis_profiling::{SystemProfiler, SystemProfileScope};

let system_profiler = SystemProfiler::new();
world.insert_resource(system_profiler);

// In each system
fn physics_system(profiler: Res<SystemProfiler>) {
    let _guard = SystemProfileScope::new(&profiler, "physics_system");
    // Physics code
}

// Analyze bottlenecks
fn analyze_performance(profiler: Res<SystemProfiler>) {
    let bottlenecks = profiler.identify_bottlenecks();
    
    for bottleneck in bottlenecks.iter().take(5) {
        warn!("Bottleneck: {}", bottleneck.name);
        warn!("  Time: {:.2}ms ({:.1}%)", 
            bottleneck.duration_ms, 
            bottleneck.percentage);
        warn!("  Recommendation: {}", bottleneck.recommendation);
    }
}
```

### Chrome Trace Export

```rust
// Start recording
profiler.begin_trace_export();

// Run application for some time
for _ in 0..1000 {
    profiler.begin_frame();
    // Application code with ProfileScope usage
    profiler.end_frame();
}

// Export trace
profiler.end_trace_export("trace.json")?;

// View in chrome://tracing or ui.perfetto.dev
```

### Hierarchical Profiling

```rust
fn game_update(mut profiler: ResMut<Profiler>) {
    profiler.begin_frame();
    
    {
        let _game = ProfileScope::new("game_update");
        
        {
            let _physics = ProfileScope::new("physics");
            // Physics code
        }
        
        {
            let _ai = ProfileScope::new("ai");
            // AI code
        }
        
        {
            let _audio = ProfileScope::new("audio");
            // Audio code
        }
    }
    
    {
        let _render = ProfileScope::new("render");
        
        {
            let _culling = ProfileScope::new("culling");
            // Culling code
        }
        
        {
            let _draw = ProfileScope::new("draw_calls");
            // Drawing code
        }
        
        {
            let _post = ProfileScope::new("post_processing");
            // Post-processing code
        }
    }
    
    profiler.end_frame();
}
```

## Performance Guidelines

### Profiling Overhead

- **ProfileScope**: ~50-100ns per scope
- **Memory tracking**: ~200ns per allocation/deallocation
- **GPU queries**: Minimal GPU overhead
- **System profiler**: ~100ns per system

### Best Practices

1. **Keep scopes focused** - Profile specific operations, not entire systems
2. **Use hierarchical scopes** - Nest scopes to identify sub-operations
3. **Disable in release builds** - Use feature flags if needed
4. **Monitor continuously** - Set up alerts for slow frames
5. **Export traces periodically** - Capture problematic frames

### Configuration Tips

```rust
// Development (detailed profiling)
let config = ProfilerConfig {
    enabled: true,
    history_size: 120,  // 2 seconds at 60fps
    slow_frame_threshold_ms: 16.67,
    trace_export: true,
};

// Release (minimal overhead)
let config = ProfilerConfig {
    enabled: true,
    history_size: 60,
    slow_frame_threshold_ms: 33.33,
    trace_export: false,
};

// Disabled (no overhead)
let config = ProfilerConfig::disabled();
```

## See Also

- [Profiling Guide](../profiling.md) - Comprehensive profiling guide
- [praxis_profiling crate](../../crates/praxis_profiling/README.md) - Crate documentation
