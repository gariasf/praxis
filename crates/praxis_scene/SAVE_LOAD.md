# Save/Load System

Comprehensive game state persistence system for the Praxis engine.

## Overview

The save/load system provides full game state persistence, allowing you to save and restore complete world states including:

- All entities and their components
- Entity hierarchies (parent-child relationships)
- Asset references (meshes, textures, materials)
- Scene metadata with versioning
- Custom save metadata (timestamps, playtime, descriptions, tags)

## Features

### Core Capabilities

- **Full State Capture**: Serializes all entities, components, and hierarchies
- **Selective Persistence**: Entities marked with `NoSave` are automatically excluded
- **Asset References**: Properly tracks mesh, texture, and material handles
- **Versioning**: Built-in version management for save format migration
- **Rich Metadata**: Timestamps, playtime tracking, descriptions, tags, and custom data
- **Multiple Save Slots**: Easy management of multiple save files
- **Fast Operations**: Optimized for quick save/load cycles
- **Validation**: Optional save file validation after writing

### Supported Components

The save system currently serializes the following components:

- `Name` - Entity identification
- `Transform` - Position, rotation, scale
- `GlobalTransform` - World-space transforms
- `MeshHandle` - Mesh asset references
- `TextureHandle` - Texture asset references  
- `MaterialHandle` - Material asset references
- `Camera` - Camera configuration
- `PerspectiveProjection` / `OrthographicProjection` - Camera projection
- `DirectionalLight` - Directional lighting
- `PointLight` - Point lighting
- `Visibility` - Visibility state
- `Active` - Active/enabled state
- `Parent` / `Children` - Entity hierarchy

## Usage

### Basic Save/Load

```rust
use praxis_scene::{SaveManager, SaveMetadata};
use praxis_ecs::World;

// Create save manager
let mut save_manager = SaveManager::new();

// Save the world
let metadata = SaveMetadata::new("My Save");
save_manager.save_to_file(&world, "saves/slot1.ron", metadata)?;

// Load the world
save_manager.load_from_file(&mut world, "saves/slot1.ron")?;
```

### Rich Metadata

```rust
let metadata = SaveMetadata::new("Chapter 3 - The Castle")
    .with_description("Player at castle entrance")
    .with_playtime(7200) // 2 hours in seconds
    .with_game_version("1.0.0")
    .with_tag("autosave")
    .with_tag("chapter_3")
    .with_custom_data("location", "castle_entrance")
    .with_custom_data("quest_stage", "3");

save_manager.save_to_file(&world, "saves/autosave.ron", metadata)?;
```

### Reading Metadata Without Loading

Useful for displaying save file information in a menu:

```rust
// Read metadata without loading the entire save
let metadata = save_manager.read_metadata("saves/slot1.ron")?;

println!("Save: {}", metadata.name);
println!("Created: {}", metadata.timestamp);
println!("Playtime: {} seconds", metadata.playtime_seconds);

if let Some(desc) = &metadata.description {
    println!("Description: {}", desc);
}
```

### Configuration

```rust
use praxis_scene::SaveConfig;

let config = SaveConfig {
    compress: false,              // Compression (future feature)
    include_editor_data: false,   // Include editor-specific data
    validate_after_save: true,    // Validate saves after writing
    pretty_print: true,           // Human-readable RON format
};

let mut save_manager = SaveManager::with_config(config);
```

### Save Statistics

```rust
save_manager.save_to_file(&world, "saves/slot1.ron", metadata)?;

if let Some(stats) = save_manager.last_stats() {
    println!("Entities: {}", stats.entity_count);
    println!("Components: {}", stats.component_count);
    println!("Duration: {:.2}ms", stats.duration_ms);
    if let Some(size) = stats.file_size_bytes {
        println!("File size: {} bytes", size);
    }
}
```

### Excluding Entities from Saves

Use the `NoSave` component to mark temporary entities:

```rust
use praxis_ecs::NoSave;

// This entity won't be saved
world.spawn((
    Name("DebugMarker".to_string()),
    Transform::default(),
    NoSave,
));
```

### Multiple Save Slots

```rust
// Save slot 1
let metadata1 = SaveMetadata::new("Manual Save 1")
    .with_description("Before boss fight");
save_manager.save_to_file(&world, "saves/slot1.ron", metadata1)?;

// Save slot 2
let metadata2 = SaveMetadata::new("Manual Save 2")
    .with_description("After boss fight");
save_manager.save_to_file(&world, "saves/slot2.ron", metadata2)?;

// Autosave
let metadata_auto = SaveMetadata::new("Autosave")
    .with_tag("autosave");
save_manager.save_to_file(&world, "saves/autosave.ron", metadata_auto)?;
```

## Save File Format

Saves are stored in RON (Rusty Object Notation) format, which is both human-readable and efficient:

```ron
(
    version: 1,
    metadata: (
        name: "Chapter 1 - Forest",
        description: Some("Player at forest entrance"),
        timestamp: "2024-01-15T10:30:00Z",
        playtime_seconds: 3600,
        game_version: Some("1.0.0"),
        tags: ["autosave", "chapter_1"],
        custom_data: {
            "location": "forest_entrance",
            "quest_progress": "2/5",
        },
    ),
    scene: (
        version: 2,
        name: "SavedGame",
        entities: [
            (
                name: Some("Player"),
                transform: Some((
                    translation: (0.0, 1.0, 0.0),
                    rotation: (0.0, 0.0, 0.0, 1.0),
                    scale: (1.0, 1.0, 1.0),
                )),
                mesh: Some("character"),
                texture: Some("player_skin"),
                visible: Some(true),
                active: Some(true),
                children: [],
            ),
            // ... more entities
        ],
    ),
)
```

## Versioning and Migration

The save system includes version tracking for both save files and scene formats:

- **Save Version**: Tracks the save file structure version
- **Scene Version**: Tracks the scene definition version

When loading an older save, the system automatically migrates it to the current version:

```rust
// Automatic migration on load
save_manager.load_from_file(&mut world, "old_save.ron")?;
// Older saves are automatically migrated
```

### Current Versions

- **Save Format Version**: 1
- **Scene Format Version**: 2 (with physics, audio, animation support)

## Best Practices

### 1. Regular Autosaves

Implement periodic autosaves to prevent progress loss:

```rust
// Autosave every 5 minutes
if game_time.elapsed_seconds() > last_autosave_time + 300.0 {
    let metadata = SaveMetadata::new("Autosave")
        .with_playtime(game_time.total_seconds() as u64)
        .with_tag("autosave");
    
    save_manager.save_to_file(&world, "saves/autosave.ron", metadata)?;
    last_autosave_time = game_time.elapsed_seconds();
}
```

### 2. Multiple Save Slots

Provide multiple manual save slots for players:

```rust
fn save_to_slot(slot_number: u32, save_manager: &mut SaveManager, world: &World) -> Result<()> {
    let metadata = SaveMetadata::new(format!("Save Slot {}", slot_number))
        .with_playtime(get_total_playtime())
        .with_tag("manual");
    
    save_manager.save_to_file(
        world,
        format!("saves/slot{}.ron", slot_number),
        metadata,
    )?;
    
    Ok(())
}
```

### 3. Save File Listing

Display available saves to players:

```rust
fn list_saves(save_manager: &SaveManager) -> Result<Vec<SaveMetadata>> {
    let mut saves = Vec::new();
    
    for entry in std::fs::read_dir("saves")? {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("ron") {
            if let Ok(metadata) = save_manager.read_metadata(entry.path()) {
                saves.push(metadata);
            }
        }
    }
    
    // Sort by timestamp
    saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    Ok(saves)
}
```

### 4. Error Handling

Always handle save/load errors gracefully:

```rust
match save_manager.save_to_file(&world, path, metadata) {
    Ok(_) => println!("Game saved successfully"),
    Err(e) => {
        eprintln!("Failed to save game: {}", e);
        // Show error message to player
        // Maybe retry or save to alternate location
    }
}
```

### 5. Mark Temporary Entities

Use `NoSave` for entities that shouldn't persist:

```rust
// Debug visualizations
world.spawn((DebugLines::default(), NoSave));

// Particle effects
world.spawn((ParticleEmitter::new("explosion"), NoSave));

// UI elements
world.spawn((MenuPanel::default(), NoSave));

// Temporary markers
world.spawn((Waypoint::default(), NoSave));
```

## Performance Considerations

### Save Performance

- **Typical save time**: 1-10ms for 100-1000 entities
- **File size**: ~1-5 KB per entity with components
- **Optimization**: Consider async saves for large worlds

### Load Performance

- **Typical load time**: 2-15ms for 100-1000 entities
- **Memory**: Loads entire file into memory during deserialization
- **World clearing**: Old world is cleared before loading

### Tips for Large Worlds

1. **Exclude temporary entities**: Use `NoSave` liberally
2. **Separate persistent and transient data**: Don't save procedural/regenerable data
3. **Async saves**: Implement background saving for large states
4. **Compression**: Enable compression (when available) for large saves

## Example: Complete Save/Load Menu

```rust
use praxis_scene::{SaveManager, SaveMetadata};
use praxis_ecs::World;

struct SaveMenu {
    save_manager: SaveManager,
    save_slots: Vec<Option<SaveMetadata>>,
}

impl SaveMenu {
    fn new() -> Self {
        Self {
            save_manager: SaveManager::new(),
            save_slots: vec![None; 3],
        }
    }
    
    fn load_slot_metadata(&mut self, slot: usize) -> Result<()> {
        let path = format!("saves/slot{}.ron", slot + 1);
        if std::path::Path::new(&path).exists() {
            self.save_slots[slot] = Some(self.save_manager.read_metadata(&path)?);
        }
        Ok(())
    }
    
    fn save_game(&mut self, world: &World, slot: usize) -> Result<()> {
        let metadata = SaveMetadata::new(format!("Save Slot {}", slot + 1))
            .with_playtime(get_playtime())
            .with_tag("manual");
        
        let path = format!("saves/slot{}.ron", slot + 1);
        self.save_manager.save_to_file(world, &path, metadata)?;
        self.load_slot_metadata(slot)?;
        
        Ok(())
    }
    
    fn load_game(&mut self, world: &mut World, slot: usize) -> Result<()> {
        let path = format!("saves/slot{}.ron", slot + 1);
        self.save_manager.load_from_file(world, &path)?;
        Ok(())
    }
    
    fn display_saves(&self) {
        for (i, metadata) in self.save_slots.iter().enumerate() {
            match metadata {
                Some(meta) => {
                    println!("Slot {}: {} - {} ({}s)",
                        i + 1,
                        meta.name,
                        meta.timestamp,
                        meta.playtime_seconds);
                }
                None => {
                    println!("Slot {}: Empty", i + 1);
                }
            }
        }
    }
}

fn get_playtime() -> u64 {
    // Return current playtime in seconds
    0
}
```

## See Also

- `examples/save_load_demo.rs` - Complete working example
- `SceneManager` - For runtime scene management
- `SceneLoader` - For loading static scene definitions
- Scene serialization documentation in `SCENE_SERIALIZATION.md`
