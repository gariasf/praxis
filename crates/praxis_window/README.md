# praxis_window

Window management for the Praxis engine using winit.

## Overview

Provides cross-platform window creation, event loop management, resize handling, and platform abstraction for graphics backends. Deliberately decoupled from graphics initialization to avoid circular dependencies.

## Features

- **Window Creation**: Configurable window creation with builder pattern
- **Event Loop Management**: ApplicationHandler-based event loop with Poll/Wait control flow
- **Resize Handling**: Debounced resize events to minimize expensive resource recreations
- **Platform Abstraction**: Raw window handles for Vulkan/DirectX/Metal integration
- **Input Events**: Keyboard, mouse, and focus event handling
- **Frame Timing**: Delta time tracking for smooth animations

## Architecture

The windowing system is intentionally decoupled from graphics initialization:

```
praxis_window (this crate)
    ├── Window creation and configuration
    ├── Event loop management
    ├── Input event handling
    └── Raw window handle provision
    
praxis_graphics (separate crate)
    ├── Vulkan surface creation (from window handle)
    ├── Swapchain creation and management
    └── Rendering pipeline
```

This separation:
- Avoids circular dependencies between window and graphics crates
- Allows graphics backends to be swapped without changing window code
- Enables headless testing and non-graphics applications
- Follows professional engine architecture patterns

## Usage

### Basic Window

```rust
use praxis_window::{WindowConfig, WindowManager};

fn main() -> praxis_utils::Result<()> {
    praxis_utils::init()?;
    
    let config = WindowConfig::default()
        .with_title("My Window")
        .with_size(1280, 720);
    
    let manager = WindowManager::new(config)?;
    manager.run()?;
    
    Ok(())
}
```

### With Custom Event Handler

```rust
use praxis_window::{WindowConfig, WindowManager, WindowEventHandler, Window};

struct MyApp {
    frame_count: u32,
}

impl WindowEventHandler for MyApp {
    fn on_init(&mut self, window: &Window) {
        println!("Window created: {:?}", window.inner_size());
    }
    
    fn on_update(&mut self, delta_time: f32) {
        // Update game logic
    }
    
    fn on_render(&mut self, _window: &Window) {
        self.frame_count += 1;
        // Submit graphics commands
    }
    
    fn on_resize(&mut self, width: u32, height: u32) {
        println!("Resized to {}x{}", width, height);
        // Recreate swapchain, update camera aspect ratio, etc.
    }
}

fn main() -> praxis_utils::Result<()> {
    praxis_utils::init()?;
    
    let app = MyApp { frame_count: 0 };
    let manager = WindowManager::with_handler(WindowConfig::default(), app)?;
    manager.run()?;
    
    Ok(())
}
```

### Integration with Graphics

```rust
use praxis_window::{WindowConfig, WindowManager, WindowEventHandler};
use praxis_graphics::RenderContext;
use std::sync::Arc;

struct GraphicsApp {
    render_context: Option<RenderContext>,
}

impl WindowEventHandler for GraphicsApp {
    fn on_init(&mut self, window: &winit::window::Window) {
        // Initialize graphics using window handle
        let window_arc = Arc::new(window);
        self.render_context = Some(
            pollster::block_on(RenderContext::new(window_arc))
                .expect("Failed to initialize graphics")
        );
    }
    
    fn on_render(&mut self, _window: &winit::window::Window) {
        if let Some(ctx) = &mut self.render_context {
            // Render frame
            ctx.render(&render_commands).unwrap();
        }
    }
    
    fn on_resize(&mut self, width: u32, height: u32) {
        if let Some(ctx) = &mut self.render_context {
            ctx.configure_surface(width, height);
        }
    }
}
```

## Resize Debouncing

Window resizing generates many events during drag operations. This crate implements debouncing to reduce expensive resource recreations:

```rust
use praxis_window::{WindowManager, WindowResizeStrategy};

let manager = WindowManager::new(config)?
    .with_resize_strategy(WindowResizeStrategy::Debounced(
        std::time::Duration::from_millis(16) // ~1 frame at 60 FPS
    ));
```

Strategies:
- `Immediate`: Process every resize (most responsive, most expensive)
- `Debounced(duration)`: Wait for inactivity (recommended default)
- `OnDragEnd`: Process only when drag completes (least expensive, frozen content)

## Raw Window Handles

Windows implement `raw-window-handle` traits for graphics integration:

```rust
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

fn create_vulkan_surface(window: &Window) {
    let window_handle = window.window_handle().unwrap();
    let display_handle = window.display_handle().unwrap();
    
    // Pass to Vulkan surface creation
    // (This is done in praxis_graphics, not here)
}
```

## Event Handler Methods

The `WindowEventHandler` trait provides these callbacks:

- `on_init(&mut self, window: &Window)` - Called after window creation
- `on_update(&mut self, delta_time: f32)` - Called each frame before rendering
- `on_render(&mut self, window: &Window)` - Called to render frame
- `on_resize(&mut self, width: u32, height: u32)` - Called on window resize (debounced)
- `on_close(&mut self) -> bool` - Called on close request (return false to prevent)
- `on_focused(&mut self)` - Called when window gains focus
- `on_unfocused(&mut self)` - Called when window loses focus
- `on_key_pressed(&mut self, key: Key, is_repeat: bool)` - Keyboard input
- `on_key_released(&mut self, key: Key)` - Keyboard input
- `on_mouse_moved(&mut self, x: f64, y: f64)` - Mouse motion
- `on_mouse_button_pressed(&mut self, button: MouseButton)` - Mouse input
- `on_mouse_button_released(&mut self, button: MouseButton)` - Mouse input
- `on_mouse_wheel(&mut self, delta_x: f32, delta_y: f32)` - Mouse scroll

All methods have default no-op implementations.

## Dependencies

- `winit` (0.30): Cross-platform window creation and event handling
- `raw-window-handle` (0.6): Raw window handle traits for graphics integration
- `praxis_utils`: Logging and error handling
- `praxis_math`: Math types (future use for DPI scaling, etc.)

## Platform Notes

### Desktop (Windows, macOS, Linux)
- Event loop runs until explicitly exited
- Windows persist across suspend/resume
- `resumed()` called once at startup

### Mobile (iOS, Android)
- OS controls app lifecycle
- Apps can be suspended/resumed frequently
- `resumed()` may be called multiple times
- Windows destroyed on suspend, recreated on resume

### Thread Safety
For maximum compatibility, create windows and event loops on the main thread. This is required on macOS.
