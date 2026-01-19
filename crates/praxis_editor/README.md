# Praxis Editor

Editor system with dockable panels, selection, undo/redo, and gizmos for the Praxis game engine.

## Overview

Comprehensive development environment with scene editing, component inspection, and powerful command system.

**Key Features:**
- Dockable panels (hierarchy, inspector, console, assets, scene view)
- Multi-entity selection with raycast and marquee picking
- Full undo/redo system (100 command history)
- Transform gizmos (translate, rotate, scale)
- Dirty state tracking for unsaved changes
- Play mode with scene snapshot/restore
- Console with log filtering and search

## Quick Start

```rust
use praxis_editor::{EditorState, UndoRedoSystem};
use praxis_ecs::World;
use egui::Context as EguiContext;
use color_eyre::Result;

fn main() -> Result<()> {
    // Initialize editor
    praxis_editor::init()?;
    
    // Create editor state
    let mut editor = EditorState::new();
    
    // Create world and undo/redo system
    let mut world = World::new();
    world.insert_resource(UndoRedoSystem::new());
    
    // Game loop
    loop {
        // Get egui context
        let egui_ctx = get_egui_context();
        
        // Render editor UI
        let mut undo_system = world.remove_resource::<UndoRedoSystem>()
            .expect("UndoRedoSystem should exist");
        
        editor.ui(&egui_ctx, Some(&mut undo_system), Some(&mut world));
        
        world.insert_resource(undo_system);
    }
    
    Ok(())
}
```

## Core Systems

### Selection System

```rust
use praxis_editor::{SelectionSystem, Selectable, Selected};
use praxis_ecs::{World, Query, Entity, Transform, With};

fn setup_selection(world: &mut World) {
    // Initialize selection system resource
    world.insert_resource(SelectionSystem::new());
    
    // Spawn selectable entities
    world.spawn((
        Transform::default(),
        Selectable,  // Mark entity as selectable
    ));
    
    world.spawn((
        Transform::default(),
        Selectable,
    ));
}

// System to highlight selected entities
fn highlight_selected_system(
    query: Query<Entity, With<Selected>>
) {
    for entity in query.iter() {
        // Draw outline or change color for selected entities
        println!("Entity {:?} is selected", entity);
    }
}
```

**See**: [SELECTION_SYSTEM.md](SELECTION_SYSTEM.md) for implementation details.

### Undo/Redo Commands

```rust
use praxis_editor::{
    CommandHistory, TransformEditCommand, CreateEntityCommand,
    DeleteEntityCommand, CompositeCommand
};
use praxis_ecs::{World, Entity, Transform};
use praxis_math::Vec3;
use color_eyre::Result;

fn undo_redo_example(world: &mut World) -> Result<()> {
    let mut history = CommandHistory::new();
    
    // Execute a transform edit
    let entity = world.spawn(Transform::default()).id();
    let old_transform = Transform::default();
    let new_transform = Transform::from_xyz(5.0, 0.0, 0.0);
    
    let command = TransformEditCommand::new(
        entity,
        old_transform,
        new_transform
    );
    
    history.execute(world, Box::new(command))?;
    
    // Undo the change
    history.undo(world)?;
    
    // Redo the change
    history.redo(world)?;
    
    Ok(())
}

fn composite_command_example(world: &mut World) -> Result<()> {
    let mut history = CommandHistory::new();
    
    // Group multiple commands into one undoable operation
    let entity = world.spawn(Transform::default()).id();
    
    let mut composite = CompositeCommand::new();
    composite.add(Box::new(TransformEditCommand::new(
        entity,
        Transform::default(),
        Transform::from_xyz(1.0, 0.0, 0.0)
    )));
    composite.add(Box::new(TransformEditCommand::new(
        entity,
        Transform::from_xyz(1.0, 0.0, 0.0),
        Transform::from_xyz(1.0, 2.0, 0.0)
    )));
    
    // Execute composite (both transforms applied)
    history.execute(world, Box::new(composite))?;
    
    // Undo undoes both transforms
    history.undo(world)?;
    
    Ok(())
}
```

**Available Commands:**
- `TransformEditCommand` - Transform changes
- `CreateEntityCommand` / `DeleteEntityCommand` - Entity lifecycle
- `AddComponentCommand` / `RemoveComponentCommand` - Component management
- `SetParentCommand` - Hierarchy changes
- `CompositeCommand` - Grouped operations

**See**: [UNDO_REDO_SYSTEM.md](UNDO_REDO_SYSTEM.md) for implementation details.

### Editor Camera

```rust
use praxis_editor::{EditorCameraController, EditorCamera};
use praxis_ecs::{World, PerspectiveCameraBundle};
use praxis_math::Vec3;

fn setup_editor_camera(world: &mut World) {
    // Initialize camera controller resource
    world.insert_resource(EditorCameraController::new());
    
    // Spawn editor camera entity
    let fov = 60.0;
    let aspect = 16.0 / 9.0;
    
    world.spawn((
        PerspectiveCameraBundle::new(
            Vec3::new(0.0, 5.0, 10.0),  // Position
            fov,
            aspect
        ),
        EditorCamera,  // Marker component
    ));
}
```

**See**: [EDITOR_CAMERA.md](EDITOR_CAMERA.md) for implementation details.

### Transform Gizmos

```rust
use praxis_editor::{GizmoSystem, GizmoMode, GizmoSpace};
use praxis_ecs::World;

fn setup_gizmos(world: &mut World) {
    // Initialize gizmo system
    world.insert_resource(GizmoSystem::new());
}

fn change_gizmo_mode(world: &mut World) {
    let mut gizmo = world.resource_mut::<GizmoSystem>();
    
    // Change gizmo mode
    gizmo.set_mode(GizmoMode::Translate);  // Move
    gizmo.set_mode(GizmoMode::Rotate);     // Rotate
    gizmo.set_mode(GizmoMode::Scale);      // Scale
    
    // Change coordinate space
    gizmo.set_space(GizmoSpace::World);    // World space
    gizmo.set_space(GizmoSpace::Local);    // Local space
}
```

**See**: [GIZMOS.md](GIZMOS.md) for implementation details.

### Menu Bar

```rust
use praxis_editor::menu_bar::{
    render_menu_bar, check_keyboard_shortcuts,
    handle_menu_action, MenuState, MenuAction
};
use praxis_editor::UndoRedoSystem;
use egui::Context as EguiContext;
use praxis_ecs::World;

fn render_editor_menu(
    ctx: &EguiContext,
    menu_state: &mut MenuState,
    undo_system: &UndoRedoSystem
) {
    // Render menu bar and get actions
    let actions = render_menu_bar(ctx, menu_state, Some(undo_system));
    
    // Handle menu actions
    for action in actions {
        match action {
            MenuAction::NewScene => {
                println!("Creating new scene");
            }
            MenuAction::SaveScene => {
                println!("Saving scene");
            }
            MenuAction::Undo => {
                println!("Undo triggered from menu");
            }
            _ => {}
        }
    }
}
```

**See**: [MENU_BAR.md](MENU_BAR.md) for implementation details.

## Documentation

### User Guides (docs/editor/)

High-level guides for using the editor:
- [Editor Overview](../../docs/editor/editor-overview.md) - Architecture and features
- [Selection System Guide](../../docs/editor/selection-system.md) - Using selection
- [Undo/Redo Guide](../../docs/editor/undo-redo.md) - Using undo/redo
- [Inspector Panel](../../docs/editor/inspector.md) - Component editing
- [Hierarchy Panel](../../docs/editor/hierarchy-panel.md) - Entity tree
- [Asset Browser](../../docs/editor/asset-browser.md) - Asset management
- [Editor Camera](../../docs/editor/editor-camera.md) - Camera controls
- [Gizmos](../../docs/editor/gizmos.md) - Transform tools
- [Menu Bar](../../docs/editor/menu-bar.md) - Menu system
- [Panels](../../docs/editor/panels.md) - Panel overview

### Technical Documentation (crates/praxis_editor/)

Implementation details and API reference:
- [SELECTION_SYSTEM.md](SELECTION_SYSTEM.md) - Selection implementation
- [UNDO_REDO_SYSTEM.md](UNDO_REDO_SYSTEM.md) - Command pattern implementation
- [EDITOR_CAMERA.md](EDITOR_CAMERA.md) - Camera controller implementation
- [GIZMOS.md](GIZMOS.md) - Gizmo system implementation
- [MENU_BAR.md](MENU_BAR.md) - Menu system implementation
- [VIEWPORT_PANEL.md](VIEWPORT_PANEL.md) - Viewport rendering
- [COMMAND_SYSTEM.md](COMMAND_SYSTEM.md) - Command architecture
- [COMMANDS_OVERVIEW.md](COMMANDS_OVERVIEW.md) - Command catalog
- [PLAY_MODE_SYSTEM.md](PLAY_MODE_SYSTEM.md) - Play mode implementation
- [TOOLBAR_SYSTEM.md](TOOLBAR_SYSTEM.md) - Toolbar system

## Examples

```bash
# Full editor demonstration
cargo run --example editor_demo

# System-specific examples
cargo run --example selection_demo
cargo run --example undo_redo_system_demo
cargo run --example command_system_demo
cargo run --example editor_camera_demo
```

## Architecture

The editor is organized into several subsystems:

- **EditorState**: Central coordinator managing panels, modes, and state
- **Panels**: Dockable UI components (hierarchy, inspector, console, etc.)
- **Systems**: ECS systems for selection, gizmos, camera
- **Commands**: Undoable operations via command pattern
- **Menu/Toolbar**: Standard UI controls with shortcuts

All panels implement the `EditorPanel` trait for consistent integration with the docking system.

## Dependencies

- `egui` 0.29 - Immediate mode GUI
- `egui_dock` - Dockable panels
- `bevy_ecs` 0.14 - ECS integration
- `ron` 0.8 - Command serialization
- `praxis_graphics` - Rendering integration
- `praxis_scene` - Transform hierarchy
- `praxis_input` - Input handling

## Features

- **Default**: Core editor functionality
- `terrain` - Terrain editing panel (requires terrain features in other crates)

## Related Crates

- `praxis_gui` - Base GUI components used by editor panels
- `praxis_graphics` - Rendering backend for viewports
- `praxis_scene` - Scene graph and transform hierarchy
- `praxis_input` - Input handling for camera and selection

## License

See `LICENSE` file in repository root.
