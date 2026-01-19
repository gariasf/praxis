# Menu Bar

Comprehensive, standards-compliant menu system for the Praxis editor with keyboard shortcuts, undo/redo integration, and dirty state tracking.

## Overview

The Menu Bar provides industry-standard menus (File, Edit, Entity, View, Help) with full keyboard shortcut support and visual feedback for editor state.

## Menu Structure

### File Menu
```
File
├── New Scene          Ctrl+N
├── Open Scene...      Ctrl+O
├── ─────────────
├── Save Scene *       Ctrl+S         (* = dirty indicator)
├── Save Scene As...   Ctrl+Shift+S
├── ─────────────
└── Exit               Alt+F4
```

### Edit Menu
```
Edit
├── Undo: Move Entity    Ctrl+Z    (shows command description)
├── Redo: Delete Entity  Ctrl+Y
├── ─────────────
├── Copy                 Ctrl+C
├── Paste                Ctrl+V
└── Duplicate            Ctrl+D
```

### Entity Menu
```
Entity
├── Create Empty
├── Create Primitive ▸
│   ├── Cube
│   ├── Sphere
│   ├── Plane
│   ├── Cylinder
│   └── Cone
└── Delete               Delete
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
└── Documentation        F1
```

## Keyboard Shortcuts

### File Operations
| Shortcut | Action |
|----------|--------|
| **Ctrl+N** | New Scene |
| **Ctrl+O** | Open Scene |
| **Ctrl+S** | Save Scene |
| **Ctrl+Shift+S** | Save Scene As |
| **Alt+F4** | Exit |

### Edit Operations
| Shortcut | Action |
|----------|--------|
| **Ctrl+Z** | Undo |
| **Ctrl+Y** | Redo |
| **Ctrl+C** | Copy |
| **Ctrl+V** | Paste |
| **Ctrl+D** | Duplicate |

### Entity Operations
| Shortcut | Action |
|----------|--------|
| **Delete** | Delete Selected |

### Help
| Shortcut | Action |
|----------|--------|
| **F1** | Open Documentation |

## Architecture

### Three-Component Design

```rust
// 1. Actions: What can be done
pub enum MenuBarAction {
    NewScene, OpenScene, SaveScene, SaveSceneAs, Exit,
    Undo, Redo, Copy, Paste, Duplicate,
    CreateEmpty, CreateCube, /* ... */, DeleteSelected,
    ShowAbout, OpenDocumentation,
}

// 2. State: UI state (mode, panel visibility)
pub struct MenuBarState {
    pub mode: EditorMode,
    pub show_hierarchy: bool,
    pub show_inspector: bool,
    // ...
}

// 3. Functions: UI rendering and action handling
pub fn render_menu_bar() -> Vec<MenuBarAction>
pub fn check_keyboard_shortcuts() -> Vec<MenuBarAction>
pub fn handle_menu_action(action, state, undo, world)
```

**Benefits**:
- Clean separation between UI and logic
- Actions can be triggered from multiple sources
- Easy to test and extend
- Custom handling of specific actions

## Usage

### Automatic Integration

The menu bar is automatically integrated with `EditorState`:

```rust
use praxis_editor::EditorState;

let mut editor = EditorState::new();

// Menu bar automatically rendered and handled
editor.ui(&egui_context, Some(&mut undo_system), Some(&mut world));
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

### Custom Action Handling

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
            // Default handling for others
            handle_menu_action(action, &mut state, Some(&mut undo), Some(&mut world));
        }
    }
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

Menu items automatically enable/disable based on availability:

```rust
// In render_menu_bar()
if let Some(undo) = undo_system {
    ui.add_enabled_ui(undo.can_undo(), |ui| {
        if ui.button(format!("Undo: {}", undo.undo_description())).clicked() {
            actions.push(MenuBarAction::Undo);
        }
    });
}
```

### Dirty State Indicator

When unsaved changes exist:
- **Menu**: "Save Scene *" (asterisk)
- **Status Bar**: "● Unsaved" in yellow

After save:
- **Menu**: "Save Scene" (no asterisk)
- **Status Bar**: "✓ Saved" in green

```rust
let is_dirty = undo_system
    .as_ref()
    .map(|u| u.is_dirty())
    .unwrap_or(false);

if is_dirty {
    ui.button("Save Scene *");
} else {
    ui.button("Save Scene");
}
```

## Panel Visibility

View menu checkboxes control panel visibility:

```rust
// Toggle panels
ui.checkbox(&mut state.show_hierarchy, "Hierarchy");
ui.checkbox(&mut state.show_inspector, "Inspector");
// ...
```

EditorState synchronizes with MenuBarState:

```rust
// Only show if enabled
if menu_state.show_hierarchy {
    hierarchy_panel.ui(ui);
}
```

## Context-Aware Shortcuts

Shortcuts disabled when typing:

```rust
pub fn check_keyboard_shortcuts(ctx: &egui::Context) -> Vec<MenuBarAction> {
    let mut actions = Vec::new();
    
    // Don't process shortcuts when typing in text fields
    if ctx.wants_keyboard_input() {
        return actions;
    }
    
    // Check shortcuts...
}
```

This prevents:
- Accidentally saving when typing "S"
- Triggering commands while entering entity names
- Conflicts with text editing shortcuts

## Advanced Features

### Recent Files List

```rust
struct RecentFiles {
    files: Vec<PathBuf>,
    max_count: usize,
}

impl RecentFiles {
    pub fn render(&self, ui: &mut egui::Ui) -> Option<PathBuf> {
        ui.separator();
        for (i, path) in self.files.iter().enumerate() {
            if ui.button(format!("{}. {}", i + 1, path.display())).clicked() {
                return Some(path.clone());
            }
        }
        None
    }
}
```

### Conditional Menu Items

Enable/disable based on selection:

```rust
let has_selection = selection
    .as_ref()
    .map(|s| s.selected_count() > 0)
    .unwrap_or(false);

ui.add_enabled_ui(has_selection, |ui| {
    if ui.button("Delete").clicked() {
        actions.push(MenuBarAction::DeleteSelected);
    }
});
```

### Command Palette

Quick command search (Ctrl+P):

```rust
struct CommandPalette {
    open: bool,
    search: String,
    commands: Vec<(String, MenuBarAction)>,
}

impl CommandPalette {
    pub fn show(&mut self, ctx: &egui::Context) -> Option<MenuBarAction> {
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::P)) {
            self.open = true;
        }
        
        if self.open {
            egui::Window::new("Command Palette").show(ctx, |ui| {
                ui.text_edit_singleline(&mut self.search);
                
                let filtered: Vec<_> = self.commands
                    .iter()
                    .filter(|(name, _)| name.contains(&self.search))
                    .collect();
                
                for (name, action) in filtered {
                    if ui.button(name).clicked() {
                        return Some(*action);
                    }
                }
            });
        }
        None
    }
}
```

## Status Bar

The menu bar includes an integrated status bar on the right:

```rust
ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
    // Mode indicator
    ui.label(match state.mode {
        EditorMode::Edit => "⏸ Edit",
        EditorMode::Play => "▶ Play",
    });
    
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
- Verify `check_keyboard_shortcuts()` is called each frame
- Check if `wants_keyboard_input()` is blocking shortcuts
- Ensure egui context is properly updated

### Menu Items Always Disabled
- Verify `UndoRedoSystem` is passed to `render_menu_bar()`
- Check undo system has commands
- Ensure `can_undo()` / `can_redo()` work correctly

### Dirty State Not Updating
- Verify commands executed through undo system
- Check `mark_saved()` called after save
- Ensure `mark_dirty()` called on commands

### Panel Visibility Not Syncing
- Verify `MenuBarState` is shared
- Check panel rendering respects visibility flags
- Use mutable reference to state

## Examples

See `examples/menu_bar_demo.rs` for complete demonstration.

## Technical Details

For implementation details, see:
- [crates/praxis_editor/MENU_BAR.md](../../crates/praxis_editor/MENU_BAR.md) - Complete implementation documentation
- Action handling details
- Keyboard shortcut processing
- State synchronization

## See Also

- [Undo/Redo System](undo-redo.md) - Command history integration
- [Panels](panels.md) - Panel visibility management
- [Editor Overview](editor-overview.md) - Overall architecture
