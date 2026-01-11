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
├── Console panel
├── Scene management
├── Save/load system
└── Serialization
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

**Practice** (10-14 hours):
1. Read [Asset Browser](../editor/asset-browser.md)
2. Read [Gizmos](../editor/gizmos.md)
3. Read [Console Panel](../editor/console-panel.md)
4. Read [Scene Format](../scene-format-v2.md)
5. Practice workflows

**Features**:
- **Asset Browser**: Drag-and-drop assets
- **Transform Gizmos**: Translate, rotate, scale tools
- **Multi-Selection**: Shift+click, box select
- **Console Panel**: Execute commands, view logs, debug
- **Scene Management**: Save/load scenes
- **Serialization**: Persist game state and scenes

### Console Panel

The console panel provides a command-line interface for debugging and runtime control.

**Console Features**:
- Execute commands at runtime
- View log messages and warnings
- Command history (↑/↓ arrows)
- Auto-completion (Tab key)
- Custom command registration

**Basic Console Commands**:
```rust
// Spawn entity
> spawn cube at 0,5,0

// Query entities
> list entities

// Modify component
> set transform position 1,2,3

// Debug info
> stats
> fps
```

**Run Console Demo**:
```bash
cargo run --example scripting_console_demo
```

### Save/Load System

Praxis supports comprehensive scene and game state persistence.

**Scene Serialization**:
```rust
use praxis_scene::{Scene, SceneManager};

// Save scene
let scene = Scene::from_world(&world);
scene.save_to_file("scenes/my_level.scene")?;

// Load scene
let scene = Scene::load_from_file("scenes/my_level.scene")?;
scene.spawn_into_world(&mut world);
```

**Serialization Format**:
- Human-readable JSON
- Includes entities, components, hierarchy
- Asset references preserved
- Transform hierarchy maintained

**Example Scene File**:
```json
{
  "entities": [
    {
      "id": 1,
      "components": {
        "Transform": {
          "translation": [0.0, 5.0, 0.0],
          "rotation": [0.0, 0.0, 0.0, 1.0],
          "scale": [1.0, 1.0, 1.0]
        },
        "MeshHandle": "cube.obj"
      }
    }
  ]
}
```

**Run Scene Serialization Demo**:
```bash
cargo run --example scene_serialization_demo
```

### Exercises

#### Exercise 1: Asset Workflow
1. Drag model from asset browser
2. Use gizmos to position
3. Multi-select and move group
4. Save scene, reload

#### Exercise 2: Build Custom Console Commands
Build a console command system for your game:

```rust
use praxis_scripting::{ScriptingContext, ConsoleCommand};

// Define custom command
struct TeleportCommand;

impl ConsoleCommand for TeleportCommand {
    fn name(&self) -> &str {
        "teleport"
    }
    
    fn help(&self) -> &str {
        "teleport <entity> <x> <y> <z> - Teleport entity to position"
    }
    
    fn execute(&self, args: &[&str], world: &mut World) -> Result<String> {
        let entity_id: u32 = args[0].parse()?;
        let x: f32 = args[1].parse()?;
        let y: f32 = args[2].parse()?;
        let z: f32 = args[3].parse()?;
        
        // Find entity and update transform
        let entity = world.entity_from_id(entity_id)?;
        if let Some(mut transform) = world.get_mut::<Transform>(entity) {
            transform.translation = Vec3::new(x, y, z);
            Ok(format!("Teleported entity {} to ({}, {}, {})", entity_id, x, y, z))
        } else {
            Err("Entity has no Transform component".into())
        }
    }
}

// Register command
context.register_command(Box::new(TeleportCommand))?;
```

**Tasks**:
1. Implement a `spawn` command that creates entities
2. Add a `delete` command to remove entities
3. Create a `save_state` command for game state persistence
4. Build a `load_state` command to restore game state

#### Exercise 3: Implement Game State Persistence
Create a save/load system for game progress:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct GameState {
    player_position: Vec3,
    inventory: Vec<String>,
    quest_progress: HashMap<String, u32>,
    entities: Vec<EntityData>,
}

impl GameState {
    fn save(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    fn load(path: &str) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let state = serde_json::from_str(&json)?;
        Ok(state)
    }
    
    fn from_world(world: &World) -> Self {
        // Extract game state from ECS world
        let mut state = GameState::default();
        
        // Query player position
        for (transform, player) in world.query::<(&Transform, &Player)>().iter() {
            state.player_position = transform.translation;
        }
        
        // Save other game data...
        state
    }
    
    fn apply_to_world(&self, world: &mut World) {
        // Restore game state to ECS world
        // Spawn entities, set positions, restore progress
    }
}
```

**Tasks**:
1. Define your game's state structure
2. Implement serialization for custom components
3. Add autosave functionality
4. Create multiple save slots
5. Handle save data versioning

#### Exercise 4: Scene Editing Workflow
1. Create a level with multiple entities
2. Use console to debug entity positions
3. Save the scene to a file
4. Modify the scene file manually
5. Reload and verify changes
6. Export game state at specific points

### Checkpoint
- [ ] Asset workflow efficient
- [ ] Gizmo manipulation smooth
- [ ] Can use console for debugging
- [ ] Understand serialization format
- [ ] Can save/load scenes
- [ ] Built custom console commands
- [ ] Implemented game state persistence

**Time**: 14-18 hours

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
