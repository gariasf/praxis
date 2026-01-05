# Editor Learning Path

Master the Praxis editor tools for level design, debugging, and custom tool development.

## Path Overview

**Time Investment**: 1-2 weeks  
**Prerequisites**: Basic engine usage  
**Final Goal**: Build custom editor tools and workflows

## Progression Map

```
Beginner (3-4 days)
├── Editor navigation
├── Entity selection
├── Hierarchy panel
└── Inspector basics
    ↓
Intermediate (4-5 days)
├── Asset browser
├── Transform gizmos
├── Multi-selection
└── Scene management
    ↓
Advanced (5-6 days)
├── Undo/redo system
├── Custom panels
├── Command system
└── Editor extensions
```

---

## Beginner: Core Tools

**Practice** (6-8 hours):
1. Read [Editor Overview](../editor/README.md)
2. Run `cargo run --example editor_demo`
3. Practice navigation and selection

**Core Tools**:
- [Editor Camera](../editor/editor-camera.md) - Navigation controls
- [Hierarchy Panel](../editor/hierarchy-panel.md) - Entity tree
- [Inspector](../editor/inspector.md) - Component editing
- [Selection](../editor/selection-system.md) - Entity picking

**Camera Controls**:
- Middle mouse drag: Orbit
- Scroll: Zoom
- Right click drag: Pan
- F: Focus on selected

**Exercises**:
1. Navigate scene
2. Select entities
3. Modify transforms in inspector
4. Parent/unparent entities

### Checkpoint
- [ ] Comfortable with camera
- [ ] Can select and edit entities
- [ ] Understand hierarchy

**Time**: 8-10 hours

---

## Intermediate: Advanced Features

**Practice** (8-10 hours):
1. Read [Asset Browser](../editor/asset-browser.md)
2. Read [Gizmos](../editor/gizmos.md)
3. Practice workflows

**Features**:
- Drag-and-drop assets
- Transform gizmos (translate, rotate, scale)
- Multi-entity selection (Shift+click, box select)
- Scene save/load

**Exercises**:
1. Drag model from asset browser
2. Use gizmos to position
3. Multi-select and move group
4. Save scene, reload

### Checkpoint
- [ ] Asset workflow efficient
- [ ] Gizmo manipulation smooth
- [ ] Can manage scenes

**Time**: 10-12 hours

---

## Advanced: Editor Extensions

**Theory** (4-5 hours):
1. Read [Undo/Redo](../editor/undo-redo.md)
2. Read [Command System](../editor/README.md)
3. Study editor architecture

**Practice** (10-12 hours):
1. Implement undo/redo for custom tools
2. Create custom panel
3. Add custom commands

**Example Custom Panel**:
```rust
struct MyCustomPanel;

impl Panel for MyCustomPanel {
    fn title(&self) -> &str {
        "My Tool"
    }

    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        if ui.button("Do Something").clicked() {
            // Custom tool logic
        }
    }
}

// Register panel
editor.add_panel(Box::new(MyCustomPanel));
```

**Example Command**:
```rust
struct SpawnCubeCommand {
    position: Vec3,
}

impl Command for SpawnCubeCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        world.spawn((
            Transform::from_translation(self.position),
            MeshHandle::new("cube"),
        ));
        Ok(())
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        // Remove spawned cube
        Ok(())
    }
}
```

**Run Examples**:
```bash
cargo run --example undo_redo_system_demo
cargo run --example command_system_demo
```

### Checkpoint
- [ ] Undo/redo working
- [ ] Created custom panel
- [ ] Extended editor functionality

**Time**: 15-20 hours

---

## Cross-References

- [Scene Format](../scene-format-v2.md) - Scene serialization
- [Input Guide](../guides/input.md) - Editor shortcuts
- [ECS Patterns](../architecture/ecs-patterns.md) - Editor ECS integration

---

[← Back to Learning Paths](README.md) | [Next: Assets Path →](assets.md)
