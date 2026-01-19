# Praxis Window

Window management and event handling for the Praxis game engine using winit.

## Overview

Cross-platform window creation, event processing, and Vulkan surface integration.

**Key Features:**
- Window creation with customizable properties
- Event handling (resize, focus, close, input)
- Vulkan surface management
- Multi-platform support (Windows, macOS, Linux)
- Multi-monitor support

## Quick Start

```rust
use praxis_window::WindowConfig;

let config = WindowConfig {
    title: "My Game".to_string(),
    width: 1920,
    height: 1080,
    resizable: true,
    fullscreen: false,
    vsync: true,
};

let window = praxis_window::create_window(config)?;
```

## Event Loop Integration

```rust
use winit::event_loop::{EventLoop, ControlFlow};

let event_loop = EventLoop::new()?;
event_loop.set_control_flow(ControlFlow::Poll);

event_loop.run(move |event, elwt| {
    match event {
        Event::WindowEvent { event, .. } => {
            match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(size) => { /* handle resize */ }
                _ => {}
            }
        }
        _ => {}
    }
})?;
```

## Integration

**Graphics:** Vulkan surface creation
**Input:** Forward events to input system
**GUI:** Event forwarding to egui

## Dependencies

- `winit` 0.30.11: Cross-platform windowing
- `vulkano`: Vulkan integration

## API Stability

**Status:** Stable

Window management API is stable. Minor changes may occur to track upstream winit updates. Breaking changes will be documented in the changelog.
