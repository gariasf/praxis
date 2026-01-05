# Hierarchy Panel Guide

## Overview

The Hierarchy Panel provides a comprehensive entity tree visualization with drag-and-drop reparenting, entity creation/deletion, and full undo system integration. It displays all entities in the scene organized by parent-child relationships.

## Features

- **Entity Tree Visualization**: Hierarchical display with proper indentation
- **Drag-and-Drop Reparenting**: Intuitively reorganize entity hierarchy
- **Entity Creation/Deletion**: Toolbar buttons with undo support
- **Multi-Selection Integration**: Works seamlessly with SelectionSystem
- **Live Updates**: Real-time refresh as entities spawn/despawn
- **Expansion State**: Collapsible tree nodes with persistent state
- **Undo Integration**: All operations create commands for undo/redo

## Architecture

### Core Structure

```rust
pub struct HierarchyPanel {
    title: String,
    entity_ops: EntityOperations,      // For create/delete with undo
    drag_entity: Option<Entity>,        // Currently dragged entity
    expanded: HashSet<Entity>,          // Expanded tree nodes
}
```

### Design Rationale

**EntityOperations Integration**: Rather than directly manipulating entities, all operations go through EntityOperations which:
- Automatically creates undo commands
- Provides consistent error handling
- Ensures proper component state capture

**Expansion State Management**: The panel maintains its own expanded state rather than storing it on entities because:
- It's UI state, not game state
- Avoids polluting ECS with editor-specific components
- Simpler to serialize for editor preferences

## Entity Tree Visualization

### Root Detection

The panel automatically identifies root entities:

```rust
// Root entities have no Parent component
fn find_root_entities(world: &World) -> Vec<Entity> {
    let mut query = world.query_filtered::<Entity, Without<Parent>>();
    query.iter(world).collect()
}
```

### Hierarchical Display

Entities are rendered recursively with proper indentation:

```
▼ Root Entity
  └─▼ Child Entity 1
     └─● Grandchild Entity
  └─● Child Entity 2
▶ Another Root Entity
```

- **▼**: Expanded node with children
- **▶**: Collapsed node with children
- **●**: Leaf node (no children)

### Indentation

Each depth level adds 20px indentation:

```rust
let indent = depth * 20.0;
ui.add_space(indent);
```

### Entity Naming

Entities display their name from the Name component, or fallback to entity ID:

```rust
let label = world
    .get::<Name>(entity)
    .map(|n| n.as_str())
    .unwrap_or_else(|| format!("Entity {:?}", entity.id()));
```

## Selection Integration

### Multi-Selection Support

The hierarchy panel fully integrates with SelectionSystem:

```rust
// Visual highlight for selected entities
if selection.is_selected(entity) {
    ui.painter().rect_filled(
        rect,
        0.0,
        egui::Color32::from_rgb(50, 100, 200),
    );
}
```

### Selection Modes

| Input | Mode | Behavior |
|-------|------|----------|
| Click | Replace | Clear selection, select clicked entity |
| Shift+Click | Add | Add clicked entity to selection |
| Ctrl+Click | Toggle | Toggle clicked entity selection |

### Implementation

```rust
fn handle_entity_interaction(
    &self,
    entity: Entity,
    response: &egui::Response,
    selection: &mut SelectionSystem,
) {
    if response.clicked() {
        let mode = if response.ctx.input(|i| i.modifiers.shift) {
            SelectionMode::Add
        } else if response.ctx.input(|i| i.modifiers.ctrl) {
            SelectionMode::Toggle
        } else {
            SelectionMode::Replace
        };
        
        selection.select_entity(entity, mode);
    }
}
```

## Drag-and-Drop Reparenting

### Drag Initiation

Click and hold an entity to start dragging:

```rust
let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

if response.drag_started() {
    self.drag_entity = Some(entity);
}
```

### Visual Feedback

During drag operation:

```rust
// Grabbing cursor
response.ctx.set_cursor_icon(egui::CursorIcon::Grabbing);

// Highlight drop targets
if self.drag_entity.is_some() && response.hovered() {
    ui.painter().rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(2.0, egui::Color32::GREEN),
    );
}
```

### Drop Handling

On mouse release:

```rust
if response.drag_released() {
    if let Some(dragged) = self.drag_entity {
        if response.hovered() {
            // Drop on entity -> set as parent
            self.reparent_entity(dragged, Some(entity), world, undo_system);
        }
        self.drag_entity = None;
    }
}

// Drop on empty space -> remove parent
if response.drag_released() && empty_space_hovered {
    if let Some(dragged) = self.drag_entity {
        self.reparent_entity(dragged, None, world, undo_system);
        self.drag_entity = None;
    }
}
```

### Circular Hierarchy Prevention

The system validates that a child cannot become the parent of its ancestor:

```rust
fn is_ancestor_of(
    entity: Entity,
    potential_ancestor: Entity,
    world: &World,
) -> bool {
    let mut current = Some(entity);
    
    while let Some(e) = current {
        if e == potential_ancestor {
            return true;
        }
        current = world.get::<Parent>(e).map(|p| p.get());
    }
    
    false
}

// In reparent_entity()
if is_ancestor_of(new_parent, entity, world) {
    // Show error: "Cannot create circular hierarchy"
    return;
}
```

## Entity Creation and Deletion

### Toolbar Buttons

Top toolbar provides creation and deletion:

```
[+ Create Entity] [🗑️ Delete]
```

### Create Entity

Creates new entity with default components:

```rust
if ui.button("+ Create Entity").clicked() {
    let transform = Transform::default();
    let name = Name::new("New Entity");
    
    self.entity_ops.create_entity(
        world,
        undo_system,
        vec![
            Box::new(transform),
            Box::new(name),
        ],
    );
}
```

### Delete Entity

Deletes all selected entities:

```rust
let has_selection = selection.selected_count() > 0;

ui.add_enabled_ui(has_selection, |ui| {
    if ui.button("🗑️ Delete").clicked() {
        let entities: Vec<_> = selection.selected_entities().collect();
        
        for entity in entities {
            self.entity_ops.delete_entity(entity, world, undo_system);
        }
        
        selection.clear();
    }
});
```

### Undo Integration

All operations automatically create undo commands:

```rust
// CreateEntityCommand captures:
// - Entity ID
// - All components and their values

// DeleteEntityCommand captures:
// - Entity ID
// - All components (for restoration)
// - Parent-child relationships

// SetParentCommand captures:
// - Entity ID
// - Old parent (Option<Entity>)
// - New parent (Option<Entity>)
```

## Expansion State

### Toggling Nodes

Click arrow to expand/collapse:

```rust
let arrow = if self.expanded.contains(&entity) { "▼" } else { "▶" };

if ui.button(arrow).clicked() {
    if self.expanded.contains(&entity) {
        self.expanded.remove(&entity);
    } else {
        self.expanded.insert(entity);
    }
}
```

### Persistent State

Expansion state persists across:
- Entity creation/deletion
- Scene loading
- Editor sessions (if serialized)

### Automatic Expansion

Expand tree to show specific entity:

```rust
pub fn expand_to_entity(&mut self, entity: Entity, world: &World) {
    let mut current = Some(entity);
    
    while let Some(e) = current {
        self.expanded.insert(e);
        current = world.get::<Parent>(e).map(|p| p.get());
    }
}

// Usage: Expand to show selected entity
if let Some(entity) = selection.first_selected() {
    hierarchy.expand_to_entity(entity, &world);
}
```

## Live Updates

### Automatic Refresh

The tree updates automatically as entities change:

```rust
// System monitors these components
Query<(Entity, Option<&Parent>, Option<&Children>, Option<&Name>)>

// On component add/remove:
// - Tree structure rebuilds
// - Expansion state preserved
// - Selection maintained
```

### Empty State

When scene has no entities:

```
╭─────────────────────╮
│  Scene is empty     │
│                     │
│  Click [+ Create]   │
│  to add entities    │
╰─────────────────────╯
```

## Integration with EditorState

### Automatic Setup

EditorState includes hierarchy panel by default:

```rust
pub struct EditorState {
    hierarchy_panel: HierarchyPanel,
    // ... other panels
}

// In EditorState::ui()
if menu_state.show_hierarchy {
    hierarchy_panel.ui_with_world(
        ui,
        &mut world,
        &mut undo_system,
        &mut selection,
    );
}
```

### Required Resources

The hierarchy panel requires three resources:

1. **World**: For querying entities and components
2. **UndoRedoSystem**: For creating undo commands
3. **SelectionSystem**: For selection integration

```rust
// Full integration
hierarchy_panel.ui_with_world(ui, world, undo_system, selection);

// Fallback (limited functionality)
hierarchy_panel.ui(ui);  // No world access, read-only
```

## Advanced Usage

### Custom Entity Labels

Customize how entities are displayed:

```rust
impl HierarchyPanel {
    pub fn set_label_formatter(
        &mut self,
        formatter: Box<dyn Fn(Entity, &World) -> String>,
    ) {
        self.label_formatter = Some(formatter);
    }
}

// Usage
hierarchy.set_label_formatter(Box::new(|entity, world| {
    let name = world.get::<Name>(entity).map(|n| n.as_str()).unwrap_or("Unnamed");
    let type_icon = if world.get::<Camera>(entity).is_some() {
        "📷"
    } else if world.get::<Light>(entity).is_some() {
        "💡"
    } else {
        "📦"
    };
    format!("{} {}", type_icon, name)
}));
```

### Entity Icons

Add visual indicators for entity types:

```rust
fn get_entity_icon(entity: Entity, world: &World) -> &'static str {
    if world.get::<Camera>(entity).is_some() {
        "📷"  // Camera
    } else if world.get::<DirectionalLight>(entity).is_some() {
        "☀️"  // Directional light
    } else if world.get::<PointLight>(entity).is_some() {
        "💡"  // Point light
    } else if world.get::<Mesh>(entity).is_some() {
        "📦"  // Mesh
    } else {
        "●"   // Generic
    }
}
```

### Context Menu

Add right-click context menu:

```rust
response.context_menu(|ui| {
    if ui.button("Duplicate").clicked() {
        duplicate_entity(entity, world);
        ui.close_menu();
    }
    
    if ui.button("Copy").clicked() {
        clipboard.copy_entity(entity, world);
        ui.close_menu();
    }
    
    if ui.button("Paste as Child").clicked() {
        clipboard.paste_as_child(entity, world);
        ui.close_menu();
    }
    
    ui.separator();
    
    if ui.button("Delete").clicked() {
        delete_entity(entity, world, undo_system);
        ui.close_menu();
    }
});
```

### Filtering

Filter entities by type or name:

```rust
pub struct HierarchyPanel {
    // ... existing fields
    filter: String,
    filter_type: Option<FilterType>,
}

pub enum FilterType {
    All,
    HasComponent(ComponentId),
    Name(String),
}

// Render filter UI
ui.text_edit_singleline(&mut self.filter);

// Filter entities
let visible_entities: Vec<_> = all_entities
    .into_iter()
    .filter(|e| matches_filter(*e, &self.filter, &self.filter_type, world))
    .collect();
```

### Drag-and-Drop Ordering

Allow reordering siblings:

```rust
// Track drop position (before/after sibling)
enum DropPosition {
    AsChild,
    BeforeSibling,
    AfterSibling,
}

// Visual indicator
if dragging_over_entity {
    let drop_pos = calculate_drop_position(mouse_y, entity_rect);
    
    match drop_pos {
        DropPosition::AsChild => {
            // Highlight entire entity
            draw_highlight(entity_rect);
        }
        DropPosition::BeforeSibling => {
            // Draw line above entity
            draw_line(entity_rect.min.y);
        }
        DropPosition::AfterSibling => {
            // Draw line below entity
            draw_line(entity_rect.max.y);
        }
    }
}
```

## Performance Optimization

### Virtual Scrolling

For scenes with thousands of entities:

```rust
// Only render visible entities
let scroll_offset = ui.scroll_offset();
let visible_height = ui.available_height();

let visible_start = (scroll_offset / ITEM_HEIGHT) as usize;
let visible_end = ((scroll_offset + visible_height) / ITEM_HEIGHT) as usize + 1;

// Add spacer for items before visible range
ui.add_space(visible_start as f32 * ITEM_HEIGHT);

// Render only visible items
for i in visible_start..visible_end.min(entities.len()) {
    render_entity(&entities[i]);
}

// Add spacer for items after visible range
ui.add_space((entities.len() - visible_end) as f32 * ITEM_HEIGHT);
```

### Caching

Cache entity hierarchies:

```rust
struct HierarchyCache {
    root_entities: Vec<Entity>,
    children_map: HashMap<Entity, Vec<Entity>>,
    dirty: bool,
}

impl HierarchyCache {
    pub fn update(&mut self, world: &World) {
        if !self.dirty {
            return;
        }
        
        // Rebuild cache
        self.root_entities = find_root_entities(world);
        self.children_map = build_children_map(world);
        self.dirty = false;
    }
    
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}
```

## Troubleshooting

### Entities Not Showing

**Problem**: Entities exist but don't appear in hierarchy

**Solutions**:
- Verify entities have required components (Transform at minimum)
- Check if filtering is hiding entities
- Ensure parent entities are expanded
- Call `refresh()` to rebuild tree

### Drag-and-Drop Not Working

**Problem**: Cannot drag entities to reparent

**Solutions**:
- Verify world is mutable (ui_with_world vs ui)
- Check if entity has Parent component capability
- Ensure undo system is provided
- Test with simple parent/child relationship

### Selection Not Syncing

**Problem**: Clicking entity doesn't select it

**Solutions**:
- Verify SelectionSystem is passed to panel
- Check entity has Selectable component
- Ensure update_selection_system is running
- Debug with logging in handle_entity_interaction

### Undo Not Working

**Problem**: Reparenting cannot be undone

**Solutions**:
- Verify UndoRedoSystem is provided
- Check SetParentCommand is being created
- Ensure commands are added to undo stack
- Test with simpler undo operation first

## Complete Example

```rust
use praxis_editor::*;
use praxis_ecs::*;
use praxis_scene::*;

fn main() {
    // Setup
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut selection = SelectionSystem::new();
    let mut hierarchy = HierarchyPanel::new();
    
    // Spawn some entities
    let root = world.spawn((
        Transform::default(),
        Name::new("Root Entity"),
        Selectable,
    )).id();
    
    let child1 = world.spawn((
        Transform::default(),
        Name::new("Child 1"),
        Parent::new(root),
        Selectable,
    )).id();
    
    let child2 = world.spawn((
        Transform::default(),
        Name::new("Child 2"),
        Parent::new(root),
        Selectable,
    )).id();
    
    // Render loop
    egui_context.run(|ctx| {
        egui::SidePanel::left("hierarchy").show(ctx, |ui| {
            hierarchy.ui_with_world(
                ui,
                &mut world,
                &mut undo_system,
                &mut selection,
            );
        });
    });
}
```

## See Also

- [Selection System](selection-system.md)
- [Undo/Redo System](undo-redo.md)
- [Inspector Panel](inspector.md)
- [Scene Management](../guides/scene_format_v2.md)
- [Transform Hierarchy](../concepts/transforms.md)
