# praxis_window Design Document

## Overview

The `praxis_window` crate provides cross-platform window management for the Praxis engine. It is deliberately decoupled from graphics initialization to avoid circular dependencies and provide architectural flexibility.

## Design Principles

### 1. Separation of Concerns

The crate follows a strict separation between windowing and graphics:

- **praxis_window**: Window creation, event loop, input events, resize handling
- **praxis_graphics**: Vulkan surface creation, swapchain, rendering pipeline

This separation:
- Avoids circular dependencies (window ↔ graphics)
- Allows graphics backends to be swapped without changing window code
- Enables headless testing and non-graphics applications
- Follows professional engine architecture (Unity, Unreal, Godot)

### 2. Modern winit Integration

Uses winit 0.30+ with the `ApplicationHandler` trait pattern:

```rust
pub trait ApplicationHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop);
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent);
}
```

This pattern:
- Supports mobile platforms (iOS/Android suspend/resume)
- Provides better control over event loop lifecycle
- Aligns with modern async/await patterns

### 3. Builder Pattern

Configuration uses the builder pattern for ergonomic API:

```rust
let config = WindowConfig::default()
    .with_title("My Game")
    .with_size(1920, 1080)
    .with_resizable(true)
    .with_vsync(true);
```

All configuration fields have sensible defaults.

### 4. Event Handler Abstraction

Users implement the `WindowEventHandler` trait instead of dealing with raw winit events:

```rust
pub trait WindowEventHandler {
    fn on_init(&mut self, window: &Window) {}
    fn on_update(&mut self, delta_time: f32) {}
    fn on_render(&mut self, window: &Window) {}
    fn on_resize(&mut self, width: u32, height: u32) {}
    // ... more callbacks
}
```

This abstraction:
- Simplifies event handling
- Provides clear lifecycle hooks
- Hides winit implementation details
- Enables future platform changes

## Architecture

### Module Structure

```
praxis_window/
├── src/
│   ├── lib.rs              # Public API and documentation
│   ├── config.rs           # WindowConfig builder
│   ├── event_handler.rs    # WindowEventHandler trait
│   ├── manager.rs          # WindowManager and ApplicationHandler impl
│   ├── window_events.rs    # Resize strategies and event types
│   └── utils.rs            # Utility functions (DPI, aspect ratio, etc.)
├── examples/
│   ├── basic_window.rs     # Simple window example
│   └── window_with_graphics.rs  # Integration with praxis_graphics
└── README.md
```

### Data Flow

```
User Application
      ↓
WindowConfig (builder)
      ↓
WindowManager::with_handler(config, handler)
      ↓
WindowManager::run()
      ↓
Event Loop (winit)
      ↓
ApplicationHandler::resumed() → Window Creation
      ↓
handler.on_init(window) → User Initialization
      ↓
┌─────────────────────────┐
│  Main Loop (Poll mode)  │
│                         │
│  ApplicationHandler::   │
│    window_event()       │
│         ↓               │
│  RedrawRequested →      │
│    handler.on_update()  │
│    handler.on_render()  │
│         ↓               │
│  Resized →              │
│    Debounce →           │
│    handler.on_resize()  │
│         ↓               │
│  KeyboardInput →        │
│    handler.on_key_*()   │
│         ↓               │
│  CloseRequested →       │
│    handler.on_close()   │
│         ↓               │
│  Exit Loop              │
└─────────────────────────┘
```

## Key Features

### Resize Debouncing

Window resizing generates many events during drag operations (can be 100+ events per second). This crate implements intelligent debouncing:

#### Strategies

1. **Immediate**: Process every resize event
   - Most responsive
   - Can cause hundreds of swapchain recreations
   - Use: Applications where responsiveness is critical

2. **Debounced** (default): Wait for inactivity
   - Waits 16ms (1 frame at 60 FPS) after last resize
   - Reduces recreations from hundreds to just a few
   - Use: Most applications (recommended)

3. **OnDragEnd**: Process only when drag completes
   - Minimal recreations (typically one)
   - Content frozen during resize
   - Use: Very expensive resize operations

#### Implementation

```rust
struct WindowState {
    current_size: PhysicalSize<u32>,
    pending_resize: Option<PendingResize>,
    // ...
}

// On resize event:
state.pending_resize = Some(PendingResize::new(new_size));

// On RedrawRequested:
if let Some(pending) = state.pending_resize {
    if pending.is_ready(resize_strategy) {
        state.current_size = pending.size;
        handler.on_resize(pending.size.width, pending.size.height);
        state.pending_resize = None;
    }
}
```

### Zero-Size Window Handling

Minimized windows report 0×0 size, which is invalid for graphics operations (Vulkan, DirectX, Metal).

#### Handling Strategy

1. **Detection**: Check `width > 0 && height > 0` before processing
2. **Skip Rendering**: Don't call `on_render()` for zero-size windows
3. **Skip Resize**: Don't call `on_resize()` for zero-size
4. **Continue Event Loop**: Still process other events (keyboard, focus, etc.)

```rust
fn should_render(&self) -> bool {
    self.current_size.width > 0 && self.current_size.height > 0
}

// In RedrawRequested handler:
if state.should_render() {
    handler.on_update(delta_time);
    handler.on_render(&state.window);
}
```

### Delta Time Tracking

Frame timing is managed internally:

```rust
let now = Instant::now();
let delta_time = now.duration_since(state.last_frame_time).as_secs_f32();
state.last_frame_time = now;

handler.on_update(delta_time);
```

Users receive accurate delta time without managing timers.

### Raw Window Handle Integration

Windows implement `raw-window-handle` traits for graphics API integration:

```rust
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

fn create_vulkan_surface(window: &Window) -> VkSurfaceKHR {
    let window_handle = window.window_handle()?;
    let display_handle = window.display_handle()?;
    
    // Create platform-specific Vulkan surface
    // (Actual implementation in praxis_graphics)
}
```

Platform-specific handles:
- Windows: HWND
- macOS: NSView/NSWindow
- Linux: XWindow (X11) or WlSurface (Wayland)
- Android: ANativeWindow
- iOS: UIView

## Graphics Integration Pattern

### Problem: Lifetime Management

RenderContext needs `Arc<Window>`, but WindowEventHandler receives `&Window`.

### Solution: Careful Arc Management

```rust
impl WindowEventHandler for GraphicsApp {
    fn on_init(&mut self, window: &Window) {
        // Convert reference to Arc (careful!)
        let window_ptr = window as *const Window;
        let window_arc = unsafe { Arc::from_raw(window_ptr) };
        
        // Clone for RenderContext
        let window_clone = Arc::clone(&window_arc);
        std::mem::forget(window_arc);  // Don't drop original
        
        // Create graphics context
        self.render_context = Some(RenderContext::new(window_clone).await?);
    }
}
```

**Note**: This pattern is not ideal and might be improved in future versions. Consider providing window as Arc in the trait.

## Platform Considerations

### Desktop (Windows, macOS, Linux)

- Event loop runs until explicitly exited
- Windows persist across suspend/resume
- `resumed()` called once at startup
- Full control over window lifecycle

### Mobile (iOS, Android)

- OS controls app lifecycle
- Apps suspended/resumed frequently
- `resumed()` called multiple times
- Windows destroyed on suspend, recreated on resume
- Must handle state persistence

### Thread Safety

- **Windows/Linux**: Can create windows on any thread
- **macOS**: MUST create windows on main thread
- **Recommendation**: Always use main thread for maximum compatibility

## Future Enhancements

### Potential Improvements

1. **Multi-Window Support**
   - Currently single-window only
   - Would require tracking window IDs
   - Useful for editor/game separation

2. **Better Graphics Integration**
   - Pass `Arc<Window>` instead of `&Window` to handlers
   - Avoid unsafe Arc manipulation
   - Cleaner lifetime management

3. **Monitor/Display Management**
   - Query available monitors
   - Get monitor properties (resolution, refresh rate)
   - Position windows on specific monitors
   - Fullscreen mode support

4. **Advanced Input**
   - Gamepad support (via winit)
   - Touch input (mobile)
   - IME (Input Method Editor) for text input

5. **Custom Cursors**
   - Load custom cursor images
   - Hide/show cursor
   - Capture cursor (FPS games)

6. **Window Icons**
   - Set window/taskbar icon
   - Animated icons (Windows)

7. **Drag and Drop**
   - File drag and drop support
   - Custom drag data

## Testing Strategy

### Unit Tests

- Configuration builder patterns
- Resize strategy logic
- Utility functions (DPI conversion, aspect ratio, etc.)

### Integration Tests

- Window creation and destruction
- Event delivery
- Resize debouncing behavior
- Zero-size window handling

### Example-Based Testing

- `basic_window.rs`: Manual testing of window features
- `window_with_graphics.rs`: Graphics integration testing

## Dependencies

- **winit** (0.30.11): Core windowing functionality
- **raw-window-handle** (0.6): Graphics API integration
- **praxis_utils**: Logging, error handling
- **praxis_math**: Math types (for future DPI/coordinate features)

## Comparison with Other Engines

### Unity

- Unity abstracts windowing completely
- No direct window access
- Platform-specific builds

### Unreal

- Separates windowing (Slate) from rendering
- Similar architecture to our approach
- More complex due to editor requirements

### Godot

- `DisplayServer` abstraction
- Similar separation of concerns
- More integrated with engine core

### Our Approach

- Explicit separation (praxis_window / praxis_graphics)
- User-facing API (WindowEventHandler trait)
- Educational transparency (clear architecture)

## Conclusion

The `praxis_window` crate provides a clean, decoupled windowing system that:
- Avoids circular dependencies
- Provides modern winit integration
- Offers intuitive event handling
- Supports future extensibility
- Maintains educational clarity

This architecture serves as a foundation for the rest of the Praxis engine while remaining flexible enough for future enhancements.
