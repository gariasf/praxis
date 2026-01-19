# Adaptive Quality System

The Adaptive Quality System automatically adjusts rendering quality settings based on recent frame time history to maintain a target FPS. This allows games to dynamically scale quality to match hardware capabilities and maintain smooth performance.

## Overview

The system monitors frame times and adjusts three key rendering parameters:

1. **LOD Bias**: Controls level-of-detail selection for meshes
2. **Mesh Streaming Priority Threshold**: Controls which meshes are loaded
3. **Shadow Map Resolution**: Dynamically scales shadow quality

## How It Works

The system maintains a moving average of recent frame times (configurable history size, default 60 frames). It compares this average to a target frame time derived from the target FPS:

- **GPU-bound (slow frames)**: When average frame time exceeds the target, the system reduces quality
- **Under budget (fast frames)**: When average frame time is significantly below the target, the system increases quality
- **Stable**: When frame time is within tolerance of the target, no changes are made

## Configuration

```rust
use praxis_graphics::adaptive_quality::{AdaptiveQualityConfig, AdaptiveQualitySystem};

let config = AdaptiveQualityConfig {
    target_fps: 60.0,                             // Target 60 FPS
    frame_history_size: 60,                       // Average over 60 frames
    
    // LOD bias range and adjustment
    min_lod_bias: -1.0,                           // Maximum quality reduction
    max_lod_bias: 0.5,                            // Maximum quality increase
    initial_lod_bias: 0.0,                        // Start neutral
    lod_bias_adjustment_rate: 0.05,               // Change by 0.05 per adjustment
    
    // Mesh streaming thresholds
    min_streaming_priority_threshold: 0.0,        // Load everything
    max_streaming_priority_threshold: 100.0,      // Load only essentials
    initial_streaming_priority_threshold: 10.0,   // Start moderate
    streaming_threshold_adjustment_rate: 5.0,     // Change by 5.0 per adjustment
    
    // Shadow resolution (must be powers of two)
    min_shadow_resolution: 512,                   // Lowest quality
    max_shadow_resolution: 2048,                  // Highest quality
    initial_shadow_resolution: 1024,              // Start at medium
    
    // Thresholds for triggering adjustments
    under_budget_threshold: 0.1,                  // 10% faster = increase quality
    over_budget_threshold: 0.05,                  // 5% slower = reduce quality
    
    // Enable/disable individual adjustments
    enable_lod_adjustment: true,
    enable_streaming_adjustment: true,
    enable_shadow_resolution_adjustment: true,
};

let mut quality_system = AdaptiveQualitySystem::new(config);
```

## Integration Example

```rust
// In your game loop:

// 1. Update the adaptive quality system with frame time
let frame_time_seconds = frame_timer.delta();
quality_system.update(frame_time_seconds);

// 2. Apply LOD bias to the LOD manager
lod_manager.set_global_lod_bias(quality_system.lod_bias());

// 3. Use streaming priority threshold in mesh loading decisions
let streaming_threshold = quality_system.streaming_priority_threshold();
if mesh_priority > streaming_threshold {
    mesh_streaming_system.load_mesh(mesh_id, mesh_data, mesh_priority);
}

// 4. Check if shadow resolution changed and recreate shadow maps if needed
if quality_system.shadow_resolution_changed() {
    let new_resolution = quality_system.shadow_resolution();
    shadow_manager.update_resolution(new_resolution);
    quality_system.clear_shadow_resolution_changed();
}

// 5. Optionally, retrieve statistics for debugging/UI
let stats = quality_system.statistics();
println!("FPS: {:.1}, LOD bias: {:.3}, Shadow res: {}x{}", 
         stats.current_fps, 
         stats.current_lod_bias,
         stats.current_shadow_resolution,
         stats.current_shadow_resolution);
```

## Quality Parameters

### LOD Bias

Controls which LOD level is selected for meshes:

- **Range**: -1.0 (lowest quality) to +1.0 (highest quality)
- **Effect**: 
  - Negative values: Objects use lower-detail LOD levels earlier (better performance)
  - Positive values: Objects use higher-detail LOD levels longer (better quality)
  - Zero: Neutral, uses default LOD distances

### Mesh Streaming Priority Threshold

Controls which meshes are loaded based on their priority:

- **Range**: Configurable, default 0.0 to 100.0
- **Effect**:
  - Lower threshold: More meshes loaded (higher quality, more memory)
  - Higher threshold: Fewer meshes loaded (lower quality, less memory)
- **Usage**: Only load meshes with `priority > threshold`

### Shadow Map Resolution

Controls the size of shadow maps:

- **Range**: Powers of two (e.g., 512, 1024, 2048, 4096)
- **Effect**:
  - Lower resolution: Blockier shadows but better performance
  - Higher resolution: Sharper shadows but more expensive
- **Note**: Changing resolution requires recreating shadow map textures

## Statistics

The system tracks various statistics accessible via `statistics()`:

```rust
pub struct AdaptiveQualityStatistics {
    pub adjustment_count: u64,              // Total adjustments made
    pub reduction_count: u64,               // Times quality was reduced
    pub increase_count: u64,                // Times quality was increased
    pub current_lod_bias: f32,              // Current LOD bias
    pub current_streaming_threshold: f32,   // Current streaming threshold
    pub current_shadow_resolution: u32,     // Current shadow resolution
    pub average_frame_time_ms: f32,         // Average frame time
    pub current_fps: f32,                   // Current FPS
    pub target_fps: f32,                    // Target FPS
}
```

## Best Practices

### History Size

- **Small (10-30 frames)**: Fast adaptation, may be jittery
- **Medium (30-90 frames)**: Good balance, recommended for most games
- **Large (90+ frames)**: Very smooth, slow to adapt

For 60 FPS target, 60 frames = 1 second of history.

### Adjustment Rates

- **LOD bias**: Typically 0.01-0.1 per adjustment
  - Smaller = smoother transitions but slower adaptation
  - Larger = faster adaptation but potentially visible quality changes

- **Streaming threshold**: Typically 1.0-10.0 per adjustment
  - Consider your priority scale when setting this

### Thresholds

- **Under budget**: How much performance headroom before increasing quality
  - 0.1 = 10% faster than target before increasing
  - Lower values = more aggressive quality increases

- **Over budget**: How much over budget before reducing quality
  - 0.05 = 5% slower than target before reducing
  - Lower values = more sensitive to performance drops

### Selective Adjustment

You can disable individual adjustments:

```rust
let config = AdaptiveQualityConfig {
    enable_lod_adjustment: true,
    enable_streaming_adjustment: false,    // Keep streaming constant
    enable_shadow_resolution_adjustment: true,
    ..Default::default()
};
```

## Performance Characteristics

- **CPU overhead**: Minimal (~0.01ms per frame)
  - Ring buffer management
  - Average calculation
  - Conditional quality adjustments

- **Memory usage**: Very small
  - ~2KB for frame history (60 frames * 4 bytes * 8)
  - ~1KB for state and statistics

- **Frame time smoothing**: The moving average filters out individual frame spikes
  - Prevents over-reaction to temporary performance hitches
  - Provides stable quality adjustments

## Debugging

Enable logging to see quality adjustments:

```rust
// In your logging configuration
RUST_LOG=praxis_graphics::adaptive_quality=debug

// Logs will show:
// - Quality reductions: "Reduced LOD bias: 0.000 -> -0.050"
// - Quality increases: "Increased shadow resolution: 512x512 -> 1024x1024"
// - System events: "Adaptive quality system enabled"
```

## Example Output

```
Creating adaptive quality system targeting 60.0 FPS
  LOD bias range: [-1.00, 0.50]
  Streaming priority range: [0.0, 100.0]
  Shadow resolution range: [512x512, 2048x2048]

// After some frames with heavy load:
Reduced LOD bias: 0.000 -> -0.050
Increased streaming threshold: 10.0 -> 15.0
Reduced shadow resolution: 1024x1024 -> 512x512

// Statistics:
Adaptive Quality Statistics:
 - Current FPS: 59.8 (target: 60.0)
 - Avg frame time: 16.72ms
 - LOD bias: -0.150
 - Streaming threshold: 20.0
 - Shadow resolution: 512x512
 - Total adjustments: 8
 - Quality reductions: 6
 - Quality increases: 2
```

## Testing

Run the example to see the system in action:

```bash
cargo run --example adaptive_quality_demo
```

The demo simulates varying frame times and shows how the system adapts.
