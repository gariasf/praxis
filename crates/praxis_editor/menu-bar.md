# MenuBar System

The MenuBar system provides a comprehensive, standards-compliant menu bar for the Praxis editor with full keyboard shortcut support.

## Overview

The menu bar includes five main menus:
- **File**: Scene management and application control
- **Edit**: Editing operations with undo/redo support
- **Entity**: Entity creation and deletion
- **View**: Panel visibility toggles
- **Help**: Documentation and about dialog

## Architecture

### Core Components

**`MenuBarAction`**: Enum representing all possible menu actions
- File actions: `NewScene`, `OpenScene`, `SaveScene`, `SaveSceneAs`, `Exit`
- Edit actions: `Undo`, `Redo`, `Copy`, `Paste`, `Duplicate`
- Entity actions: `CreateEmpty`, `CreateCube`, `CreateSphere`, `CreatePlane`, `CreateCylinder`, `CreateCone`, `DeleteEntity`
- View actions: `ToggleHierarchy`, `ToggleInspector`, `ToggleConsole`, `ToggleAssets`, `ToggleScene`
- Help actions: `About`, `Documentation`
- Mode toggle: `TogglePlayMode`

**`MenuBarState`**: Tracks menu bar state including:
- Current editor mode (Edit/Play)
- Panel visibility flags for all panels
- Synchronized with `EditorState`

### Key Functions

**`render_menu_bar()`**: Renders the menu bar UI and returns triggered actions
- Displays all menus with proper styling
- Shows keyboard shortcuts next to menu items
- Integrates with `UndoRedoSystem` for undo/redo state
- Shows dirty state indicator when there are unsaved changes
- Returns `Vec<MenuBarAction>` of actions triggered during the frame

**`check_keyboard_shortcuts()`**: Processes keyboard input for shortcuts
- Checks for all standard shortcuts (Ctrl+N/O/S, etc.)
- Respects text input context (doesn't trigger when typing)
- Returns `Vec<MenuBarAction>` of shortcuts triggered during the frame

**`handle_menu_action()`**: Executes a menu action
- Handles all `MenuBarAction` variants
- Updates `MenuBarState` as needed
- Executes undo/redo commands through `UndoRedoSystem`
- Logs all actions for debugging

## Keyboard Shortcuts

### File Menu
- **Ctrl+N**: New Scene
- **Ctrl+O**: Open Scene
- **Ctrl+S**: Save Scene
- **Ctrl+Shift+S**: Save Scene As
- **Alt+F4**: Exit

### Edit Menu
- **Ctrl+Z**: Undo last action
- **Ctrl+Y**: Redo last undone action
- **Ctrl+C**: Copy selected entities
- **Ctrl+V**: Paste entities
- **Ctrl+D**: Duplicate selected entities

### Entity Menu
- **Delete**: Delete selected entity

### Help Menu
- **F1**: Open documentation

## Integration with EditorState

The `EditorState` automatically manages the menu bar:

```rust
use praxis_editor::{EditorState, UndoRedoSystem};
use bevy_ecs::world::World;

let mut editor = EditorState::new();
let mut undo_system = UndoRedoSystem::new();
let mut world = World::new();

// Render editor with menu bar
editor.ui(&egui_context, Some(&mut undo_system), Some(&mut world));
```

The `EditorState::ui()` method:
1. Renders the menu bar
2. Checks for keyboard shortcuts
3. Handles all triggered actions
4. Synchronizes state between `MenuBarState` and `EditorState`

## Advanced Usage

### Direct Menu Bar API

You can use the menu bar API directly without `EditorState`:

```rust
use praxis_editor::menu_bar::{
    render_menu_bar, check_keyboard_shortcuts, handle_menu_action, MenuBarState
};

let mut menu_state = MenuBarState::new();

// In your render loop:
let mut actions = render_menu_bar(&egui_ctx, &mut menu_state, Some(&undo_system));
actions.extend(check_keyboard_shortcuts(&egui_ctx));

for action in actions {
    handle_menu_action(action, &mut menu_state, Some(&mut undo_system), Some(&mut world));
}
```

### Custom Action Handling

You can handle actions yourself for custom behavior:

```rust
use praxis_editor::{MenuBarAction, MenuBarState};
use praxis_editor::menu_bar::{render_menu_bar, check_keyboard_shortcuts};

let actions = render_menu_bar(&egui_ctx, &mut menu_state, Some(&undo_system));

for action in actions {
    match action {
        MenuBarAction::NewScene => {
            // Custom new scene logic
            clear_scene(&mut world);
            info!("Created new scene");
        }
        MenuBarAction::SaveScene => {
            // Custom save logic
            save_scene_to_file(&world, "scene.ron")?;
            undo_system.mark_saved();
            info!("Scene saved");
        }
        _ => {
            // Use default handler for other actions
            handle_menu_action(action, &mut menu_state, Some(&mut undo_system), Some(&mut world));
        }
    }
}
```

## Visual Features

### Dirty State Indicator

When there are unsaved changes, the menu bar displays:
- **"Save Scene *"** in the File menu (asterisk indicates unsaved)
- **"● Unsaved"** indicator in the status bar (yellow dot)

The dirty state is automatically tracked by `UndoRedoSystem` and cleared when `SaveScene` action is executed.

### Undo/Redo Integration

The Edit menu displays:
- Command descriptions in menu items (e.g., "Undo: Move Entity")
- Enabled/disabled state based on whether undo/redo is available
- Keyboard shortcuts in the menu

Example display:
```
Undo: Move Entity (Ctrl+Z)    [enabled]
Redo: Delete Entity (Ctrl+Y)   [enabled]
```

### Panel Visibility Toggles

The View menu shows checkboxes for each panel:
- ✓ Hierarchy
- ✓ Inspector
- ✓ Console
- ✓ Assets
- ✓ Scene View

Toggling a panel immediately updates its visibility state.

## Menu Structure

### File Menu
```
File
├── New Scene          (Ctrl+N)
├── Open Scene         (Ctrl+O)
├── ─────────────
├── Save Scene         (Ctrl+S)      [* if dirty]
├── Save Scene As...   (Ctrl+Shift+S)
├── ─────────────
└── Exit               (Alt+F4)
```

### Edit Menu
```
Edit
├── Undo: [description]  (Ctrl+Z)
├── Redo: [description]  (Ctrl+Y)
├── ─────────────
├── Copy                 (Ctrl+C)
├── Paste                (Ctrl+V)
└── Duplicate            (Ctrl+D)
```

### Entity Menu
```
Entity
├── Create Empty
├── ─────────────
├── Create Primitive ▸
│   ├── Cube
│   ├── Sphere
│   ├── Plane
│   ├── Cylinder
│   └── Cone
├── ─────────────
└── Delete               (Delete)
```

### View Menu
```
View
├── ☑ Hierarchy
├── ☑ Inspector
├── ☑ Console
├── ☑ Assets
└── ☑ Scene View
```

### Help Menu
```
Help
├── About Praxis
└── Documentation        (F1)
```

## Implementation Notes

### Keyboard Shortcut Handling

Shortcuts are processed in two places:
1. **Menu bar rendering**: `render_menu_bar()` displays shortcuts and handles clicks
2. **Global shortcut checking**: `check_keyboard_shortcuts()` processes keyboard input

This dual approach ensures shortcuts work both when clicking menu items and when using the keyboard directly.

### Context Awareness

The shortcut system respects UI context:
- Shortcuts are disabled when typing in text fields (`ctx.wants_keyboard_input()`)
- This prevents accidental actions while editing text

### Undo/Redo Integration

The menu bar integrates seamlessly with the command system:
- `UndoRedoSystem` provides undo/redo state and descriptions
- Actions trigger through `system.undo()` and `system.redo()`
- Dirty state is automatically tracked and displayed
- `mark_saved()` clears dirty state on save

## Example

See `examples/menu_bar_demo.rs` for a complete working example demonstrating:
- All menu items and their functionality
- Keyboard shortcuts
- Undo/redo integration
- Dirty state tracking
- Panel visibility toggles
- Mode switching (Edit/Play)

## Future Enhancements

Potential improvements for the menu bar system:
- Recent files list in File menu
- Customizable keyboard shortcuts
- Menu item enable/disable based on selection
- Context-sensitive menus
- Plugin menu extensibility
- Localization support
