# praxis_utils

Utilities for Praxis engine: logging, error handling, and timing.

## Overview

This crate provides cross-cutting concerns that all other Praxis subsystems depend on:

- **Error Handling**: Beautiful error reports via `color-eyre`
- **Logging**: Structured logging via `tracing` and `tracing-subscriber`
- **Timing**: High-resolution timing via `web-time`

## Features

### Error Handling

```rust
use praxis_utils::color_eyre::{Result, eyre};

fn may_fail() -> Result<()> {
    Err(eyre!("Something went wrong"))
}
```

### Logging

```rust
use praxis_utils::tracing::{info, warn, error};

info!("Engine initialized");
warn!("Frame time exceeded threshold");
error!("Failed to load asset");
```

### Timing

```rust
use praxis_utils::web_time::Instant;

let start = Instant::now();
// ... work ...
let elapsed = start.elapsed();
```

## Dependencies

- `color-eyre`: Error handling with beautiful reports
- `tracing`: Structured, composable logging
- `tracing-subscriber`: Logging configuration
- `web-time`: High-resolution timing (web-compatible)

## Usage

Add to your `Cargo.toml`:

```toml
praxis_utils = { path = "../praxis_utils", version = "0.1.0" }
```
