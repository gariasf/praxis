# Praxis Utils

Utilities, logging, error handling, and timing for the Praxis game engine.

## Overview

Core utilities for error handling, structured logging, and timing.

**Key Features:**
- Rich error reporting with context (color-eyre)
- Structured logging with levels (tracing)
- High-precision timing utilities
- Engine-wide Result type

## Quick Start

### Error Handling

```rust
use praxis_utils::Result;
use color_eyre::eyre::Context;

fn load_asset(path: &str) -> Result<Asset> {
    let data = std::fs::read(path)
        .context("Failed to read asset file")?;
    Ok(parse_asset(&data)?)
}
```

### Logging

```rust
use tracing::{info, debug, warn, error};

info!("Engine started");
debug!("Loading asset: {}", path);
warn!("Performance warning: frame time {}ms", frame_time);
error!("Failed to initialize: {}", err);
```

### Initialization

```rust
use praxis_utils::init_logging;

fn main() -> praxis_utils::Result<()> {
    init_logging()?;
    info!("Logging initialized");
    Ok(())
}
```

### Environment Variables

```bash
# Set log level
RUST_LOG=info cargo run
RUST_LOG=debug cargo run
RUST_LOG=praxis_graphics=trace cargo run
```

## Timing

```rust
use std::time::Instant;

let mut last_frame = Instant::now();

loop {
    let now = Instant::now();
    let delta_time = now.duration_since(last_frame).as_secs_f32();
    last_frame = now;
    
    update_physics(delta_time);
}
```

## Log Levels

- `trace`: Very detailed (performance-sensitive)
- `debug`: Debugging info
- `info`: General information
- `warn`: Unexpected but recoverable
- `error`: Serious failures

## Documentation

**Guides:**
- [Logging Guide](../../docs/logging.md)

## Dependencies

- `color-eyre` 0.6: Rich error reporting
- `tracing` 0.1: Structured logging
- `tracing-subscriber` 0.3: Log formatting
