# Hierarchy Panel

The Hierarchy Panel provides a tree-based view of the scene's entity structure, displaying parent-child relationships and enabling intuitive scene organization.

## Overview

The Hierarchy Panel is the primary interface for managing entity relationships in the Praxis editor. It visualizes the ECS entity hierarchy, supports drag-and-drop reparenting, multi-selection, and integrates seamlessly with the undo/redo system.

### Key Features

- **Tree Visualization**: Hierarchical display of entities with parent-child relationships
- **Drag-and-Drop Reparenting**: Intuitive entity organization via mouse
- **Multi-Selection**: Select multiple entities with Shift/Ctrl modifiers
- **Entity Operations**: Create, delete, and duplicate entities with undo support
- **Search/Filter**: Find entities by name
- **Expand/Collapse**: Navigate large hierarchies efficiently
- **Context Menus**: Right-click operations for quick access
- **Circular Dependency Prevention**: Automatic validation of hierarchy changes

## Architecture

### ECS Integration

The Hierarchy Panel directly reads and modifies ECS components:

| Component | Purpose |
|-----------|---------|
| `Parent` | Stores the parent entity reference |
| `Children` | Tracks list of child entities |
| `Name` | Display name in the hierarchy |
| `Transform` | Local transform relative to parent |
| `GlobalTransform` | Computed world-space transform |

**Note**: The `Parent`/`Children` relationship is bidirectional and automatically synchronized by the ECS transform propagation system.

### Panel Structure

```rust
pub struct HierarchyPanel {
    // Panel state
    title: String,
    visible: bool,
    
    // Entity operations with undo support
    entity_ops: EntityOperations,
    
    // Drag-and-drop state
    drag_entity: Option<Entity>,
    
    // UI state
    expanded: HashSet<Entity>,      // Expanded tree nodes
    search_filter: String,          // Search text
    
    // Selection integration
    selection_state: SelectionState,
}
```

## Usage

### Basic Setup

```rust
use praxis_editor::panels::HierarchyPanel;
use praxis_editor::{SelectionSystem, UndoRedoSystem};
use praxis_ecs::World;

let mut hierarchy_panel = HierarchyPanel::new();
let mut world = World::new();
let mut undo_system = UndoRedoSystem::new();
let mut selection_system = SelectionSystem::new();

// In your editor update loop
hierarchy_panel.ui_with_world(
    ui,
    &mut world,
    &mut undo_system,
    &mut selection_system,
);
```

### Integration with EditorState

The Hierarchy Panel is typically used as part of the `EditorState`:

```rust
use praxis_editor::EditorState;

let mut editor_state = EditorState::new();

// Hierarchy panel is automatically included
// Access via editor_state if needed
```

## Entity Operations

### Creating Entities

**Via Toolbar Button**:
- Click "➕ Create Entity" button
- New entity is automatically selected
- Undo/redo supported

**Programmatically**:
```rust
use praxis_editor::EntityOperations;

let mut entity_ops = EntityOperations::new();

// Create empty entity
let entity = entity_ops.create_entity(&mut world, &mut undo_system)?;

// Create with name and transform
let entity = entity_ops.create_entity_with_components(
    &mut world,
    &mut undo_system,
    "Player",
    Transform::from_xyz(0.0, 1.0, 0.0),
)?;
```

### Deleting Entities

**Via Toolbar Button**:
- Select one or more entities
- Click "🗑 Delete" button
- Entities and all children are removed
- Full undo support

**Programmatically**:
```rust
// Delete single entity
entity_ops.delete_entity(&mut world, &mut undo_system, entity)?;

// Delete multiple entities
let entities = vec![entity1, entity2, entity3];
entity_ops.delete_entities(&mut world, &mut undo_system, entities)?;
```

**Note**: Deleting a parent entity recursively deletes all children.

### Reparenting Entities

**Via Drag-and-Drop**:
1. Click and drag an entity
2. Hover over target parent entity (highlights)
3. Release to reparent
4. Drop on empty space to make root entity

**Programmatically**:
```rust
use crate::undo::SetParentCommand;

// Reparent entity
let command = SetParentCommand::new(child, old_parent, Some(new_parent));
undo_system.execute_command(&mut world, Box::new(command))?;

// Remove parent (make root)
let command = SetParentCommand::new(child, old_parent, None);
undo_system.execute_command(&mut world, Box::new(command))?;
```

**Validation**: The system automatically prevents circular hierarchies (e.g., making a parent a child of its descendant).

### Duplicating Entities

```rust
// Duplicate entity with all components
let new_entity = entity_ops.duplicate_entity(
    &mut world,
    &mut undo_system,
    entity,
)?;

// Duplicate with position offset
let new_entity = entity_ops.duplicate_entity_with_offset(
    &mut world,
    &mut undo_system,
    entity,
    Vec3::new(1.0, 0.0, 0.0),
)?;
```

**Duplicated Components**:
- `Transform` (with optional offset)
- `Name` (with " Copy" suffix)
- `Parent` (maintains hierarchy position)

## Selection System

### Selection Modes

| Mode | Input | Behavior |
|------|-------|----------|
| **Replace** | Click | Clear existing selection, select clicked entity |
| **Add** | Shift+Click | Add entity to selection |
| **Toggle** | Ctrl+Click | Toggle entity selection state |

### Querying Selection

```rust
use praxis_editor::SelectionSystem;

let selection = SelectionSystem::new();

// Check if entity is selected
if selection.is_selected(entity) {
    // Render selection highlight
}

// Iterate selected entities
for entity in selection.selected_entities() {
    // Process entity
}

// Get selection count
let count = selection.selection_count();
```

### Selection Events

The selection system emits events when selection changes:

```rust
use praxis_editor::SelectionEvent;

for event in selection_system.drain_events() {
    match event {
        SelectionEvent::Selected(entities) => {
            // Entities were added to selection
        }
        SelectionEvent::Deselected(entities) => {
            // Entities were removed from selection
        }
        SelectionEvent::Cleared => {
            // All entities deselected
        }
        SelectionEvent::Changed => {
            // Generic selection change
        }
    }
}
```

## Tree Navigation

### Expand/Collapse

**Interactive**:
- Click **▶** arrow to expand
- Click **▼** arrow to collapse
- Only entities with children show arrows

**Programmatic**:
```rust
// Expand all entities
hierarchy_panel.expand_all(&world);

// Collapse all entities
hierarchy_panel.collapse_all();

// Expand tree to show specific entity
hierarchy_panel.expand_to_entity(&world, entity);
```

### Search/Filter

Users can filter entities by name using the search bar:

```rust
// Filter is case-insensitive and matches partial names
// Search for "player" will match:
// - "Player"
// - "PlayerController"  
// - "EnemyPlayer"
```

## Rendering Details

### Visual States

Entities are rendered with different visual styles:

```rust
// Selected entity
let (bg_color, text_color) = (
    ui.visuals().selection.bg_fill,
    ui.visuals().selection.stroke.color,
);

// Hovered entity
let visuals = ui.style().interact(&response);
ui.painter().rect_filled(rect, 2.0, visuals.bg_fill);

// Drag source
ui.ctx().set_cursor_icon(CursorIcon::Grabbing);

// Drop target
ui.painter().rect_stroke(
    rect,
    2.0,
    egui::Stroke::new(2.0, ui.visuals().selection.stroke.color),
);
```

### Indentation

Tree depth is visualized through indentation:

```rust
let indent = depth as f32 * 20.0;  // 20 pixels per level
ui.add_space(indent);
```

### Entity Labels

Display format: `{name} (Entity ID)`

```rust
let name = world.get::<Name>(entity)
    .map(|n| n.0.clone())
    .unwrap_or_else(|| format!("Entity {:?}", entity));
```

## Undo/Redo Integration

All hierarchy operations create commands for the undo system:

### Available Commands

| Command | Operation | Undo Behavior |
|---------|-----------|---------------|
| `CreateEntityCommand` | Create new entity | Delete entity |
| `DeleteEntityCommand` | Delete entity | Recreate with captured state |
| `SetParentCommand` | Change parent | Restore previous parent |
| `CompositeCommand` | Batch operations | Undo all in reverse |

### Example Command Pattern

```rust
use praxis_editor::undo::{SetParentCommand, UndoRedoSystem};

// Create command
let command = SetParentCommand::new(
    child,
    old_parent,  // Captured before change
    new_parent,  // New parent (or None)
);

// Execute with undo support
undo_system.execute_command(&mut world, Box::new(command))?;

// Undo reverts to old_parent
undo_system.undo(&mut world)?;

// Redo applies new_parent again
undo_system.redo(&mut world)?;
```

## Advanced Patterns

### Custom Entity Creation

```rust
use praxis_editor::EntityOperations;

// Create entity with custom components
let entity = entity_ops.create_entity(&mut world, &mut undo_system)?;

// Add components manually
world.entity_mut(entity).insert((
    MeshHandle::new("cube"),
    MaterialHandle::new("pbr_default"),
    RigidBody::Dynamic,
));
```

### Batch Operations

Group multiple operations into a single undo command:

```rust
// Begin batch
entity_ops.begin_batch("Create Level Geometry");

for i in 0..10 {
    entity_ops.create_entity_with_components(
        &mut world,
        &mut undo_system,
        format!("Wall_{}", i),
        Transform::from_xyz(i as f32 * 2.0, 0.0, 0.0),
    )?;
}

// End batch (all operations undo as one)
entity_ops.end_batch(&mut world, &mut undo_system)?;
```

### Hierarchy Traversal

```rust
use praxis_ecs::{Children, Parent};

// Get all children of an entity
fn get_children(world: &World, entity: Entity) -> Vec<Entity> {
    world.get::<Children>(entity)
        .map(|c| c.0.clone())
        .unwrap_or_default()
}

// Get parent of an entity
fn get_parent(world: &World, entity: Entity) -> Option<Entity> {
    world.get::<Parent>(entity).map(|p| p.0)
}

// Walk hierarchy recursively
fn walk_hierarchy<F: FnMut(Entity, usize)>(
    world: &World,
    entity: Entity,
    depth: usize,
    visitor: &mut F,
) {
    visitor(entity, depth);
    
    for child in get_children(world, entity) {
        walk_hierarchy(world, child, depth + 1, visitor);
    }
}

// Example: print entire hierarchy
walk_hierarchy(&world, root_entity, 0, &mut |entity, depth| {
    let indent = "  ".repeat(depth);
    let name = world.get::<Name>(entity)
        .map(|n| n.0.as_str())
        .unwrap_or("Unnamed");
    println!("{}{} - {:?}", indent, name, entity);
});
```

### Finding Root Entities

```rust
// Find all root entities (entities without Parent component)
let root_entities: Vec<Entity> = world
    .iter_entities()
    .filter(|entity_ref| entity_ref.get::<Parent>().is_none())
    .map(|entity_ref| entity_ref.id())
    .collect();
```

## Best Practices

### Performance

1. **Limit Expanded Nodes**: Collapse unused hierarchy branches
2. **Use Search**: Filter large hierarchies instead of scrolling
3. **Batch Operations**: Group multiple operations when possible
4. **Avoid Deep Nesting**: Keep hierarchy depth reasonable (< 10 levels)

### Organization

1. **Use Descriptive Names**: Make entities easy to find
2. **Group by Function**: Organize related entities under common parents
3. **Empty Container Entities**: Use empty parents for logical grouping

```rust
// Example hierarchy organization
Scene Root
├── Environment
│   ├── Terrain
│   ├── Sky
│   └── Lighting
│       ├── Sun
│       └── Ambient
├── Gameplay
│   ├── Player
│   └── Enemies
│       ├── Enemy_01
│       └── Enemy_02
└── UI
    ├── HUD
    └── Menu
```

### Naming Conventions

```rust
// Good entity names
"Player"
"MainCamera"
"PointLight_01"
"Wall_Section_A"

// Avoid
"Entity 123"
"e1"
"Untitled"
```

## Common Issues

### Circular Hierarchy Prevention

**Problem**: Attempting to make a parent a child of its descendant.

**Solution**: The system automatically validates and prevents this:

```rust
fn is_ancestor_of(&self, world: &World, entity: Entity, potential_ancestor: Entity) -> bool {
    if entity == potential_ancestor {
        return true;
    }
    
    let mut current = entity;
    while let Some(entity_ref) = world.get_entity(current) {
        if let Some(parent) = entity_ref.get::<Parent>() {
            if parent.0 == potential_ancestor {
                return true;
            }
            current = parent.0;
        } else {
            break;
        }
    }
    
    false
}
```

### Missing Children Component

**Problem**: `Children` component not automatically added when setting `Parent`.

**Solution**: Use the transform propagation system which maintains `Children` automatically:

```rust
// Ensure transform propagation system is running
schedule.add_systems((
    sync_parent_to_children_system,
    propagate_transforms_system,
));
```

### Transform Synchronization

**Problem**: Entity transforms not updating after reparenting.

**Solution**: Transform propagation runs automatically each frame. For immediate updates:

```rust
use praxis_ecs::systems::propagate_transforms_system;

// Manually trigger transform propagation
propagate_transforms_system.run(world);
```

## Examples

### Example 1: Creating a Scene Hierarchy

```rust
use praxis_editor::EntityOperations;
use praxis_ecs::{Transform, World};

let mut world = World::new();
let mut undo_system = UndoRedoSystem::new();
let mut entity_ops = EntityOperations::new();

// Create scene root
let scene_root = entity_ops.create_entity_with_components(
    &mut world,
    &mut undo_system,
    "Scene",
    Transform::default(),
)?;

// Create environment parent
let environment = entity_ops.create_entity_with_components(
    &mut world,
    &mut undo_system,
    "Environment",
    Transform::default(),
)?;
entity_ops.add_parent(&mut world, &mut undo_system, environment, scene_root)?;

// Create terrain as child of environment
let terrain = entity_ops.create_entity_with_components(
    &mut world,
    &mut undo_system,
    "Terrain",
    Transform::from_xyz(0.0, -1.0, 0.0),
)?;
entity_ops.add_parent(&mut world, &mut undo_system, terrain, environment)?;
```

### Example 2: Moving Multiple Entities

```rust
use praxis_editor::SelectionSystem;

// Select multiple entities
selection_system.select_entities(
    vec![entity1, entity2, entity3],
    SelectionMode::Replace,
);

// Create new parent
let container = entity_ops.create_entity_with_components(
    &mut world,
    &mut undo_system,
    "Container",
    Transform::default(),
)?;

// Move all selected entities under container
entity_ops.begin_batch("Group Entities");
for entity in selection_system.selected_entities() {
    entity_ops.add_parent(&mut world, &mut undo_system, entity, container)?;
}
entity_ops.end_batch(&mut world, &mut undo_system)?;
```

### Example 3: Custom Context Menu

```rust
// Extend HierarchyPanel with custom context menu
if ui.button("Create Prefab").clicked() {
    let prefab = create_prefab_entity(&mut world, &mut undo_system)?;
    entity_ops.add_parent(&mut world, &mut undo_system, prefab, selected_entity)?;
}

fn create_prefab_entity(
    world: &mut World,
    undo_system: &mut UndoRedoSystem,
) -> Result<Entity> {
    let mut entity_ops = EntityOperations::new();
    
    // Begin batch for complex entity creation
    entity_ops.begin_batch("Create Prefab");
    
    let root = entity_ops.create_entity_with_components(
        world,
        undo_system,
        "Prefab",
        Transform::default(),
    )?;
    
    // Add components
    world.entity_mut(root).insert((
        MeshHandle::new("cube"),
        MaterialHandle::new("default"),
    ));
    
    entity_ops.end_batch(world, undo_system)?;
    
    Ok(root)
}
```

## See Also

- [Inspector Panel](inspector.md) - Component editing
- [Selection System](selection.md) - Entity selection
- [Undo/Redo System](undo-redo.md) - Command history
- [Entity Operations](../../crates/praxis_editor/src/entity_operations.rs) - Implementation details
