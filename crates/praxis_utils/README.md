# praxis_utils

Foundational utilities for the Praxis game engine: structured logging, error handling, and frame timing.

## Overview

`praxis_utils` provides cross-cutting concerns that all other Praxis subsystems depend on. It establishes the observability and reliability foundation for the entire engine.

## Features

### 🔍 Structured Logging with `tracing`

Hierarchical, context-aware logging that goes beyond traditional string-based logs:

```rust
use praxis_utils::{info, debug, instrument};

#[instrument]
fn load_scene(name: &str) -> Result<Scene> {
    info!("Loading scene");
    
    // Structured fields (not just strings)
    debug!(asset_count = scene.assets.len(), "Assets loaded");
    
    Ok(scene)
}
```

**Key benefits:**
- Zero-cost abstractions when logging is disabled
- Automatic span timing and context propagation
- Environment-based filtering via `RUST_LOG`
- Pretty-printed output with colors and indentation

### ❗ Beautiful Error Reports with `color-eyre`

Enhanced error handling with automatic error chains, suggestions, and optional backtraces:

```rust
use praxis_utils::{Result, WrapErr};

fn load_shader(path: &str) -> Result<ShaderModule> {
    std::fs::read(path)
        .wrap_err_with(|| format!("Failed to read shader: {}", path))?;
    
    // Automatic error chain display:
    // Error: Failed to load shader 'vertex.glsl'
    // Caused by:
    //    0: Failed to read shader: shaders/vertex.glsl
    //    1: No such file or directory (os error 2)
    
    Ok(shader_module)
}
```

**Custom Result type** for convenience:
```rust
pub type Result<T> = std::result::Result<T, color_eyre::Report>;
```

### ⏱️ High-Precision Frame Timing

Frame-rate independent game loops with delta time tracking and optional FPS limiting:

```rust
use praxis_utils::timing::{FrameTimer, delta_time};

fn main() -> Result<()> {
    let mut timer = FrameTimer::new_with_global();
    timer.set_target_fps(Some(60.0));
    
    loop {
        timer.tick(); // Updates global timing
        
        // All systems can now use delta time
        update_physics(delta_time());
        render();
        
        timer.sleep_if_needed(); // Maintain 60 FPS
    }
}
```

**Global timing accessors:**
- `delta_time()` - Seconds since last frame
- `current_fps()` - Frames per second
- `total_time()` - Time since application start
- `frame_count()` - Total frames rendered

## Initialization

Call `init()` as the first line in `main()`:

```rust
use praxis_utils::{Result, info};

fn main() -> Result<()> {
    // Initialize utilities first
    praxis_utils::init()?;
    
    // Now logging works
    info!("Application started");
    
    // Rest of your application
    run_game()?;
    
    Ok(())
}
```

This initializes:
1. `color-eyre` for enhanced error reporting
2. `tracing` subscriber for structured logging
3. Default log levels (debug globally, info for winit, debug for vulkano)

## Configuration

### Log Levels

Control logging via the `RUST_LOG` environment variable:

```bash
# Global debug level
RUST_LOG=debug cargo run

# Per-crate levels
RUST_LOG=praxis_graphics=trace,praxis_physics=debug,praxis=info cargo run

# Quiet third-party crates
RUST_LOG=warn,praxis=debug cargo run
```

### Custom Layers (Advanced)

For editor tools or specialized logging:

```rust
use praxis_utils::init_tracing_with_layer;

// Create a custom layer (e.g., captures logs to a buffer)
let console_layer = ConsoleLayer::new(log_buffer);

// Initialize with custom layer
init_tracing_with_layer(Some(console_layer))?;
```

## Error Handling Patterns

### Adding Context

```rust
use praxis_utils::{Result, WrapErr, Context};

// Wrap Result errors
fn load_config() -> Result<Config> {
    std::fs::read_to_string("config.toml")
        .wrap_err("Failed to read configuration")?;
    // ...
}

// Convert Option to Result
fn find_entity(id: u32) -> Result<Entity> {
    world.get(id)
        .context("Entity not found")
}
```

### Validation

```rust
use praxis_utils::{Result, ensure, bail};

fn create_texture(width: u32, height: u32) -> Result<Texture> {
    ensure!(width > 0, "Width must be positive");
    ensure!(height <= 8192, "Height exceeds maximum");
    
    if !is_power_of_two(width) {
        bail!("Width must be power of two");
    }
    
    // Create texture...
}
```

## Timing Patterns

### Frame-Rate Independent Movement

```rust
use praxis_utils::timing::delta_time;

fn update_position(transform: &mut Transform, velocity: Vec3) {
    // Movement scales with time, not frame rate
    transform.translation += velocity * delta_time();
}
```

### Fixed Timestep Physics

```rust
const PHYSICS_DT: f32 = 1.0 / 60.0;
let mut accumulator = 0.0;

loop {
    timer.tick();
    accumulator += delta_time();
    
    while accumulator >= PHYSICS_DT {
        update_physics(PHYSICS_DT);
        accumulator -= PHYSICS_DT;
    }
    
    render();
}
```

## Module Documentation

- **`observability`**: Tracing setup and configuration
- **`errors`**: Error handling utilities and extension traits
- **`timing`**: Frame timing and delta time calculation

See the module-level documentation for detailed guides and examples.

## Dependencies

| Crate | Purpose | Version |
|-------|---------|---------|
| `tracing` | Structured logging | 0.1 |
| `tracing-subscriber` | Log configuration | 0.3 |
| `color-eyre` | Enhanced error reports | 0.6 |

## Best Practices

### Logging

- ✅ Use structured fields: `debug!(count = items.len(), "Processing items")`
- ✅ Use `#[instrument]` for automatic span creation
- ✅ Skip large arguments: `#[instrument(skip(buffer))]`
- ❌ Don't log in tight loops (use `trace!` level at most)

### Error Handling

- ✅ Add context at each layer: `.wrap_err("Context message")`
- ✅ Use lazy evaluation: `.wrap_err_with(|| expensive_format())`
- ✅ Provide suggestions: `.suggestion("Try this instead")`
- ❌ Don't use `unwrap()` or `expect()` in production code
- ❌ Don't lose error context by creating new errors

### Timing

- ✅ Use delta time for all time-based calculations
- ✅ Call `timer.tick()` once per frame
- ✅ Consider fixed timestep for physics
- ❌ Don't use frame count for timing (use delta time instead)

## See Also

- [CLAUDE.md](../../CLAUDE.md) - Engine architecture and patterns
- [tracing documentation](https://docs.rs/tracing)
- [color-eyre documentation](https://docs.rs/color-eyre)
- [Fix Your Timestep](https://gafferongames.com/post/fix_your_timestep/)

## License

MIT
