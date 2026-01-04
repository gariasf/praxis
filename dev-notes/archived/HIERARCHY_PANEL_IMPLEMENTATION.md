# Hierarchy Panel Implementation

## Overview

The HierarchyPanel has been fully implemented with complete ECS World integration, providing a comprehensive entity tree visualization with drag-and-drop reparenting, entity creation/deletion, and undo system integration.

## Features Implemented

### 1. Entity Tree Visualization
- **Hierarchical Display**: Shows all entities in the scene organized by parent-child relationships
- **Root Entity Detection**: Automatically identifies entities without parents as root nodes
- **Proper Indentation**: Visual hierarchy with 20px indentation per depth level
- **Entity Naming**: Displays entity names (from Name component) or fallback to entity ID
- **Expansion State**: Collapsible tree nodes with arrow indicators (▶/▼)
- **Empty State Message**: Shows "Scene is empty" when no entities exist

### 2. Drag-and-Drop Reparenting
- **Drag Source**: Click and drag any entity in the tree
- **Visual Feedback**: 
  - Grabbing cursor icon during drag
  - Highlight border on drop targets
  - Visual indication of current drag operation
- **Drop Targets**: 
  - Drop on another entity to set as parent
  - Drop on empty space to remove parent (make root entity)
- **Circular Hierarchy Prevention**: Validates that child cannot become parent of its ancestor
- **Undo Integration**: All reparenting operations create SetParentCommand for undo/redo

### 3. Entity Creation/Deletion
- **Create Button**: Creates new entity with default transform and name "New Entity"
- **Delete Button**: 
  - Only enabled when entities are selected
  - Deletes all selected entities
  - Clears selection after deletion
- **Undo Integration**: 
  - CreateEntityCommand for new entities
  - DeleteEntityCommand for removed entities
  - Captures full entity state for reliable undo

### 4. Selection Integration
- **Multi-Selection Support**: 
  - Click to select single entity (Replace mode)
  - Shift+Click to add to selection (Add mode)
  - Ctrl+Click to toggle selection (Toggle mode)
- **Visual Highlighting**: Selected entities shown with blue background
- **Automatic Synchronization**: Works seamlessly with SelectionSystem resource

### 5. Live Updates
- **Real-time Refresh**: Tree updates automatically as entities spawn/despawn
- **Component Tracking**: Monitors Parent, Children, and Name components
- **Persistent Expansion**: Maintains expanded state as tree changes

## Implementation Details

### Core Structure

```rust
pub struct HierarchyPanel {
    title: String,
    entity_ops: EntityOperations,      // For create/delete with undo
    drag_entity: Option<Entity>,        // Currently dragged entity
    expanded: HashSet<Entity>,          // Expanded tree nodes
}
```

### Key Methods

1. **`ui_with_world()`**: Main rendering method requiring World, UndoRedoSystem, and SelectionSystem
2. **`render_entity_tree()`**: Finds root entities and renders tree structure
3. **`render_entity_node()`**: Recursively renders entity and its children
4. **`render_entity_label()`**: Draws individual entity with drag-and-drop support
5. **`handle_entity_interaction()`**: Processes selection and drop events
6. **`reparent_entity()`**: Executes reparenting with validation and undo
7. **`is_ancestor_of()`**: Checks for circular hierarchy
8. **`expand_to_entity()`**: Expands tree to show specific entity

### Integration with EditorState

The panel integrates with EditorState through the EditorTabViewer:

```rust
EditorTab::Hierarchy => {
    if let (Some(world), Some(undo_system), Some(selection_system)) = (
        self.world.as_deref_mut(),
        self.undo_system.as_deref_mut(),
        self.selection_system.as_deref_mut(),
    ) {
        self.hierarchy_panel.ui_with_world(ui, world, undo_system, selection_system);
    } else {
        self.hierarchy_panel.ui(ui);  // Fallback
    }
}
```

### EditorState API Changes

The `EditorState::ui()` method signature was updated to accept SelectionSystem:

```rust
pub fn ui(
    &mut self,
    ctx: &egui::Context,
    undo_system: Option<&mut UndoRedoSystem>,
    world: Option<&mut World>,
    selection_system: Option<&mut SelectionSystem>,  // NEW
)
```

## Usage Example

```rust
use praxis_editor::{EditorState, SelectionSystem, UndoRedoSystem};
use praxis_ecs::World;
use egui::Context;

// Setup
let mut world = World::new();
let mut editor_state = EditorState::new();
let mut undo_system = UndoRedoSystem::new();
let mut selection_system = SelectionSystem::new();

// In your render loop
editor_state.ui(
    &egui_context,
    Some(&mut undo_system),
    Some(&mut world),
    Some(&mut selection_system),
);
```

## Architecture Decisions

### 1. Mutable World References
The hierarchy panel requires `&mut World` for:
- Executing undo commands (SetParentCommand)
- Creating/deleting entities through EntityOperations
- Ensuring proper ECS state consistency

### 2. EntityOperations Integration
Rather than directly manipulating entities, all operations go through EntityOperations which:
- Automatically creates undo commands
- Provides consistent error handling
- Ensures proper component state capture

### 3. Expansion State Management
The panel maintains its own expanded state (HashSet<Entity>) rather than storing it on entities because:
- It's UI state, not game state
- Avoids polluting ECS with editor-specific components
- Simpler to serialize/deserialize editor preferences

### 4. Drag-and-Drop Implementation
Custom implementation using egui primitives:
- `allocate_exact_size()` with `Sense::click_and_drag()`
- Manual hover detection and visual feedback
- State tracking with `drag_entity: Option<Entity>`

## Future Enhancements

Potential improvements for future iterations:

1. **Context Menu**: Right-click menu for entity operations (duplicate, copy, paste)
2. **Search/Filter**: Find entities by name or component type
3. **Multi-entity Drag**: Drag multiple selected entities simultaneously
4. **Entity Icons**: Visual indicators for entity types (camera, light, mesh, etc.)
5. **Performance Optimization**: Virtual scrolling for scenes with thousands of entities
6. **Keyboard Navigation**: Arrow keys to navigate tree, Enter to rename
7. **Entity Renaming**: Double-click to rename entities inline
8. **Visibility Toggle**: Eye icon to toggle entity visibility
9. **Lock/Unlock**: Prevent accidental selection or modification
10. **Sorting Options**: Sort by name, creation time, or custom order

## Testing Recommendations

To test the hierarchy panel:

1. **Create entities** using the Create Entity button
2. **Drag entities** onto each other to create parent-child relationships
3. **Use Ctrl+Z/Y** to undo/redo reparenting operations
4. **Select multiple entities** with Shift+Click
5. **Delete selected entities** and verify undo works
6. **Test circular hierarchy prevention** by trying to make a child the parent of its ancestor
7. **Verify live updates** by spawning entities programmatically

## Related Components

- **EntityOperations**: High-level API for entity manipulation with undo
- **SelectionSystem**: Multi-entity selection with events
- **UndoRedoSystem**: Command history and undo/redo management
- **SetParentCommand**: Command for changing entity parent relationships
- **EditorState**: Main editor coordinator integrating all panels

## Documentation

Full API documentation is available in:
- `crates/praxis_editor/src/panels/hierarchy_panel.rs`
- `crates/praxis_editor/src/entity_operations.rs`
- `crates/praxis_editor/src/selection.rs`
- `crates/praxis_editor/src/undo.rs`
