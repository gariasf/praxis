# Save/Load System

Praxis provides a comprehensive save/load system for persisting complete game state. The `SaveManager` handles full world serialization including entity hierarchies, components, asset references, and rich metadata with versioning support.

## Overview

The save system uses RON (Rusty Object Notation) for human-readable save files with automatic versioning and migration support.

**Features:**
- **Complete World Serialization**: All entities, components, and hierarchies
- **Asset Reference Tracking**: Meshes, textures, materials preserved by ID
- **Rich Metadata**: Timestamps, playtime, descriptions, custom data
- **Versioning**: Automatic migration between save format versions
- **Selective Persistence**: Exclude entities with `NoSave` component
- **Save Slots**: Multiple named save files
- **Fast Metadata Reading**: Read save info without loading full state

**File Format**: RON (`.ron` extension)
- Human-readable text format
- Easy to debug and manually edit
- Supports comments for development
- Compact with pretty-printing options

## Basic Usage

### Saving Game State

```rust
use praxis_scene::{SaveManager, SaveMetadata};
use praxis_ecs::World;

let mut world = World::new();
let mut save_manager = SaveManager::new();

// Create metadata
let metadata = SaveMetadata::new("Checkpoint 1")
    .with_description("Player at dungeon entrance")
    .with_playtime(1847); // seconds

// Save to file
save_manager.save_to_file(
    &world,
    "saves/slot1.ron",
    metadata
)?;

// Check statistics
if let Some(stats) = save_manager.last_stats() {
    println!("Saved {} entities in {:.2}ms", 
        stats.entity_count, stats.duration_ms);
}
```

### Loading Game State

```rust
let mut save_manager = SaveManager::new();
let mut world = World::new();

// Load replaces all world contents
save_manager.load_from_file(&mut world, "saves/slot1.ron")?;

// Verify load
if let Some(stats) = save_manager.last_stats() {
    println!("Loaded {} entities", stats.entity_count);
}
```

### Reading Metadata Only

For save file selection screens, read metadata without loading:

```rust
let metadata = save_manager.read_metadata("saves/slot1.ron")?;

println!("Save: {}", metadata.name);
println!("Time: {}", metadata.timestamp);
println!("Playtime: {}s", metadata.playtime_seconds);
```

## Save Metadata

### Basic Metadata

```rust
let metadata = SaveMetadata::new("Manual Save 1")
    .with_description("Before boss fight")
    .with_playtime(3600)
    .with_game_version("1.2.0");
```

### Rich Metadata

```rust
let metadata = SaveMetadata::new("Chapter 3 - The Fortress")
    .with_description("Player has entered the fortress")
    .with_playtime(7245)  // 2 hours 45 seconds
    .with_game_version("1.0.0")
    .with_screenshot("saves/screenshots/slot1.png")
    .with_tag("manual")
    .with_tag("chapter_3")
    .with_custom_data("location", "fortress_gate")
    .with_custom_data("difficulty", "hard")
    .with_custom_data("quest_progress", "12/20");
```

### Metadata Fields

```rust
pub struct SaveMetadata {
    pub name: String,                      // Display name
    pub description: Option<String>,       // Optional description
    pub timestamp: String,                 // ISO 8601 timestamp (auto-generated)
    pub playtime_seconds: u64,            // Total playtime
    pub game_version: Option<String>,      // Game version string
    pub screenshot_path: Option<String>,   // Relative screenshot path
    pub tags: Vec<String>,                 // Organization tags
    pub custom_data: HashMap<String, String>, // Custom key-value data
}
```

## Configuration

### Save Config

```rust
use praxis_scene::SaveConfig;

let config = SaveConfig {
    compress: false,              // Future: gzip compression
    include_editor_data: false,   // Save editor-specific data
    validate_after_save: true,    // Verify save integrity
    pretty_print: true,           // Human-readable formatting
};

let mut save_manager = SaveManager::with_config(config);
```

**Configuration Guidelines:**

| Setting | Production | Development | Description |
|---------|-----------|-------------|-------------|
| `compress` | `true` | `false` | Smaller files, slower I/O |
| `include_editor_data` | `false` | `true` | Editor metadata |
| `validate_after_save` | `true` | `true` | Integrity check |
| `pretty_print` | `false` | `true` | Readable vs. compact |

## Components Saved

The save system automatically serializes these components:

### Transform Components
- `Transform`: Local position, rotation, scale
- `GlobalTransform`: World-space transform (recomputed on load)
- `Parent`/`Children`: Hierarchy relationships

### Rendering Components
- `MeshHandle`: Mesh asset reference
- `TextureHandle`: Texture asset reference
- `MaterialHandle`: Material asset reference
- `Visibility`: Visible/Hidden state

### Lighting Components
- `DirectionalLight`: Direction, color, intensity
- `PointLight`: Color, intensity, range

### Camera Components
- `Camera`: Active state, priority
- `PerspectiveProjection`: FOV, aspect, near/far
- `OrthographicProjection`: Bounds, near/far

### Metadata Components
- `Name`: Entity name
- `Active`: Active state marker

### Excluded Components
- `NoSave`: Marker to exclude entity
- Temporary/runtime components (physics state, etc.)

## Selective Persistence

### Excluding Entities

Use `NoSave` component to exclude entities:

```rust
use praxis_ecs::NoSave;

// Temporary debug marker - not saved
world.spawn((
    Name("DebugVisualization".to_string()),
    Transform::from_xyz(0.0, 2.0, 0.0),
    NoSave, // This entity will not be saved
));

// Game entity - will be saved
world.spawn((
    Name("Player".to_string()),
    Transform::from_xyz(0.0, 0.0, 0.0),
    MeshHandle::new("player_mesh"),
));
```

**Use Cases for `NoSave`:**
- Debug visualizations
- UI elements
- Temporary effects
- Camera rigs (if using separate camera system)
- Editor-only entities

### Component Filtering

Currently all present components are saved. For custom component filtering:

```rust
// Before saving, temporarily remove component
let temp_components: Vec<_> = world
    .query::<(Entity, &TempComponent)>()
    .iter(&world)
    .map(|(e, c)| (e, *c))
    .collect();

for (entity, _) in &temp_components {
    world.entity_mut(*entity).remove::<TempComponent>();
}

save_manager.save_to_file(&world, path, metadata)?;

// Restore components
for (entity, component) in temp_components {
    world.entity_mut(entity).insert(component);
}
```

## Entity Hierarchies

The save system preserves parent-child relationships:

```rust
// Create hierarchy
let parent = world.spawn((
    Name("ParentObject".to_string()),
    Transform::from_xyz(5.0, 0.0, 0.0),
)).id();

let child = world.spawn((
    Name("ChildObject".to_string()),
    Transform::from_xyz(1.0, 1.0, 0.0), // Local to parent
    Parent(parent),
)).id();

// Add to parent's children list
if let Some(mut parent_entity) = world.get_entity_mut(parent) {
    if let Some(mut children) = parent_entity.get_mut::<Children>() {
        children.0.push(child);
    } else {
        parent_entity.insert(Children(vec![child]));
    }
}

// Save - hierarchy is preserved
save_manager.save_to_file(&world, "save.ron", metadata)?;

// Load - hierarchy is restored
let mut new_world = World::new();
save_manager.load_from_file(&mut new_world, "save.ron")?;
```

**Hierarchy Features:**
- Unlimited nesting depth
- Preserves transform parenting
- Maintains sibling order
- Automatically restores `Parent` and `Children` components

## Asset References

Asset handles are saved as string IDs and must be resolved on load:

### Saving with Assets

```rust
world.spawn((
    Name("Enemy".to_string()),
    Transform::default(),
    MeshHandle::new("orc_mesh"),           // Saved as "orc_mesh"
    TextureHandle::new("orc_texture"),     // Saved as "orc_texture"
    MaterialHandle::new("orc_material"),   // Saved as "orc_material"
));

save_manager.save_to_file(&world, "save.ron", metadata)?;
```

### Loading with Asset System

After loading, resolve asset handles:

```rust
// Load save file
save_manager.load_from_file(&mut world, "save.ron")?;

// Asset handles reference IDs like "orc_mesh"
// Your asset system should load these on-demand or have them preloaded

// Example: Load referenced assets
for (mesh_handle,) in world.query::<(&MeshHandle,)>().iter(&world) {
    if !asset_manager.is_loaded(&mesh_handle.id) {
        asset_manager.load_mesh(&mesh_handle.id)?;
    }
}
```

**Best Practice**: Preload common assets before loading saves, or implement lazy loading:

```rust
impl AssetManager {
    fn ensure_loaded(&mut self, handle: &MeshHandle) -> Result<()> {
        if !self.is_loaded(&handle.id) {
            self.load_mesh(&handle.id)?;
        }
        Ok(())
    }
}
```

## Save File Format

### Example Save File

```ron
SaveFile(
    version: 1,
    metadata: SaveMetadata(
        name: "Chapter 1 - Forest",
        description: Some("Player at forest entrance"),
        timestamp: "2024-01-15T14:30:00Z",
        playtime_seconds: 1847,
        game_version: Some("1.0.0"),
        screenshot_path: None,
        tags: ["autosave", "chapter_1"],
        custom_data: {
            "location": "forest_entrance",
            "quest_progress": "2/5",
        },
    ),
    scene: SceneDefinition(
        version: 1,
        name: "SavedGame",
        metadata: SceneMetadata(
            description: Some("Auto-generated save file"),
            author: None,
            version: Some("1"),
            tags: ["save"],
        ),
        entities: [
            EntityDefinition(
                name: Some("Player"),
                transform: Some(TransformDef(
                    translation: (0.0, 1.0, 0.0),
                    rotation: (0.0, 0.0, 0.0, 1.0),
                    scale: (1.0, 1.0, 1.0),
                )),
                mesh: Some("character"),
                texture: Some("player_skin"),
                material: None,
                camera: None,
                directional_light: None,
                point_light: None,
                visible: Some(true),
                active: Some(true),
                children: [],
            ),
            // ... more entities
        ],
    ),
)
```

### File Structure

```
SaveFile
├── version: u32                  // Save format version
├── metadata: SaveMetadata        // Rich save metadata
└── scene: SceneDefinition
    ├── version: u32              // Scene format version
    ├── name: String
    ├── metadata: SceneMetadata
    └── entities: Vec<EntityDefinition>
        ├── name: Option<String>
        ├── transform: Option<TransformDef>
        ├── mesh: Option<String>
        ├── texture: Option<String>
        ├── material: Option<String>
        ├── camera: Option<CameraDef>
        ├── directional_light: Option<DirectionalLightDef>
        ├── point_light: Option<PointLightDef>
        ├── visible: Option<bool>
        ├── active: Option<bool>
        └── children: Vec<EntityDefinition>  // Recursive
```

## Versioning and Migration

### Version Detection

```rust
// Save files include version numbers
pub const CURRENT_SAVE_VERSION: u32 = 1;
pub const CURRENT_SCENE_VERSION: u32 = 1;

// On load, versions are checked
if save_file.version < CURRENT_SAVE_VERSION {
    // Migrate save format
    migrate_save(&mut save_file)?;
}

if save_file.scene.version < CURRENT_SCENE_VERSION {
    // Migrate scene format
    migrate_scene(&mut save_file.scene)?;
}
```

### Custom Migration

Implement migration for version changes:

```rust
use praxis_scene::migration;

// Example migration from version 1 to 2
fn migrate_save_v1_to_v2(save_file: &mut SaveFile) -> Result<()> {
    if save_file.version != 1 {
        return Ok(());
    }
    
    // Migrate metadata structure changes
    if save_file.metadata.custom_data.contains_key("old_field") {
        let value = save_file.metadata.custom_data
            .remove("old_field")
            .unwrap();
        save_file.metadata.custom_data.insert("new_field".to_string(), value);
    }
    
    save_file.version = 2;
    Ok(())
}
```

## Multiple Save Slots

### Managing Save Slots

```rust
use std::path::PathBuf;
use std::fs;

struct SaveSlotManager {
    save_dir: PathBuf,
    save_manager: SaveManager,
}

impl SaveSlotManager {
    fn new(save_dir: impl Into<PathBuf>) -> Result<Self> {
        let save_dir = save_dir.into();
        fs::create_dir_all(&save_dir)?;
        
        Ok(Self {
            save_dir,
            save_manager: SaveManager::new(),
        })
    }
    
    fn save_to_slot(&mut self, world: &World, slot: u32, name: String) -> Result<()> {
        let path = self.save_dir.join(format!("slot_{}.ron", slot));
        let metadata = SaveMetadata::new(name)
            .with_playtime(self.get_playtime())
            .with_game_version(env!("CARGO_PKG_VERSION"));
        
        self.save_manager.save_to_file(world, path, metadata)
    }
    
    fn load_from_slot(&mut self, world: &mut World, slot: u32) -> Result<()> {
        let path = self.save_dir.join(format!("slot_{}.ron", slot));
        self.save_manager.load_from_file(world, path)
    }
    
    fn list_saves(&self) -> Result<Vec<(u32, SaveMetadata)>> {
        let mut saves = Vec::new();
        
        for entry in fs::read_dir(&self.save_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                if let Ok(metadata) = self.save_manager.read_metadata(&path) {
                    if let Some(slot) = self.extract_slot_number(&path) {
                        saves.push((slot, metadata));
                    }
                }
            }
        }
        
        saves.sort_by_key(|(slot, _)| *slot);
        Ok(saves)
    }
    
    fn delete_slot(&self, slot: u32) -> Result<()> {
        let path = self.save_dir.join(format!("slot_{}.ron", slot));
        fs::remove_file(path)?;
        Ok(())
    }
    
    fn extract_slot_number(&self, path: &Path) -> Option<u32> {
        path.file_stem()?
            .to_str()?
            .strip_prefix("slot_")?
            .parse()
            .ok()
    }
    
    fn get_playtime(&self) -> u64 {
        // Implement playtime tracking
        0
    }
}
```

### Save Slot UI

```rust
// Example save/load menu
fn render_save_menu(ui: &mut egui::Ui, slot_manager: &SaveSlotManager) {
    ui.heading("Save Game");
    
    let saves = slot_manager.list_saves().unwrap_or_default();
    
    for slot in 0..10 {
        let slot_name = format!("Slot {}", slot + 1);
        
        if let Some((_, metadata)) = saves.iter().find(|(s, _)| *s == slot) {
            ui.group(|ui| {
                ui.label(&metadata.name);
                ui.label(format!("Playtime: {}s", metadata.playtime_seconds));
                ui.label(format!("Date: {}", metadata.timestamp));
                
                if ui.button("Load").clicked() {
                    // Load slot
                }
                
                if ui.button("Overwrite").clicked() {
                    // Save to slot
                }
                
                if ui.button("Delete").clicked() {
                    slot_manager.delete_slot(slot).ok();
                }
            });
        } else {
            if ui.button(&format!("{} (Empty)", slot_name)).clicked() {
                // Save to new slot
            }
        }
    }
}
```

## Auto-Save System

### Checkpoint-Based Auto-Save

```rust
use std::time::{Duration, Instant};

struct AutoSaveSystem {
    save_manager: SaveManager,
    last_autosave: Instant,
    autosave_interval: Duration,
    autosave_path: PathBuf,
}

impl AutoSaveSystem {
    fn new(save_dir: PathBuf) -> Self {
        Self {
            save_manager: SaveManager::new(),
            last_autosave: Instant::now(),
            autosave_interval: Duration::from_secs(300), // 5 minutes
            autosave_path: save_dir.join("autosave.ron"),
        }
    }
    
    fn update(&mut self, world: &World) -> Result<()> {
        if self.last_autosave.elapsed() >= self.autosave_interval {
            self.autosave(world)?;
            self.last_autosave = Instant::now();
        }
        Ok(())
    }
    
    fn autosave(&mut self, world: &World) -> Result<()> {
        let metadata = SaveMetadata::new("Autosave")
            .with_description("Automatic checkpoint")
            .with_tag("autosave");
        
        self.save_manager.save_to_file(world, &self.autosave_path, metadata)?;
        println!("Autosave complete");
        Ok(())
    }
    
    fn trigger_checkpoint(&mut self, world: &World, checkpoint_name: &str) -> Result<()> {
        let metadata = SaveMetadata::new(checkpoint_name)
            .with_description("Checkpoint reached")
            .with_tag("checkpoint")
            .with_tag("autosave");
        
        self.save_manager.save_to_file(world, &self.autosave_path, metadata)?;
        self.last_autosave = Instant::now();
        Ok(())
    }
}
```

### Rotating Auto-Saves

Keep multiple auto-save backups:

```rust
struct RotatingAutoSave {
    save_manager: SaveManager,
    save_dir: PathBuf,
    max_autosaves: usize,
}

impl RotatingAutoSave {
    fn autosave(&mut self, world: &World) -> Result<()> {
        // Rotate existing autosaves
        for i in (1..self.max_autosaves).rev() {
            let old_path = self.save_dir.join(format!("autosave_{}.ron", i));
            let new_path = self.save_dir.join(format!("autosave_{}.ron", i + 1));
            
            if old_path.exists() {
                fs::rename(old_path, new_path)?;
            }
        }
        
        // Delete oldest
        let oldest = self.save_dir.join(format!("autosave_{}.ron", self.max_autosaves));
        if oldest.exists() {
            fs::remove_file(oldest)?;
        }
        
        // Save new autosave
        let path = self.save_dir.join("autosave_1.ron");
        let metadata = SaveMetadata::new("Autosave")
            .with_tag("autosave");
        
        self.save_manager.save_to_file(world, path, metadata)
    }
}
```

## Performance

### Save Performance

Typical performance at various entity counts:

| Entity Count | Components | File Size | Save Time | Load Time |
|-------------|------------|-----------|-----------|-----------|
| 100 | 400 | 25 KB | 2-3 ms | 3-5 ms |
| 1,000 | 4,000 | 250 KB | 15-20 ms | 20-30 ms |
| 10,000 | 40,000 | 2.5 MB | 100-150 ms | 150-200 ms |
| 100,000 | 400,000 | 25 MB | 1-2 s | 2-3 s |

### Optimization Tips

1. **Async Saving**: Save on background thread
   ```rust
   let world_clone = world.clone(); // If possible
   std::thread::spawn(move || {
       save_manager.save_to_file(&world_clone, path, metadata).ok();
   });
   ```

2. **Incremental Saves**: Only save changed entities (requires change tracking)

3. **Compression**: Enable compression for large save files
   ```rust
   let config = SaveConfig {
       compress: true,
       ..Default::default()
   };
   ```

4. **Binary Format**: For maximum performance, consider bincode instead of RON
   (trade-off: not human-readable)

## Common Patterns

### Quick Save/Load

```rust
struct QuickSaveSystem {
    save_manager: SaveManager,
    quicksave_path: PathBuf,
}

impl QuickSaveSystem {
    fn quicksave(&mut self, world: &World) -> Result<()> {
        let metadata = SaveMetadata::new("Quick Save");
        self.save_manager.save_to_file(world, &self.quicksave_path, metadata)
    }
    
    fn quickload(&mut self, world: &mut World) -> Result<()> {
        self.save_manager.load_from_file(world, &self.quicksave_path)
    }
}
```

### Save Corruption Detection

```rust
fn verify_save_integrity(path: &Path) -> Result<bool> {
    let save_manager = SaveManager::new();
    
    // Try to read metadata
    match save_manager.read_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) => {
            eprintln!("Save file corrupted: {}", e);
            Ok(false)
        }
    }
}
```

### Cloud Save Integration

```rust
async fn sync_save_to_cloud(
    local_path: &Path,
    cloud_service: &CloudStorage,
) -> Result<()> {
    let save_data = fs::read(local_path)?;
    cloud_service.upload("savegame.ron", save_data).await?;
    Ok(())
}

async fn download_save_from_cloud(
    cloud_service: &CloudStorage,
    local_path: &Path,
) -> Result<()> {
    let save_data = cloud_service.download("savegame.ron").await?;
    fs::write(local_path, save_data)?;
    Ok(())
}
```

## See Also

- [Scene Format Reference](../reference/scene-format.md) - Scene format specification
- [ECS Architecture](../concepts/ecs-architecture.md) - Entity-Component-System fundamentals
- [Assets Guide](assets/README.md) - Asset loading and management
- [praxis_scene Crate](../../crates/praxis_scene/README.md) - Crate documentation

## Examples

Run the serialization examples:

```bash
cargo run --example save_load_demo
cargo run --example scene_serialization_demo
```
