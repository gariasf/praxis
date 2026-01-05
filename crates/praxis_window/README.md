# Praxis Window

Window management and event handling for the Praxis game engine using winit.

## Features

- **Window Creation**: Create and configure game windows with customizable properties
- **Event Handling**: Process window events (resize, focus, close, etc.)
- **Surface Management**: Vulkan surface creation for rendering
- **Input Integration**: Forward input events to the input system
- **Multi-Platform**: Windows, macOS, and Linux support via winit

## Usage

### Basic Window Creation

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

### Event Loop Integration

```rust
use praxis_window::WindowEvent;
use winit::event_loop::{EventLoop, ControlFlow};

let event_loop = EventLoop::new()?;
let window = create_window(&event_loop)?;

event_loop.run(move |event, elwt| {
    match event {
        Event::WindowEvent { event, .. } => {
            match event {
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                WindowEvent::Resized(size) => {
                    // Handle window resize
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    // Forward to input system
                }
                _ => {}
            }
        }
        Event::AboutToWait => {
            // Run frame update
            window.request_redraw();
        }
        _ => {}
    }
})?;
```

### Vulkan Surface Creation

```rust
use praxis_window::create_vulkan_surface;
use vulkano::instance::Instance;

let instance = Instance::new(/* ... */)?;
let surface = create_vulkan_surface(&instance, &window)?;
```

## Window Configuration

### WindowConfig

```rust
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub fullscreen: bool,
    pub vsync: bool,
    pub decorations: bool,
    pub transparent: bool,
    pub always_on_top: bool,
}
```

### Default Configuration

```rust
let config = WindowConfig::default();
// title: "Praxis Engine"
// width: 1280
// height: 720
// resizable: true
// fullscreen: false
// vsync: true
```

## Event Types

The window system processes and forwards these events:

- **Window Events**: Resize, close, focus, move
- **Input Events**: Keyboard, mouse button, cursor movement, scroll
- **System Events**: Resume, suspend, scale factor changed
- **Device Events**: Raw input device events

## Integration with Other Systems

### Graphics

```rust
use praxis_graphics::RenderContext;
use praxis_window::Window;

let window = Window::new()?;
let graphics = RenderContext::new(&window)?;

// Window resize handling
fn on_resize(graphics: &mut RenderContext, new_size: (u32, u32)) {
    graphics.handle_resize(new_size)?;
}
```

### Input

```rust
use praxis_input::InputState;
use praxis_window::Window;

let mut input = InputState::default();

// In event loop
match event {
    WindowEvent::KeyboardInput { event, .. } => {
        praxis_input::winit_integration::process_key_event(&mut input, &event);
    }
    WindowEvent::MouseInput { state, button, .. } => {
        praxis_input::winit_integration::process_mouse_button(
            &mut input,
            button,
            state
        );
    }
    _ => {}
}
```

### GUI

```rust
use praxis_gui::GuiState;
use praxis_window::Window;

let window = Window::new()?;
let mut gui = GuiState::new(&window)?;

// In event loop
match event {
    WindowEvent::.. => {
        gui.handle_event(&window, &event);
    }
    _ => {}
}
```

## Platform-Specific Features

### Windows

- Native window decorations
- DPI awareness
- High DPI support

### macOS

- Retina display support
- Native fullscreen mode
- System integration

### Linux

- X11 and Wayland support
- Multiple display servers
- System tray integration

## Multi-Monitor Support

```rust
use winit::monitor::MonitorHandle;

// Get primary monitor
let monitor = window.primary_monitor()?;

// Get all monitors
let monitors: Vec<MonitorHandle> = window.available_monitors().collect();

// Set fullscreen on specific monitor
window.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
```

## Window Management

### Window State

```rust
// Set title
window.set_title("New Title");

// Set size
window.set_inner_size(winit::dpi::PhysicalSize::new(1920, 1080));

// Toggle fullscreen
window.set_fullscreen(Some(Fullscreen::Borderless(None)));
window.set_fullscreen(None); // Exit fullscreen

// Minimize/maximize
window.set_minimized(true);
window.set_maximized(true);

// Visibility
window.set_visible(true);
```

## Performance Considerations

- **Event batching**: winit batches events for efficiency
- **VSync**: Enable VSync to prevent tearing and reduce GPU load
- **Resize handling**: Recreate swapchain on resize for proper display
- **Input polling**: Use event-driven input handling, not polling

## Dependencies

- `winit` 0.30.11: Cross-platform window management
- `pollster` 0.4: Block on async operations
- `praxis_utils`: Error handling and logging
- `praxis_graphics`: Vulkan surface integration
- `praxis_math`: Math types for window coordinates
- `praxis_ecs`: ECS integration for window state
- `praxis_gui`: GUI event forwarding

## Examples

See window usage in all examples:

```bash
cargo run --example comprehensive_scene_demo
cargo run --example gui_demo
cargo run --example editor_demo
```

## See Also

- [winit Documentation](https://docs.rs/winit)
- [Input System](../praxis_input/README.md)
- [Graphics System](../praxis_graphics/README.md)
- [GUI System](../praxis_gui/README.md)
