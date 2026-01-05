# Praxis Utils

Utility functions, logging, error handling, and timing for the Praxis game engine.

## Features

- **Error Handling**: Centralized error type with rich context via `color-eyre`
- **Logging**: Structured logging with multiple levels via `tracing`
- **Timing**: High-precision frame timing and delta time calculation
- **Result Type**: Convenient `Result<T>` alias for engine-wide error handling

## Error Handling

### Result Type

```rust
use praxis_utils::Result;

fn load_asset(path: &str) -> Result<Asset> {
    // Returns Result<Asset, color_eyre::Report>
    let data = std::fs::read(path)?;
    Ok(parse_asset(&data)?)
}
```

### Error Context

```rust
use praxis_utils::Result;
use color_eyre::eyre::Context;

fn init_graphics() -> Result<()> {
    create_device()
        .context("Failed to create Vulkan device")?;
    
    create_swapchain()
        .context("Failed to create swapchain")?;
    
    Ok(())
}
```

### Error Reporting

```rust
use praxis_utils::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    
    // Errors will print with full context and suggestions
    run_game()?;
    
    Ok(())
}
```

## Logging

### Log Levels

```rust
use tracing::{info, debug, warn, error, trace};

info!("Engine started");
debug!("Loading asset: {}", path);
warn!("Performance warning: frame time {}ms", frame_time);
error!("Failed to initialize audio: {}", err);
trace!("Detailed trace information");
```

### Initialization

```rust
use praxis_utils::init_logging;

fn main() -> praxis_utils::Result<()> {
    // Initialize logging with default settings
    init_logging()?;
    
    info!("Logging initialized");
    
    Ok(())
}
```

### Environment Variables

Control logging via environment variables:

```bash
# Set log level
RUST_LOG=info cargo run
RUST_LOG=debug cargo run
RUST_LOG=praxis_graphics=trace cargo run

# Multiple filters
RUST_LOG=info,praxis_graphics=debug cargo run
```

## Timing

### Delta Time

```rust
use std::time::Instant;

let mut last_frame = Instant::now();

loop {
    let now = Instant::now();
    let delta_time = now.duration_since(last_frame).as_secs_f32();
    last_frame = now;
    
    // Use delta_time for frame-independent updates
    update_physics(delta_time);
    update_animation(delta_time);
}
```

### Frame Rate Calculation

```rust
use std::time::Instant;

struct FrameTimer {
    last_frame: Instant,
    frame_count: u32,
    fps: f32,
}

impl FrameTimer {
    fn update(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        
        self.frame_count += 1;
        if self.frame_count >= 60 {
            self.fps = self.frame_count as f32 / delta;
            self.frame_count = 0;
        }
    }
}
```

## Best Practices

### Error Handling

1. **Use context**: Add context to errors for better debugging
2. **Propagate errors**: Use `?` to propagate errors up the call stack
3. **Handle at boundaries**: Catch errors at API boundaries
4. **Log errors**: Log errors before propagating when appropriate

### Logging

1. **Use appropriate levels**:
   - `trace`: Very detailed information
   - `debug`: Debugging information
   - `info`: General informational messages
   - `warn`: Warning messages for unexpected situations
   - `error`: Error messages for failures

2. **Structured logging**: Include context in log messages
   ```rust
   info!(path = %asset_path, "Loading asset");
   ```

3. **Performance**: Debug/trace logs have minimal overhead when disabled

### Timing

1. **Use delta time**: All time-based updates should use delta time
2. **Avoid system time**: Use `Instant` for game timing, not `SystemTime`
3. **Fixed timestep**: Use fixed timestep for physics simulation
4. **Frame-independent**: Make all updates frame-rate independent

## Dependencies

- `color-eyre` 0.6: Rich error reporting with context
- `tracing` 0.1: Structured logging framework
- `tracing-subscriber` 0.3: Log formatting and filtering

## Examples

All examples use the utilities crate:

```bash
# See error handling in action
cargo run --example comprehensive_scene_demo

# Logging at different levels
RUST_LOG=debug cargo run --example scene_demo
```

## See Also

- [Error Handling Guide](../../docs/guides/error-handling.md)
- [Logging Best Practices](../../docs/guides/logging.md)
- [color-eyre Documentation](https://docs.rs/color-eyre)
- [tracing Documentation](https://docs.rs/tracing)
