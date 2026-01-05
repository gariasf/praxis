# Menu Bar Guide

## Overview

The Menu Bar provides a comprehensive, industry-standard menu system for the Praxis editor with full keyboard shortcut support, undo/redo integration, and panel visibility management.

## Design Philosophy

The menu bar follows established editor conventions:
- **Standard menus**: File, Edit, Entity, View, Help
- **Keyboard shortcuts**: All common operations (Ctrl+S, Ctrl+Z, etc.)
- **Context awareness**: Menu items enable/disable based on state
- **Dirty state tracking**: Visual indicator for unsaved changes
- **Action-based**: Clean separation between UI and logic

## Architecture

### Three-Component Design

```rust
// 1. Actions: What can be done
pub enum MenuBarAction { /* 22 total actions */ }

// 2. State: UI state (mode, panel visibility)
pub struct MenuBarState { /* mode, panel flags */ }

// 3. Functions: UI rendering and action handling
pub fn render_menu_bar() -> Vec<MenuBarAction>
pub fn check_keyboard_shortcuts() -> Vec<MenuBarAction>
pub fn handle_menu_action(action, state, undo, world)
```

### Design Rationale

**Action-Based Architecture**: 
- UI rendering returns actions (what user wants to do)
- Action handling executes actions (how to do it)
- Clean separation enables custom handling and testing

**State Management**:
- MenuBarState tracks UI state only
- Game/editor state managed separately
- Easy to serialize for preferences

## Menu Structure

### File Menu

```
File
├── New Scene         Ctrl+N
├── Open Scene...     Ctrl+O
├── Save Scene *      Ctrl+S      (* = dirty indicator)
├── Save Scene As...  Ctrl+Shift+S
├── ─────────────
└── Exit              Alt+F4
```

### Edit Menu

```
Edit
├── Undo: Move Entity    Ctrl+Z    (shows command description)
├── Redo: Delete Entity  Ctrl+Y
├── ─────────────
├── Copy              Ctrl+C
├── Paste             Ctrl+V
└── Duplicate         Ctrl+D
```

### Entity Menu

```
Entity
├── Create Empty
├── Create Primitive ▶
│   ├── Cube
│   ├── Sphere
│   ├── Plane
│   ├── Cylinder
│   └── Cone
└── Delete            Delete
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
└── Documentation     F1
```

## Keyboard Shortcuts

### File Operations

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Ctrl+N` | New Scene | Clear scene, create new |
| `Ctrl+O` | Open Scene | Show file picker |
| `Ctrl+S` | Save Scene | Save to current file |
| `Ctrl+Shift+S` | Save As | Show save dialog |
| `Alt+F4` | Exit | Close editor |

### Edit Operations

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Ctrl+Z` | Undo | Undo last command |
| `Ctrl+Y` | Redo | Redo last undone command |
| `Ctrl+C` | Copy | Copy selected entities |
| `Ctrl+V` | Paste | Paste copied entities |
| `Ctrl+D` | Duplicate | Duplicate selection |

### Entity Operations

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Delete` | Delete | Delete selected entities |

### Help Operations

| Shortcut | Action | Description |
|----------|--------|-------------|
| `F1` | Documentation | Open documentation |

## Basic Usage

### Integration with EditorState

The menu bar is automatically integrated with EditorState:

```rust
use praxis_editor::EditorState;

let mut editor = EditorState::new();

// In your render loop
editor.ui(
    &egui_context,
    Some(&mut undo_system),
    Some(&mut world),
);

// Menu bar automatically rendered and handled
```

### Manual Integration

For custom editors, use the three-function pattern:

```rust
use praxis_editor::menu_bar::*;

// 1. Render menu and collect actions
let mut actions = render_menu_bar(
    &egui_context,
    &mut menu_bar_state,
    Some(&undo_system),
);

// 2. Check keyboard shortcuts
actions.extend(check_keyboard_shortcuts(&egui_context));

// 3. Handle all actions
for action in actions {
    handle_menu_action(
        action,
        &mut menu_bar_state,
        Some(&mut undo_system),
        Some(&mut world),
    );
}
```

## MenuBarAction

Enum representing all possible menu actions:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuBarAction {
    // File
    NewScene,
    OpenScene,
    SaveScene,
    SaveSceneAs,
    Exit,
    
    // Edit
    Undo,
    Redo,
    Copy,
    Paste,
    Duplicate,
    
    // Entity
    CreateEmpty,
    CreateCube,
    CreateSphere,
    CreatePlane,
    CreateCylinder,
    CreateCone,
    DeleteSelected,
    
    // View
    ToggleModeEditPlay,
    
    // Help
    ShowAbout,
    OpenDocumentation,
}
```

## MenuBarState

Tracks menu and panel state:

```rust
pub struct MenuBarState {
    pub mode: EditorMode,
    pub show_hierarchy: bool,
    pub show_inspector: bool,
    pub show_console: bool,
    pub show_assets: bool,
    pub show_scene_view: bool,
}

pub enum EditorMode {
    Edit,
    Play,
}
```

**Default State**:
```rust
MenuBarState {
    mode: EditorMode::Edit,
    show_hierarchy: true,
    show_inspector: true,
    show_console: true,
    show_assets: true,
    show_scene_view: true,
}
```

## Undo/Redo Integration

### Command Descriptions

Undo/Redo menu items show command descriptions:

```
Undo: Move Entity    Ctrl+Z
Redo: Delete Entity  Ctrl+Y
```

### Enabled State

Menu items automatically enable/disable:

```rust
// In render_menu_bar()
if let Some(undo) = undo_system {
    ui.add_enabled_ui(undo.can_undo(), |ui| {
        if ui.button(format!("Undo: {}", undo.undo_description())).clicked() {
            actions.push(MenuBarAction::Undo);
        }
    });
} else {
    ui.add_enabled(false, egui::Button::new("Undo"));
}
```

### Executing Commands

```rust
// In handle_menu_action()
MenuBarAction::Undo => {
    if let Some(undo) = undo_system {
        undo.undo(world.unwrap());
    }
}
```

## Dirty State Tracking

### Visual Indicators

**Menu Bar**:
- "Save Scene *" when unsaved changes exist
- Normal "Save Scene" when clean

**Status Bar**:
- "● Unsaved" in yellow when dirty
- "✓ Saved" in green when clean

### Implementation

```rust
// Check dirty state
let is_dirty = undo_system
    .as_ref()
    .map(|u| u.is_dirty())
    .unwrap_or(false);

// Render save button
if is_dirty {
    if ui.button("Save Scene *").clicked() {
        actions.push(MenuBarAction::SaveScene);
    }
} else {
    if ui.button("Save Scene").clicked() {
        actions.push(MenuBarAction::SaveScene);
    }
}
```

### Clearing Dirty Flag

```rust
// After successful save
MenuBarAction::SaveScene => {
    save_scene(&world)?;
    if let Some(undo) = undo_system {
        undo.mark_saved();
    }
}
```

## Panel Visibility

### Toggle Panels

View menu checkboxes control panel visibility:

```rust
// In render_menu_bar()
ui.checkbox(&mut state.show_hierarchy, "Hierarchy");
ui.checkbox(&mut state.show_inspector, "Inspector");
ui.checkbox(&mut state.show_console, "Console");
ui.checkbox(&mut state.show_assets, "Assets");
ui.checkbox(&mut state.show_scene_view, "Scene View");
```

### Synchronization

EditorState synchronizes with MenuBarState:

```rust
// In EditorState::ui()
let menu_state = self.menu_bar_state();

// Only show panels if enabled
if menu_state.show_hierarchy {
    self.hierarchy_panel.ui(ui);
}
if menu_state.show_inspector {
    self.inspector_panel.ui(ui);
}
// ...
```

## Mode Switching

### Edit vs. Play Mode

Toggle button in menu bar:

```rust
// Visual indicator
let mode_text = match state.mode {
    EditorMode::Edit => "⏸ Edit",
    EditorMode::Play => "▶ Play",
};

if ui.button(mode_text).clicked() {
    actions.push(MenuBarAction::ToggleModeEditPlay);
}
```

### Mode Effects

```rust
MenuBarAction::ToggleModeEditPlay => {
    state.mode = match state.mode {
        EditorMode::Edit => {
            // Start play mode
            save_editor_state();
            EditorMode::Play
        }
        EditorMode::Play => {
            // Return to edit mode
            restore_editor_state();
            EditorMode::Edit
        }
    };
}
```

## Context-Aware Shortcuts

### Text Input Detection

Shortcuts disabled when typing in text fields:

```rust
pub fn check_keyboard_shortcuts(ctx: &egui::Context) -> Vec<MenuBarAction> {
    let mut actions = Vec::new();
    
    // Don't process shortcuts when typing
    if ctx.wants_keyboard_input() {
        return actions;
    }
    
    // Check shortcuts
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
        actions.push(MenuBarAction::SaveScene);
    }
    
    actions
}
```

This prevents:
- Accidentally saving when typing "S" in a text field
- Triggering commands while entering entity names
- Conflicts with text editing shortcuts

## Custom Action Handling

### Selective Handling

Handle specific actions yourself:

```rust
let actions = render_menu_bar(ctx, &mut state, Some(&undo));

for action in actions {
    match action {
        MenuBarAction::SaveScene => {
            // Custom save logic
            custom_save_scene(&world)?;
            undo.mark_saved();
        }
        MenuBarAction::OpenDocumentation => {
            // Custom documentation viewer
            open_internal_docs();
        }
        _ => {
            // Use default handling for others
            handle_menu_action(action, &mut state, Some(&mut undo), Some(&mut world));
        }
    }
}
```

### Adding Custom Actions

Extend MenuBarAction enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuBarAction {
    // ... existing actions
    
    // Custom actions
    ExportToGLTF,
    ImportAssets,
    RunScripts,
}

// Add to menu rendering
fn render_custom_menu(ui: &mut egui::Ui) -> Vec<MenuBarAction> {
    let mut actions = Vec::new();
    
    ui.menu_button("Tools", |ui| {
        if ui.button("Export to GLTF").clicked() {
            actions.push(MenuBarAction::ExportToGLTF);
        }
        if ui.button("Import Assets...").clicked() {
            actions.push(MenuBarAction::ImportAssets);
        }
    });
    
    actions
}
```

## Advanced Features

### Recent Files List

Add recent files to File menu:

```rust
struct RecentFiles {
    files: Vec<PathBuf>,
    max_count: usize,
}

impl RecentFiles {
    pub fn add(&mut self, path: PathBuf) {
        self.files.retain(|p| p != &path);
        self.files.insert(0, path);
        self.files.truncate(self.max_count);
    }
    
    pub fn render(&self, ui: &mut egui::Ui) -> Option<PathBuf> {
        let mut clicked = None;
        
        ui.separator();
        for (i, path) in self.files.iter().enumerate() {
            let label = format!("{}. {}", i + 1, path.display());
            if ui.button(label).clicked() {
                clicked = Some(path.clone());
            }
        }
        
        clicked
    }
}
```

### Conditional Menu Items

Enable/disable based on selection:

```rust
// In render_menu_bar()
let has_selection = selection
    .as_ref()
    .map(|s| s.selected_count() > 0)
    .unwrap_or(false);

ui.add_enabled_ui(has_selection, |ui| {
    if ui.button("Delete").clicked() {
        actions.push(MenuBarAction::DeleteSelected);
    }
    if ui.button("Duplicate").clicked() {
        actions.push(MenuBarAction::Duplicate);
    }
});
```

### Menu Search / Command Palette

Implement quick command search:

```rust
struct CommandPalette {
    open: bool,
    search: String,
    commands: Vec<(String, MenuBarAction)>,
}

impl CommandPalette {
    pub fn show(&mut self, ctx: &egui::Context) -> Option<MenuBarAction> {
        let mut action = None;
        
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::P)) {
            self.open = true;
        }
        
        if self.open {
            egui::Window::new("Command Palette")
                .show(ctx, |ui| {
                    ui.text_edit_singleline(&mut self.search);
                    
                    let filtered: Vec<_> = self.commands
                        .iter()
                        .filter(|(name, _)| name.contains(&self.search))
                        .collect();
                    
                    for (name, cmd_action) in filtered {
                        if ui.button(name).clicked() {
                            action = Some(*cmd_action);
                            self.open = false;
                        }
                    }
                });
        }
        
        action
    }
}
```

## Status Bar

The menu bar includes an integrated status bar:

```rust
// Right side of menu bar
ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
    // Mode indicator
    let mode_text = match state.mode {
        EditorMode::Edit => "⏸ Edit",
        EditorMode::Play => "▶ Play",
    };
    ui.label(mode_text);
    
    ui.separator();
    
    // Dirty state
    if is_dirty {
        ui.colored_label(egui::Color32::YELLOW, "● Unsaved");
    } else {
        ui.colored_label(egui::Color32::GREEN, "✓ Saved");
    }
    
    ui.separator();
    
    // Undo stack info
    if let Some(undo) = undo_system {
        ui.label(format!("Undo: {}", undo.undo_count()));
    }
});
```

## Troubleshooting

### Shortcuts Not Working

**Problem**: Keyboard shortcuts don't trigger actions

**Solutions**:
- Verify `check_keyboard_shortcuts()` is called each frame
- Check if `wants_keyboard_input()` is blocking shortcuts
- Ensure egui context is properly updated
- Test with simple action (e.g., Ctrl+S for save)

### Menu Items Always Disabled

**Problem**: Undo/Redo always grayed out

**Solutions**:
- Verify UndoRedoSystem is passed to `render_menu_bar()`
- Check undo system actually has commands
- Ensure `can_undo()` / `can_redo()` work correctly
- Add debug logging to menu rendering

### Dirty State Not Updating

**Problem**: Save indicator doesn't show asterisk

**Solutions**:
- Verify commands are executed through undo system
- Check `mark_dirty()` is called on commands
- Ensure `mark_saved()` called after save
- Test with simple command (move entity)

### Panel Visibility Not Syncing

**Problem**: Unchecking View menu doesn't hide panel

**Solutions**:
- Verify MenuBarState is shared between menu and editor
- Check panel rendering respects visibility flags
- Ensure state updates are propagated
- Use mutable reference to state

## Complete Example

```rust
use praxis_editor::menu_bar::*;
use praxis_editor::{EditorState, UndoRedoSystem};
use praxis_ecs::World;

fn main() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut menu_state = MenuBarState::default();
    
    // Game loop
    loop {
        egui_context.run(|ctx| {
            // Render menu bar
            let mut actions = render_menu_bar(
                ctx,
                &mut menu_state,
                Some(&undo_system),
            );
            
            // Check keyboard shortcuts
            actions.extend(check_keyboard_shortcuts(ctx));
            
            // Handle actions
            for action in actions {
                match action {
                    MenuBarAction::SaveScene => {
                        println!("Saving scene...");
                        // Your save logic
                        undo_system.mark_saved();
                    }
                    MenuBarAction::Exit => {
                        println!("Exiting...");
                        std::process::exit(0);
                    }
                    _ => {
                        handle_menu_action(
                            action,
                            &mut menu_state,
                            Some(&mut undo_system),
                            Some(&mut world),
                        );
                    }
                }
            }
        });
    }
}
```

## See Also

- [Undo/Redo System](undo-redo.md)
- [Editor Overview](README.md)
- [Panels Guide](panels.md)
- [Keyboard Shortcuts Reference](../reference/shortcuts.md)
