# praxis_profiling

Performance profiling and metrics for Praxis engine.

## Overview

Provides performance measurement, profiling, and metrics collection with Chrome trace export.

## Features

### Timing

- High-resolution timers
- Frame time tracking
- Delta time calculation
- Average, min, max statistics

### Profiling Scopes

- Hierarchical scope tracking
- CPU profiling
- Thread-safe profiling
- Zero-cost when disabled

### Metrics Collection

- FPS tracking
- Frame time distribution
- Memory usage
- Custom metrics

### Chrome Trace Export

- Compatible with chrome://tracing
- Visualize performance in browser
- Thread and scope hierarchy
- Duration and instant events

## Example

```rust
use praxis_profiling::{Profiler, profile_scope};

let mut profiler = Profiler::new();

// Profile a scope
{
    let _scope = profile_scope!("render_frame");
    
    // Nested scopes
    {
        let _scope = profile_scope!("update_transforms");
        // ...
    }
    
    {
        let _scope = profile_scope!("draw_calls");
        // ...
    }
}

// Export trace
profiler.export_chrome_trace("trace.json")?;
```

## Macro Usage

```rust
// Profile function
#[profile_function]
fn expensive_operation() {
    // ...
}

// Profile scope with custom name
profile_scope!("custom_name");

// Profile with metadata
profile_scope!("load_asset", "path" => asset_path);
```

## Integration

```rust
use praxis_profiling::{Profiler, ProfilerConfig};

// Initialize
let config = ProfilerConfig {
    enabled: true,
    max_frames: 300,
    ..Default::default()
};
let mut profiler = Profiler::new_with_config(config);

// Each frame
profiler.begin_frame();
// ... game logic ...
profiler.end_frame();

// Report
let stats = profiler.frame_stats();
println!("Frame time: {:.2}ms", stats.avg_frame_time_ms);
```

## Chrome Trace Format

Open `trace.json` in Chrome:
1. Navigate to `chrome://tracing`
2. Click "Load"
3. Select trace file
4. Visualize performance timeline

## Dependencies

- `serde`: Serialization
- `serde_json`: JSON export
- `web-time`: High-resolution timing
- `rustc-hash`: Fast hash maps
- `parking_lot`: Fast mutexes

## Usage

```toml
praxis_profiling = { path = "../praxis_profiling", version = "0.1.0" }
```
