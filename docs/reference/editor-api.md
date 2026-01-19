# Editor API Reference

API reference for the Praxis editor system.

## Core Types

### EditorState

Main editor state coordinator.

```rust
pub struct EditorState { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `ui(ctx: &egui::Context, undo_system: Option<&mut UndoRedoSystem>, world: Option<&mut World>)`
- `set_play_mode(playing: bool)`
- `is_playing() -> bool`
- `is_dirty() -> bool`
- `mark_dirty()`
- `mark_clean()`

## Selection System

### SelectionSystem

Manages entity selection.

```rust
#[derive(Resource)]
pub struct SelectionSystem { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `select(entity: Entity)`
- `deselect(entity: Entity)`
- `toggle(entity: Entity)`
- `clear()`
- `is_selected(entity: Entity) -> bool`
- `selected_entities() -> &[Entity]`
- `primary_selection() -> Option<Entity>`

### Selectable

Component marking entity as selectable.

```rust
#[derive(Component)]
pub struct Selectable;
```

### Selected

Automatically added to selected entities.

```rust
#[derive(Component)]
pub struct Selected;
```

## Undo/Redo System

### UndoRedoSystem

Command history manager.

```rust
#[derive(Resource)]
pub struct UndoRedoSystem { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `with_capacity(capacity: usize) -> Self`
- `execute(world: &mut World, command: Box<dyn Command>) -> Result<()>`
- `undo(world: &mut World) -> Result<()>`
- `redo(world: &mut World) -> Result<()>`
- `can_undo() -> bool`
- `can_redo() -> bool`
- `clear()`
- `history_size() -> usize`

### Command Trait

Interface for undoable commands.

```rust
pub trait Command: Send + Sync {
    fn execute(&mut self, world: &mut World) -> Result<()>;
    fn undo(&mut self, world: &mut World) -> Result<()>;
    fn name(&self) -> &str;
}
```

### Built-in Commands

#### TransformEditCommand

Records transform changes.

```rust
pub struct TransformEditCommand {
    entity: Entity,
    old_transform: Transform,
    new_transform: Transform,
}
```

**Methods:**
- `new(entity, old, new) -> Self`

#### CreateEntityCommand

Records entity creation.

```rust
pub struct CreateEntityCommand {
    entity: Option<Entity>,
    components: EntityComponents,
}
```

#### DeleteEntityCommand

Records entity deletion.

```rust
pub struct DeleteEntityCommand {
    entity: Entity,
    components: EntityComponents,
}
```

#### CompositeCommand

Groups multiple commands.

```rust
pub struct CompositeCommand {
    name: String,
    commands: Vec<Box<dyn Command>>,
}
```

**Methods:**
- `new(name: &str) -> Self`
- `add_command(command: Box<dyn Command>)`

## Editor Camera

### EditorCameraController

Orbit camera controller.

```rust
#[derive(Resource)]
pub struct EditorCameraController {
    pub orbit_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub smoothness: f32,
}
```

**Methods:**
- `new() -> Self`
- `update(input: &InputState, delta: f32) -> Transform`
- `focus_on(target: Vec3, distance: f32)`
- `reset()`

### EditorCamera

Marker component for editor camera.

```rust
#[derive(Component)]
pub struct EditorCamera;
```

## Gizmo System

### GizmoSystem

Transform manipulation gizmos.

```rust
#[derive(Resource)]
pub struct GizmoSystem { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `set_mode(mode: GizmoMode)`
- `set_space(space: GizmoSpace)`
- `attach(entity: Entity)`
- `detach(entity: Entity)`
- `is_attached(entity: Entity) -> bool`
- `update(camera: &Camera, input: &InputState) -> Option<TransformDelta>`

### GizmoMode

Gizmo operation mode.

```rust
pub enum GizmoMode {
    Translate,  // Move along axes
    Rotate,     // Rotate around axes
    Scale,      // Resize along axes
}
```

### GizmoSpace

Coordinate space for gizmo.

```rust
pub enum GizmoSpace {
    World,   // World-aligned axes
    Local,   // Entity-aligned axes
}
```

### TransformDelta

Transformation delta from gizmo interaction.

```rust
pub struct TransformDelta {
    pub translation: Option<Vec3>,
    pub rotation: Option<Quat>,
    pub scale: Option<Vec3>,
}
```

## Panels

### Hierarchy Panel

Shows entity tree.

```rust
pub struct HierarchyPanel { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `ui(ctx: &egui::Context, world: &World, selection: &mut SelectionSystem)`

### Inspector Panel

Shows component details.

```rust
pub struct InspectorPanel { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `ui(ctx: &egui::Context, world: &mut World, selection: &SelectionSystem)`

### Console Panel

Log viewer and command input.

```rust
pub struct ConsolePanel { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `ui(ctx: &egui::Context)`
- `add_log(level: LogLevel, message: &str)`
- `clear()`

### Asset Browser

Asset management panel.

```rust
pub struct AssetBrowser { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `ui(ctx: &egui::Context, asset_manager: &AssetManager)`
- `refresh()`

## Menu Bar

### Menu Actions

```rust
pub enum MenuAction {
    NewScene,
    OpenScene,
    SaveScene,
    SaveSceneAs,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,
    DeselectAll,
    Play,
    Pause,
    Stop,
}
```

**Functions:**
- `render_menu_bar(ctx, menu_state, undo_system) -> Vec<MenuAction>`
- `handle_menu_action(action, world, editor_state) -> Result<()>`
- `check_keyboard_shortcuts(input) -> Option<MenuAction>`

## Play Mode

### PlayModeSystem

Manages play/edit mode transitions.

```rust
#[derive(Resource)]
pub struct PlayModeSystem { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `enter_play_mode(world: &World) -> WorldSnapshot`
- `exit_play_mode(world: &mut World, snapshot: WorldSnapshot)`

### WorldSnapshot

Serialized world state for play mode.

```rust
pub struct WorldSnapshot {
    pub data: Vec<u8>,
    pub timestamp: Instant,
}
```

## Common Patterns

### Basic Editor Setup

```rust
use praxis_editor::{EditorState, SelectionSystem, UndoRedoSystem, EditorCameraController};

// Initialize resources
world.insert_resource(SelectionSystem::new());
world.insert_resource(UndoRedoSystem::new());
world.insert_resource(EditorCameraController::new());

let mut editor = EditorState::new();

// Spawn editor camera
world.spawn((
    PerspectiveCameraBundle::new(Vec3::new(0.0, 5.0, 10.0), fov, aspect),
    EditorCamera,
));

// In UI loop
editor.ui(&egui_ctx, Some(&mut undo_system), Some(&mut world));
```

### Selection Workflow

```rust
use praxis_editor::{SelectionSystem, Selectable, Selected};

// Mark entities as selectable
world.spawn((
    Transform::default(),
    Mesh::new("cube"),
    Selectable,
));

// In selection system
fn selection_system(
    mut selection: ResMut<SelectionSystem>,
    input: Res<InputState>,
    query: Query<(Entity, &Transform), With<Selectable>>,
) {
    if input.is_mouse_button_just_pressed(MouseButton::Left) {
        if let Some(entity) = raycast_selection(&query) {
            if input.is_key_pressed(KeyCode::ShiftLeft) {
                selection.toggle(entity);
            } else {
                selection.clear();
                selection.select(entity);
            }
        }
    }
}

// Highlight selected
fn highlight_selected(
    query: Query<&Transform, With<Selected>>,
) {
    for transform in &query {
        // Draw highlight
    }
}
```

### Undo/Redo Usage

```rust
use praxis_editor::{UndoRedoSystem, TransformEditCommand};

fn transform_edit_system(
    mut undo_system: ResMut<UndoRedoSystem>,
    selection: Res<SelectionSystem>,
    mut query: Query<&mut Transform>,
) {
    if let Some(entity) = selection.primary_selection() {
        if let Ok(mut transform) = query.get_mut(entity) {
            let old = *transform;
            
            // Modify transform
            transform.translation.y += 1.0;
            
            let new = *transform;
            
            // Record change
            let command = TransformEditCommand::new(entity, old, new);
            undo_system.execute(&mut world, Box::new(command)).unwrap();
        }
    }
}

// Keyboard shortcuts
fn undo_redo_input(
    input: Res<InputState>,
    mut undo_system: ResMut<UndoRedoSystem>,
    mut world: ResMut<World>,
) {
    if input.is_key_pressed(KeyCode::ControlLeft) {
        if input.is_key_just_pressed(KeyCode::KeyZ) {
            if input.is_key_pressed(KeyCode::ShiftLeft) {
                let _ = undo_system.redo(&mut world);
            } else {
                let _ = undo_system.undo(&mut world);
            }
        }
    }
}
```

### Gizmo Integration

```rust
use praxis_editor::{GizmoSystem, GizmoMode, GizmoSpace};

let mut gizmo_system = GizmoSystem::new();
gizmo_system.set_mode(GizmoMode::Translate);
gizmo_system.set_space(GizmoSpace::World);
world.insert_resource(gizmo_system);

fn gizmo_system(
    mut gizmo: ResMut<GizmoSystem>,
    selection: Res<SelectionSystem>,
    camera: Query<&Camera, With<EditorCamera>>,
    input: Res<InputState>,
    mut transforms: Query<&mut Transform>,
    mut undo_system: ResMut<UndoRedoSystem>,
) {
    // Attach gizmo to selected entity
    if let Some(entity) = selection.primary_selection() {
        if !gizmo.is_attached(entity) {
            gizmo.attach(entity);
        }
        
        // Update gizmo
        if let Ok(camera) = camera.get_single() {
            if let Some(delta) = gizmo.update(camera, &input) {
                // Apply transformation
                if let Ok(mut transform) = transforms.get_mut(entity) {
                    let old = *transform;
                    
                    if let Some(t) = delta.translation {
                        transform.translation += t;
                    }
                    if let Some(r) = delta.rotation {
                        transform.rotation = r * transform.rotation;
                    }
                    if let Some(s) = delta.scale {
                        transform.scale *= s;
                    }
                    
                    let new = *transform;
                    
                    // Record for undo
                    let cmd = TransformEditCommand::new(entity, old, new);
                    let _ = undo_system.execute(&mut world, Box::new(cmd));
                }
            }
        }
    }
    
    // Cycle gizmo mode with keyboard
    if input.is_key_just_pressed(KeyCode::KeyT) {
        gizmo.set_mode(GizmoMode::Translate);
    }
    if input.is_key_just_pressed(KeyCode::KeyR) {
        gizmo.set_mode(GizmoMode::Rotate);
    }
    if input.is_key_just_pressed(KeyCode::KeyS) {
        gizmo.set_mode(GizmoMode::Scale);
    }
}
```

### Custom Command

```rust
use praxis_editor::Command;

struct SetColorCommand {
    entity: Entity,
    old_color: [f32; 4],
    new_color: [f32; 4],
}

impl Command for SetColorCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        if let Some(mut material) = world.get_mut::<Material>(self.entity) {
            material.base_color = self.new_color;
        }
        Ok(())
    }
    
    fn undo(&mut self, world: &mut World) -> Result<()> {
        if let Some(mut material) = world.get_mut::<Material>(self.entity) {
            material.base_color = self.old_color;
        }
        Ok(())
    }
    
    fn name(&self) -> &str {
        "Set Color"
    }
}
```

## See Also

- [Editor Guide](../editor/editor-overview.md) - Comprehensive editor guide
- [Selection System Guide](../editor/selection-system.md) - Selection details
- [Undo/Redo Guide](../editor/undo-redo.md) - Command system details
- [praxis_editor crate](../../crates/praxis_editor/README.md) - Crate documentation
