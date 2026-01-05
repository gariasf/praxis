# Praxis GUI

GUI system for the Praxis game engine, providing debug UI, entity inspection, and transform gizmos using egui.

## Features

- **FPS Counter**: Real-time performance monitoring overlay
- **Performance Metrics**: Detailed frame timing and statistics
- **Entity Inspector**: Browse and edit ECS component data at runtime
- **Transform Gizmos**: Interactive scene editing tools for translation, rotation, and scaling
- **ImGui Integration**: Immediate mode GUI via egui
- **Vulkan Rendering**: Direct integration with Vulkan rendering pipeline

## Integration

The GUI system integrates with the engine through `GuiState`, which manages all GUI components:

```rust
use praxis_gui::GuiState;
use praxis_ecs::World;

// Initialize GUI state
let mut gui_state = GuiState::new(
    event_loop,
    window.clone(),
    graphics_queue.clone(),
    swapchain_format,
);

// Handle events
gui_state.handle_event(&window, &event);

// Render GUI
gui_state.render(
    &window,
    &mut world,
    swapchain_image_view.clone(),
    render_pass.clone(),
)?;
```

## Components

### Debug UI

Displays FPS counter and performance metrics:

```rust
// Toggle visibility
gui_state.debug_ui.toggle();

// Configure what to show
gui_state.debug_ui.show_fps = true;
gui_state.debug_ui.show_performance = true;
```

### Entity Inspector

Browse and edit entity components:

```rust
// Select an entity
gui_state.entity_inspector.select_entity(entity);

// Toggle visibility
gui_state.entity_inspector.toggle();
```

Supports editing:
- Transform (translation, rotation, scale)
- Name
- Camera settings
- Light properties
- Hierarchy information

### Transform Gizmos

Interactive transform tools for scene editing:

```rust
// Add gizmo to entity
gui_state.transform_gizmos.add_gizmo(entity);

// Set mode
gui_state.transform_gizmos.set_mode(GizmoMode::Translate);
gui_state.transform_gizmos.set_mode(GizmoMode::Rotate);
gui_state.transform_gizmos.set_mode(GizmoMode::Scale);

// Cycle through modes
gui_state.transform_gizmos.cycle_mode();

// Apply transformations
gui_state.transform_gizmos.apply_translation(&mut world, entity, delta);
gui_state.transform_gizmos.apply_rotation(&mut world, entity, quat);
gui_state.transform_gizmos.apply_scale(&mut world, entity, scale);
```

## Keyboard Shortcuts

- `F1`: Toggle debug UI
- `F2`: Toggle entity inspector
- `F3`: Toggle transform gizmos
- `T`: Switch to translate mode
- `R`: Switch to rotate mode
- `S`: Switch to scale mode

## Usage Examples

### Debug Overlay

```rust
use praxis_gui::DebugUI;

let mut debug_ui = DebugUI::new();
debug_ui.show_fps = true;
debug_ui.show_performance = true;

// In render loop
debug_ui.update(delta_time, frame_time_ms);
debug_ui.render(&egui_context);
```

### Entity Inspection

```rust
use praxis_gui::EntityInspector;
use praxis_ecs::World;

let mut inspector = EntityInspector::new();
inspector.select_entity(player_entity);

// In GUI render
inspector.render(&egui_context, &mut world);
```

### Custom GUI Panels

```rust
use egui::Context;

fn render_custom_panel(ctx: &Context) {
    egui::Window::new("Settings")
        .show(ctx, |ui| {
            ui.label("Game Settings");
            ui.separator();
            
            ui.checkbox(&mut settings.vsync, "VSync");
            ui.checkbox(&mut settings.fullscreen, "Fullscreen");
            
            ui.add(egui::Slider::new(&mut settings.volume, 0.0..=1.0)
                .text("Volume"));
        });
}
```

## Requirements

- `egui` 0.29
- `egui-winit` 0.29
- `egui_vulkano` 0.6
- Vulkan-compatible graphics device

## Performance

The GUI system is designed to have minimal performance impact:
- Immediate mode rendering (egui)
- Efficient Vulkan integration
- Minimal draw calls
- Optional visibility controls for all components

## Examples

Run the GUI demo:

```bash
cargo run --example gui_demo
```

This demonstrates:
- Debug UI with FPS counter
- Entity inspector with component editing
- Transform gizmos
- Custom GUI panels
- Keyboard shortcuts

## Dependencies

- `egui` 0.29: Immediate mode GUI framework
- `egui-winit` 0.29: winit integration for egui
- `egui_vulkano` 0.6: Vulkan rendering backend for egui
- `praxis_ecs`: ECS integration
- `praxis_math`: Math types
- `praxis_utils`: Error handling

## See Also

- [Editor System](../praxis_editor/README.md)
- [GUI Demo](../../examples/gui_demo.rs)
- [egui Documentation](https://docs.rs/egui)
