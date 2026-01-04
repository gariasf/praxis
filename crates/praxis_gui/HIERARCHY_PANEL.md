# Hierarchy Panel

The Hierarchy Panel provides a tree view for visualizing and manipulating the ECS scene graph in the Praxis engine.

## Features

### Tree View with Collapsible Headers

- Hierarchical display of entities using `egui::collapsing_header`
- Entities are organized by parent-child relationships
- Collapsible/expandable nodes for entities with children
- Indentation-based visual hierarchy
- Sorted alphabetically by entity name

### Entity Drag-Drop Reparenting

- Click and drag entities to reparent them
- Visual feedback with yellow highlight on drop target
- Circular dependency prevention
- Automatic Children component synchronization

### Multi-Select Support

- Single-click: Select single entity (clears previous selection)
- Ctrl/Cmd + Click: Toggle entity in selection
- Shift + Click: Add entity to selection
- Clear Selection button to deselect all

### Right-Click Context Menu

#### On Entity
- **Create Child**: Creates a new child entity under the selected entity
- **Duplicate**: Creates a copy of the entity with same components
- **Delete**: Deletes the entity and all its children recursively
- **Remove Parent**: Detaches entity from parent (makes it root)

#### On Background
- **Create Entity**: Creates a new root entity with Transform
- **Create Camera**: Creates a camera entity with all required components
- **Create Light**: Creates a point light entity

### Selection System Integration

The `SelectionState` resource is shared between:
- **HierarchyPanel**: Tree view selection
- **EntityInspector**: Component editing for selected entity

Both components stay synchronized through the ECS world resource system.

## Usage

```rust
use praxis_gui::{GuiState, SelectionState};
use praxis_ecs::World;

// The HierarchyPanel is automatically added to GuiState
let mut gui_state = GuiState::new(event_loop, surface, queue, format);

// Render the GUI (including hierarchy panel)
gui_state.render(window, &mut world, image_view, render_pass)?;

// Access selection state
let selection = world.inner().resource::<SelectionState>();
if let Some(entity) = selection.primary_selection {
    println!("Primary selection: {:?}", entity);
}
println!("Total selected: {}", selection.selection_count());
```

## Architecture

### SelectionState Resource

```rust
pub struct SelectionState {
    pub selected_entities: HashSet<Entity>,
    pub primary_selection: Option<Entity>,
}
```

Methods:
- `select_single(entity)`: Select one entity, clear others
- `toggle_selection(entity)`: Toggle entity in multi-select
- `add_to_selection(entity)`: Add entity to selection
- `clear()`: Deselect all
- `is_selected(entity)`: Check if entity is selected
- `selection_count()`: Get number of selected entities

### HierarchyPanel Component

```rust
pub struct HierarchyPanel {
    pub visible: bool,
    search_filter: String,
    drag_source: Option<Entity>,
    context_menu: Option<ContextMenuTarget>,
    collapsed_entities: HashSet<Entity>,
}
```

Methods:
- `new()`: Create new hierarchy panel
- `render(ctx, world)`: Render the panel UI
- `toggle()`: Toggle visibility
- `set_visible(visible)`: Set visibility

### Integration Points

The HierarchyPanel integrates with:
- **GuiState**: Rendered alongside other GUI components
- **EntityInspector**: Shares SelectionState for synchronized selection
- **ECS World**: Directly manipulates entities and components
- **Parent/Children Components**: Maintains hierarchy relationships

## Search and Filter

- Text search box at top of panel
- Case-insensitive filtering by entity name
- Clear button (✖) to reset filter
- Filtered entities still maintain hierarchy relationships

## Visual Design

- 🔍 Search icon
- ➕ Create button
- ▶/▼ Collapse/expand arrows
- Selected entities highlighted in light blue
- Indentation of 20px per hierarchy level
- Entity display format: "Name (Entity ID)"

## Component Integration

Works seamlessly with existing ECS components:
- `Name`: Entity display names
- `Transform`: Position/rotation/scale for created entities
- `GlobalTransform`: World-space transforms
- `Parent`: Parent reference
- `Children`: Child entity list
- `Camera`: Camera components for camera creation
- `PointLight`: Light components for light creation

## Error Handling

- Checks for entity existence before operations
- Prevents circular parent-child references
- Handles missing components gracefully
- Cleans up selection when entities are deleted
