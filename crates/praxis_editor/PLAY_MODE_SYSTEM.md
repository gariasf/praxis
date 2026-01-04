# Play Mode System

The Play Mode System provides a comprehensive solution for testing game functionality within the Praxis editor while maintaining complete isolation between edit and runtime states.

## Overview

The `PlayModeSystem` manages the complete lifecycle of play mode, including:

- **Edit/Play State Machine**: Clean state transitions between Edit and Play modes
- **Scene Snapshot/Restore**: Automatic capture and restoration of scene state
- **Runtime ECS Isolation**: Changes made in play mode don't affect the original scene
- **Input Routing Toggle**: Configurable input handling for play mode
- **Visual Indicators**: Clear visual feedback for current mode (viewport borders, button states)

## Architecture

### State Machine

The play mode system implements a simple three-state machine:

```
┌──────────┐
│   Edit   │ ◄─────────────────────────┐
└────┬─────┘                            │
     │ enter_play_mode()                │
     ▼                                  │
┌──────────┐                            │
│ Playing  │ ───────────────────────────┤
└────┬─────┘  exit_play_mode()          │
     │                                  │
     │ pause_play_mode()                │
     ▼                                  │
┌──────────┐                            │
│  Paused  │ ────────────────────────────┘
└──────────┘  exit_play_mode()
```

### Key Components

#### PlayModeState
```rust
pub enum PlayModeState {
    Edit,     // Editor is in edit mode
    Playing,  // Game simulation is running
    Paused,   // Game simulation is paused (treated as edit for input)
}
```

#### SceneSnapshot
```rust
pub struct SceneSnapshot {
    scene_definition: SceneDefinition,  // Serialized scene state
    metadata: SnapshotMetadata,         // Timestamp, entity count, etc.
}
```

The snapshot captures:
- All entities (except those marked with `NoSave` component)
- Transform hierarchies (Parent/Children relationships)
- Component data (Name, Transform, Mesh, Material, Camera, Lights, etc.)
- Visibility and Active state

## Usage

### Basic Integration

```rust
use praxis_editor::{EditorState, PlayModeSystem};
use praxis_ecs::World;

let mut world = World::new();
let mut editor = EditorState::new();

// Enter play mode
editor.enter_play_mode(&mut world)?;

// While in play mode:
// - Scene modifications are isolated
// - Input can be routed to play mode systems
// - Visual indicators show play mode is active

// Exit play mode (restores original scene)
editor.exit_play_mode(&mut world)?;
```

### Direct PlayModeSystem Usage

```rust
use praxis_editor::PlayModeSystem;
use praxis_ecs::World;

let mut world = World::new();
let mut play_mode = PlayModeSystem::new();

// Enter play mode
play_mode.enter_play_mode(&mut world)?;

// Check state
assert!(play_mode.is_playing());
assert_eq!(play_mode.state(), PlayModeState::Playing);

// Exit play mode
play_mode.exit_play_mode(&mut world)?;
```

### Pause and Resume

```rust
// Enter play mode
editor.enter_play_mode(&mut world)?;

// Pause (stops game simulation, keeps snapshot)
editor.pause_play_mode();

// Resume (continues game simulation)
editor.resume_play_mode();

// Stop and restore (clears snapshot)
editor.exit_play_mode(&mut world)?;
```

## Visual Feedback

### Viewport Border Colors

The system provides clear visual feedback through viewport border colors:

- **Edit Mode**: Dark gray `[0.3, 0.3, 0.35]`
- **Playing Mode**: Green `[0.2, 0.8, 0.3]`
- **Paused Mode**: Orange/Yellow `[0.9, 0.7, 0.2]`

Access the current border color:
```rust
let color = play_mode.viewport_border_color(); // Returns [f32; 3]
let egui_color = play_mode.viewport_border_color_egui(); // Returns egui::Color32
```

### Toolbar Button States

The toolbar provides visual feedback through button states:

- **Play Button**: Green fill when in edit mode and ready to play
- **Pause Button**: Orange fill when in play mode
- **Stop Button**: Red fill when in play mode
- Buttons are disabled when not applicable (e.g., Play disabled during play mode)

## Scene Snapshotting

### What Gets Saved

The snapshot system captures all entities **without** the `NoSave` component marker:

#### Captured Components
- `Name`: Entity name
- `Transform`: Local position, rotation, scale
- `GlobalTransform`: World-space transform (rebuilt on restore)
- `Parent` / `Children`: Hierarchy relationships
- `MeshHandle`: Reference to mesh asset
- `MaterialHandle`: Reference to material asset
- `Camera` + `PerspectiveProjection` / `OrthographicProjection`: Camera configuration
- `DirectionalLight`: Sun-like lights
- `PointLight`: Positional lights
- `Visibility`: Visibility state
- `Active`: Active state

#### Excluded Entities

Entities with the `NoSave` component are excluded from snapshots:

```rust
use praxis_ecs::{World, Transform, GlobalTransform, NoSave, Name};

let mut world = World::new();

// This entity will be saved in snapshots
world.spawn((
    Name::new("SavedEntity"),
    Transform::default(),
    GlobalTransform::default(),
));

// This entity will NOT be saved (editor-only)
world.spawn((
    Name::new("EditorOnlyEntity"),
    Transform::default(),
    GlobalTransform::default(),
    NoSave,  // Marks entity as editor-only
));
```

Use `NoSave` for:
- Editor camera entities
- Debug visualization entities
- Gizmos and editor UI elements
- Temporary preview entities

### Snapshot Process

#### Enter Play Mode (Taking Snapshot)

1. **Validation**: Checks if already in play mode
2. **Serialization**: Converts ECS World to SceneDefinition
   - Query all entities without `NoSave`
   - Serialize components for each entity
   - Preserve hierarchy relationships
3. **Storage**: Store snapshot in `Option<SceneSnapshot>`
4. **State Transition**: Change state to `Playing`

#### Exit Play Mode (Restoring Snapshot)

1. **Validation**: Checks if in play mode
2. **Clear Runtime**: Despawn all entities without `NoSave`
3. **Restore**: Spawn entities from snapshot
   - Recreate all entities with their components
   - Rebuild parent-child hierarchy
   - Restore transform chains
4. **Cleanup**: Clear snapshot reference
5. **State Transition**: Change state to `Edit`

## Input Routing

The system provides configurable input routing for play mode:

```rust
// Enable input routing to play mode (default)
play_mode.set_route_input_to_play(true);

// Disable input routing (input goes to editor only)
play_mode.set_route_input_to_play(false);

// Check if input should be routed
if play_mode.should_route_input_to_play() {
    // Handle input for game systems
} else {
    // Handle input for editor only
}
```

This allows you to:
- Test player input in play mode
- Keep editor controls active while playing
- Toggle between game testing and editor interaction

## Integration with Editor State

The `EditorState` provides high-level methods that integrate with the play mode system:

### Automatic Mode Synchronization

```rust
// EditorState automatically syncs mode across:
// - play_mode_system
// - menu_bar_state
// - toolbar_state

editor.enter_play_mode(&mut world)?;
assert_eq!(editor.mode(), EditorMode::Play);
assert_eq!(editor.toolbar_state().editor_mode, EditorMode::Play);
```

### Toolbar Actions

The toolbar's Play/Pause/Stop buttons automatically trigger the appropriate play mode transitions:

```rust
// Clicking "Play" button triggers:
editor.enter_play_mode(&mut world)?;

// Clicking "Pause" button triggers:
editor.pause_play_mode();

// Clicking "Stop" button triggers:
editor.exit_play_mode(&mut world)?;
```

### Viewport Visual Feedback

The scene viewport automatically updates its border color based on play mode:

```rust
// EditorState automatically updates scene panel border color
editor.scene_panel_mut()
    .set_border_color(play_mode.viewport_border_color_egui());
```

## Error Handling

All play mode transitions return `Result<()>` and handle errors gracefully:

```rust
use praxis_utils::Result;

// Enter play mode
match editor.enter_play_mode(&mut world) {
    Ok(()) => {
        // Successfully entered play mode
    }
    Err(e) => {
        // Handle error (e.g., log, show user message)
        eprintln!("Failed to enter play mode: {}", e);
    }
}

// Exit play mode
match editor.exit_play_mode(&mut world) {
    Ok(()) => {
        // Successfully exited play mode
    }
    Err(e) => {
        // Handle error
        eprintln!("Failed to exit play mode: {}", e);
    }
}
```

Common error scenarios:
- Already in play mode when trying to enter
- Not in play mode when trying to exit
- Scene serialization/deserialization failures

## Best Practices

### 1. Mark Editor-Only Entities

Always use the `NoSave` component for editor-only entities:

```rust
// Editor camera (not part of game scene)
world.spawn((
    Name::new("EditorCamera"),
    Camera::default(),
    Transform::default(),
    NoSave,  // Don't save in snapshots
));

// Debug visualization
world.spawn((
    Name::new("DebugGrid"),
    Transform::default(),
    NoSave,  // Editor-only
));
```

### 2. Test Scene State Isolation

Verify that play mode changes don't leak into edit mode:

```rust
// Enter play mode
editor.enter_play_mode(&mut world)?;

// Make runtime changes
world.spawn((Name::new("RuntimeEntity"), Transform::default()));

// Exit play mode - RuntimeEntity should be gone
editor.exit_play_mode(&mut world)?;
```

### 3. Handle Play Mode in Game Systems

Design game systems to work both in play mode and standalone:

```rust
use praxis_ecs::{Query, Transform, Res};
use praxis_editor::PlayModeSystem;

fn movement_system(
    mut query: Query<&mut Transform>,
    play_mode: Option<Res<PlayModeSystem>>,
) {
    // Only run in play mode or when editor not present
    if let Some(pm) = play_mode {
        if !pm.is_playing() {
            return; // Don't run in edit mode
        }
    }
    
    // Movement logic here
    for mut transform in query.iter_mut() {
        // ...
    }
}
```

### 4. Save Before Play Testing

Always save your scene before entering play mode for important work:

```rust
// Save scene
scene_manager.save_scene(&world, "my_scene.ron")?;

// Now safe to enter play mode
editor.enter_play_mode(&mut world)?;
```

## Implementation Details

### Hierarchy Preservation

The snapshot system preserves parent-child relationships through a three-pass algorithm:

1. **First Pass**: Collect all saveable entities and serialize their components
2. **Second Pass**: Identify root entities (no parent or parent not saveable)
3. **Third Pass**: Build hierarchy by adding children to their parents

This ensures the complete hierarchy is correctly restored.

### Transform Propagation

After restoring from snapshot, the editor should run transform propagation systems:

```rust
use praxis_ecs::systems::*;

// After exit_play_mode, run these systems:
schedule.add_systems((
    sync_parent_child_relationships,
    cleanup_removed_parents,
    propagate_transforms,
    propagate_transforms_for_reparented,
    propagate_transforms_for_changed_children,
).chain());
```

This ensures `GlobalTransform` components are correctly computed from local transforms and hierarchy.

## Testing

The play mode system includes comprehensive tests:

```rust
#[test]
fn test_enter_play_mode() {
    let mut world = World::new();
    let mut system = PlayModeSystem::new();
    
    // Add test entity
    world.spawn((Name::new("Test"), Transform::default()));
    
    // Enter play mode
    system.enter_play_mode(&mut world).unwrap();
    
    assert!(system.is_playing());
    assert!(system.snapshot.is_some());
}

#[test]
fn test_scene_restoration() {
    let mut world = World::new();
    let mut system = PlayModeSystem::new();
    
    // Create original entity
    let original = world.spawn((
        Name::new("Original"),
        Transform::from_xyz(1.0, 2.0, 3.0),
    ));
    
    // Enter play mode
    system.enter_play_mode(&mut world).unwrap();
    
    // Modify in play mode
    world.spawn((Name::new("Runtime"), Transform::default()));
    
    // Exit play mode
    system.exit_play_mode(&mut world).unwrap();
    
    // Verify restoration
    assert_eq!(count_entities(&world), 1);
}
```

## Future Enhancements

Potential improvements to the play mode system:

1. **Incremental Snapshots**: Only save changed entities for faster snapshots
2. **Multiple Snapshots**: Stack of snapshots for nested play mode sessions
3. **Snapshot Comparison**: Diff snapshots to show what changed during play
4. **Selective Restoration**: Restore only certain entities or components
5. **Snapshot Serialization**: Save/load snapshots to disk for crash recovery
6. **Hot Reload**: Update play mode scene without full restart
7. **Time Control**: Slow motion, step frame-by-frame in play mode
8. **Profiling Integration**: Automatic performance profiling during play mode

## See Also

- `EditorState`: Main editor coordinator that uses PlayModeSystem
- `EditorMode`: Edit/Play mode enumeration
- `SceneDefinition`: Scene serialization format
- `NoSave` component: Marker for editor-only entities
- `ToolbarState`: Toolbar integration with play mode buttons
