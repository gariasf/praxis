# praxis_window

Window management for Praxis engine using winit.

## Overview

Provides cross-platform window creation and event handling through `winit`.

## Features

- Cross-platform window creation
- Event handling (resize, close, focus, etc.)
- Raw window handle for Vulkan
- Monitor and display management
- High-DPI support

## Example

```rust
use praxis_window::winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};

let event_loop = EventLoop::new()?;
let window = WindowBuilder::new()
    .with_title("Praxis Engine")
    .with_inner_size((1920, 1080))
    .build(&event_loop)?;
```

## Dependencies

- `winit`: Window creation and event handling
- `raw-window-handle`: Raw window handle for graphics APIs

## Usage

```toml
praxis_window = { path = "../praxis_window", version = "0.1.0" }
```
