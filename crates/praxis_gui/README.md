# praxis_gui

Immediate-mode GUI for Praxis engine using egui.

## Overview

Integrates egui immediate-mode GUI with Vulkan rendering for editor tools and debug UI.

## Features

- **Immediate-Mode**: No retained state, easy to use
- **Vulkan Integration**: Renders via `egui_vulkano`
- **Window Management**: Windows, panels, menus
- **Widgets**: Buttons, sliders, text, images, plots
- **Styling**: Customizable themes
- **Input Handling**: Mouse, keyboard, touch

## Example

```rust
use praxis_gui::{egui, EguiRenderer};

// Create renderer
let mut egui_renderer = EguiRenderer::new(device, queue, surface_format);

// Render UI
egui_renderer.run(|ctx| {
    egui::Window::new("Debug").show(ctx, |ui| {
        ui.label(format!("FPS: {}", fps));
        ui.separator();
        
        if ui.button("Reset").clicked() {
            // Reset
        }
        
        ui.add(egui::Slider::new(&mut value, 0.0..=1.0)
            .text("Speed"));
    });
});
```

## Common Patterns

### Inspector Panel

```rust
egui::SidePanel::left("inspector").show(ctx, |ui| {
    ui.heading("Inspector");
    ui.label("Transform");
    ui.add(egui::DragValue::new(&mut pos.x).prefix("X: "));
    ui.add(egui::DragValue::new(&mut pos.y).prefix("Y: "));
    ui.add(egui::DragValue::new(&mut pos.z).prefix("Z: "));
});
```

### Menu Bar

```rust
egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("New").clicked() { /* ... */ }
            if ui.button("Open").clicked() { /* ... */ }
        });
    });
});
```

## Dependencies

- `egui`: Immediate-mode GUI library
- `egui_vulkano`: Vulkan integration
- `egui-winit`: Window integration
- `vulkano`: Vulkan bindings

## Usage

```toml
praxis_gui = { path = "../praxis_gui", version = "0.1.0" }
```
