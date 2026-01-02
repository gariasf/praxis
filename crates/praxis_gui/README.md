# Praxis GUI

GUI system for the Praxis game engine, providing debug UI, entity inspection, and transform gizmos using egui.

## Features

- **FPS Counter**: Real-time performance monitoring overlay
- **Performance Metrics**: Detailed frame timing and statistics
- **Entity Inspector**: Browse and edit ECS component data at runtime
- **Transform Gizmos**: Interactive scene editing tools for translation, rotation, and scaling

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
