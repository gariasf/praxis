# praxis_utils Implementation

This document describes the complete implementation of the `praxis_utils` foundational utilities crate.

## Overview

The `praxis_utils` crate provides cross-cutting concerns for the Praxis game engine:

1. **Structured Logging** - `tracing` with configured subscribers and environment-based filtering
2. **Error Handling** - `color-eyre` with custom Result types and extension traits
3. **Frame Timing** - High-precision timing with delta calculation and FPS tracking
4. **Observability** - Foundation for debugging and performance monitoring

## Module Structure

```
praxis_utils/
├── src/
│   ├── lib.rs           # Main crate interface with comprehensive documentation
│   ├── observability.rs # Tracing initialization and configuration
│   ├── errors.rs        # Error handling utilities and extension traits
│   └── timing.rs        # Frame timing and delta time calculation
├── Cargo.toml           # Dependencies: tracing, tracing-subscriber, color-eyre
└── README.md            # User-facing documentation
```

## Implementation Details

### 1. Observability Module (`observability.rs`)

**Purpose**: Initialize and configure the `tracing` logging system.

**Key Functions**:
- `init_tracing()` - Standard initialization with default configuration
- `init_tracing_with_layer<L>()` - Advanced initialization with custom layers
- `create_default_filter()` - Internal helper for environment-based filtering

**Configuration**:
- Default log level: `debug` globally
- Quieted crates: `winit=info`, `vulkano=debug`
- Environment override: `RUST_LOG` variable
- Output format: Pretty, with colors, thread IDs, and span events

**Features**:
- Zero-cost logging when disabled
- Hierarchical spans for context
- Structured fields (not just strings)
- Multi-layer composition for custom backends (editor console, files, etc.)

### 2. Errors Module (`errors.rs`)

**Purpose**: Provide error handling utilities and patterns.

**Key Types**:
- `Result<T>` - Alias for `Result<T, color_eyre::Report>`
- `WrapErr<T, E>` trait - Add context to `Result` types
- `Context<T>` trait - Convert `Option` to `Result` with context

**Re-exported Macros**:
- `bail!` - Early return with error
- `ensure!` - Assert condition or return error
- `eyre!` - Create error without returning

**Features**:
- Automatic error conversion via `?` operator
- Lazy context evaluation with `wrap_err_with()`
- Error chain building for multi-layer context
- Beautiful colorized error output
- Optional backtraces with `RUST_BACKTRACE=1`

**Common Patterns**:
```rust
// Add context to errors
result.wrap_err("Operation failed")?;

// Convert Option to Result
option.context("Value not found")?;

// Validation
ensure!(width > 0, "Width must be positive");

// Early return
bail!("Requirements not met");
```

### 3. Timing Module (`timing.rs`)

**Purpose**: Provide frame timing utilities for game loops.

**Key Types**:
- `FrameTimer` - Main timing utility for game loops
- `GlobalTiming` - Internal shared state for global accessors

**Global Accessors**:
- `delta_time()` - Seconds since last frame (f32)
- `delta_duration()` - Delta time as Duration
- `current_fps()` - Frames per second
- `total_time()` - Time since start
- `frame_count()` - Total frames rendered

**Features**:
- Frame-rate independent timing via delta time
- Automatic delta clamping (max 100ms) for stability
- Optional FPS limiting with `set_target_fps()`
- Rolling FPS calculation (1-second window)
- Thread-safe global timing via `OnceLock` + `Mutex`

**Usage Pattern**:
```rust
let mut timer = FrameTimer::new_with_global();
timer.set_target_fps(Some(60.0));

loop {
    timer.tick(); // Updates global timing
    update_systems(delta_time());
    render();
    timer.sleep_if_needed(); // Maintain target FPS
}
```

### 4. Main Library Interface (`lib.rs`)

**Purpose**: Provide unified API and comprehensive documentation.

**Key Function**:
- `init()` - One-call initialization for all utilities

**Re-exports**:
```rust
// Tracing
pub use tracing::{debug, error, info, instrument, trace, warn};

// Error handling
pub use color_eyre::{eyre, Error, Report, Result};
pub use errors::{bail, ensure, Context, WrapErr};

// Initialization
pub use observability::{init_tracing, init_tracing_with_layer};
```

**Documentation**:
- 400+ lines of module-level docs with examples
- Usage patterns and best practices
- Performance considerations
- Integration guidance

## Dependencies

| Crate | Version | Purpose | Features |
|-------|---------|---------|----------|
| `tracing` | 0.1 | Structured logging | Default |
| `tracing-subscriber` | 0.3 | Log configuration | `env-filter`, `fmt`, `ansi` |
| `color-eyre` | 0.6 | Error reports | Default |

All dependencies are widely used, stable, and well-maintained.

## Integration with Engine

### Initialization Order

`praxis_utils::init()` must be called **first** in the engine initialization sequence:

```rust
pub fn run() -> praxis_utils::Result<()> {
    // 1. Utils (this crate) - establishes logging and error handling
    praxis_utils::init()?;
    
    // 2. ECS - requires logging for diagnostics
    praxis_ecs::init()?;
    
    // 3. Other subsystems...
}
```

### Usage in Subsystems

All engine crates depend on `praxis_utils`:

```toml
[dependencies]
praxis_utils = { path = "../praxis_utils", version = "0.1.0" }
```

Common imports:
```rust
use praxis_utils::{info, debug, error, Result, WrapErr};
use praxis_utils::timing::delta_time;
```

## Testing

Tests are embedded in each module using `#[cfg(test)]`:

- `observability.rs`: Initialization tests
- `errors.rs`: Extension trait tests, macro tests
- `timing.rs`: Frame timer tests, clamping tests, global timing tests

Run tests:
```bash
cargo test -p praxis_utils
```

## Documentation

Multiple levels of documentation:

1. **Module docs** (`lib.rs`) - Comprehensive guide with examples
2. **Function docs** - Individual API documentation with examples
3. **README.md** - User-facing quick reference
4. **IMPLEMENTATION.md** (this file) - Technical implementation details

Generate docs:
```bash
cargo doc -p praxis_utils --no-deps --open
```

## Design Rationale

### Why Structured Logging?

Traditional string-based logging loses information. Structured logging with `tracing`:
- Preserves types and relationships
- Enables filtering and analysis
- Provides hierarchical context
- Zero cost when disabled

### Why color-eyre?

`color-eyre` enhances `eyre` with:
- Beautiful colorized output
- Automatic span capture
- Suggestion system
- Consistent error handling patterns

Alternative considered: `anyhow` (simpler but less feature-rich)

### Why Global Timing?

Global timing accessors simplify ECS systems:
- No need to pass timing through every function
- Single source of truth for delta time
- Thread-safe via `OnceLock` + `Mutex`
- Minimal overhead (one lock per frame)

Alternative: Pass timing explicitly (more verbose, no real benefit)

## Best Practices

### Logging

✅ **Do**:
- Use structured fields: `debug!(count = n, "Processing")`
- Use `#[instrument]` for automatic spans
- Skip large args: `#[instrument(skip(buffer))]`
- Use appropriate levels (trace/debug/info/warn/error)

❌ **Don't**:
- Log in tight loops (use `trace!` at most)
- Include secrets in logs
- Use string formatting unnecessarily

### Error Handling

✅ **Do**:
- Add context at each layer: `.wrap_err("Context")`
- Use lazy evaluation: `.wrap_err_with(|| expensive())`
- Prefer `ensure!` over `if` + `bail!`
- Build error chains from low to high level

❌ **Don't**:
- Use `unwrap()` or `expect()` in production
- Lose context by creating new errors
- Include secrets in error messages
- Add redundant context

### Timing

✅ **Do**:
- Call `timer.tick()` once per frame
- Use `delta_time()` for frame-rate independence
- Consider fixed timestep for physics
- Log stats periodically for debugging

❌ **Don't**:
- Use frame count for timing (use delta time)
- Forget to clamp delta time (already done)
- Create multiple global timers

## Future Enhancements

Potential improvements (not currently needed):

1. **Configurable delta clamp** - Allow adjusting max delta per game
2. **Metrics system** - Structured performance metrics
3. **Log filtering helpers** - Dynamic log level control
4. **Async logging** - Non-blocking log writes (if needed)
5. **Custom formatters** - JSON, binary, etc. for production

## Performance Characteristics

- **Logging**: Zero cost when disabled via `tracing` macros
- **Error handling**: `Arc` internally, cheap to clone
- **Global timing**: One `Mutex` lock per frame (negligible)
- **Delta clamping**: Simple comparison, no allocation

No performance concerns identified in profiling.

## Conclusion

The `praxis_utils` crate provides a solid foundation for the Praxis engine:

- ✅ Structured logging with `tracing`
- ✅ Beautiful error reports with `color-eyre`
- ✅ Frame timing with delta calculation
- ✅ Comprehensive documentation
- ✅ Tested and integrated
- ✅ Production-ready patterns

All foundational utilities are implemented and ready for use across the engine.
