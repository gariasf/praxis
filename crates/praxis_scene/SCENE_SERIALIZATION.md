# Scene Serialization System

This document describes the complete scene serialization system in Praxis, including versioning, migration, editor-only data, and best practices.

## Overview

The scene serialization system provides:

- **RON-based format** for human-readable scene files
- **Versioning system** to track format changes
- **Automatic migration** from older versions
- **Validation** to catch errors early
- **Editor-only data** preserved separately from runtime data
- **Complete state preservation** for editor workflows

## Scene Format

Scenes are serialized in RON (Rusty Object Notation) format, which is human-readable and easy to edit by hand.

### Basic Scene Structure

```ron
(
    version: 1,
    name: "My Scene",
    entities: [
        // Entity definitions...
    ],
    metadata: (
        description: Some("Scene description"),
        author: Some("Author Name"),
        version: Some("1.0.0"),
        tags: ["level", "gameplay"],
    ),
    editor_data: Some((
        camera: Some(/* editor camera state */),
        selected_entities: ["Entity1"],
        viewport: Some(/* viewport settings */),
        preferences: Some(/* editor preferences */),
    )),
)
```

### Version Field

The `version` field tracks the scene format version:

- Current version: `1`
- Default value for new scenes: `1`
- Used for automatic migration when loading older scenes

### Entities

Entities support all standard components:

- Transform (position, rotation, scale)
- Mesh and texture handles
- Cameras (perspective/orthographic)
- Lights (directional/point)
- Visibility and active state
- Hierarchical parent-child relationships

### Metadata

Optional metadata for organization and documentation:

- `description`: Human-readable description
- `author`: Scene creator
- `version`: Scene version (separate from format version)
- `tags`: Categorization tags

## Editor-Only Data

Editor-only data is preserved in scenes but not used at runtime. This allows the editor to restore complete state when reopening a scene.

### Editor Camera

Stores the editor's scene view camera state:

```rust
EditorCamera {
    position: (0.0, 5.0, 10.0),          // World position
    target: (0.0, 0.0, 0.0),             // Look-at target
    distance: 10.0,                       // Distance from target
    pitch: -0.523599,                     // Up/down angle (radians)
    yaw: 0.785398,                        // Left/right angle (radians)
    fov: 60.0,                            // Field of view (degrees)
    near_clip: 0.1,                       // Near clipping plane
    far_clip: 1000.0,                     // Far clipping plane
    mode: Orbit,                          // Camera control mode
}
```

Camera modes:
- **Orbit**: Camera orbits around a target point
- **Free**: Camera moves freely with WASD/QE controls
- **Fly**: Camera flies with smooth acceleration

### Selected Entities

List of entity names that were selected when the scene was saved:

```rust
selected_entities: ["Player", "MainCamera", "DirectionalLight"]
```

This allows the editor to restore selection state when reopening the scene.

### Viewport Settings

Visual settings for the editor viewport:

```rust
ViewportSettings {
    show_grid: true,                      // Show grid floor
    show_gizmos: true,                    // Show manipulation gizmos
    show_wireframe: false,                // Show wireframe overlay
    show_bounds: false,                   // Show bounding boxes
    show_lights: true,                    // Show light visualizations
    show_cameras: false,                  // Show camera frustums
    grid_size: 20,                        // Number of grid lines
    grid_spacing: 1.0,                    // Distance between lines
    background_color: (0.118, 0.118, 0.137), // RGB background
    gizmo_mode: Translate,                // Active gizmo mode
}
```

Gizmo modes:
- **Translate**: Move entities
- **Rotate**: Rotate entities
- **Scale**: Scale entities

### Editor Preferences

Scene-specific editor preferences:

```rust
EditorPreferences {
    auto_save_enabled: true,              // Enable auto-save
    auto_save_interval: 300.0,            // Auto-save every 5 minutes
    snap_to_grid: false,                  // Snap transforms to grid
    snap_size: 1.0,                       // Grid snap size
    rotation_snap: 15.0,                  // Rotation snap angle (degrees)
    last_asset_path: Some("assets/"),     // Last used asset path
    collapsed_hierarchy_nodes: ["Environment"], // Collapsed tree nodes
}
```

## Versioning and Migration

### Version History

**Version 1** (Current):
- Initial versioned format
- Added `version` field
- Added `editor_data` support
- All entity and component features

**Version 0** (Legacy):
- Original format without version field
- Automatically migrated to version 1 on load

### Migration Process

When loading a scene, the system:

1. **Parse** the RON file
2. **Check version** against current version
3. **Apply migrations** sequentially if needed
4. **Validate** the resulting scene
5. **Return** the migrated and validated scene

Migration is automatic and transparent to the user.

### Adding New Versions

When making backwards-incompatible changes:

1. Increment `CURRENT_SCENE_VERSION` in `definition.rs`
2. Add migration function in `migration.rs`:
   ```rust
   fn migrate_to_vN(scene: &mut SceneDefinition) -> Result<()> {
       // Transform old data to new format
       Ok(())
   }
   ```
3. Add case to migration match statement
4. Test migration with old scene files

## Validation

The validation system checks for:

### Scene-Level Validation

- Scene name is not empty
- Version is recognized
- All entities are valid

### Entity Validation

- Cameras have valid near/far planes (near < far)
- Camera FOV is in valid range (0 to π radians)
- Entities have at least one component or children
- Child entities are valid recursively

### Editor Data Validation

- Editor camera has valid near/far clips
- Editor camera FOV is in valid range (0 to 180 degrees)
- Editor camera distance is non-negative
- Viewport grid size is non-zero
- Viewport grid spacing is positive
- Viewport background color components are in [0, 1]
- Auto-save interval is non-negative
- Snap sizes are positive
- Rotation snap is in valid range

## API Usage

### Loading Scenes

```rust
use praxis_scene::{SceneLoader, SceneDefinition};

let loader = SceneLoader::new();

// Load from file (automatic migration and validation)
let scene = loader.load_from_file("assets/scenes/level1.ron")?;

// Load from string
let ron_string = std::fs::read_to_string("scene.ron")?;
let scene = loader.load_from_string(&ron_string)?;

// Access scene data
println!("Loaded scene: {}", scene.name);
println!("Version: {}", scene.version);
println!("Entities: {}", scene.entity_count());

// Check for editor data
if scene.has_editor_data() {
    let editor = scene.editor_data().unwrap();
    if let Some(camera) = &editor.camera {
        println!("Editor camera at {:?}", camera.position);
    }
}
```

### Saving Scenes

```rust
use praxis_scene::{SceneLoader, SceneDefinition, EditorData, EditorCamera};

let mut scene = SceneDefinition::new("My Scene");

// Add entities...
scene.add_entity(entity);

// Add editor data
let editor_data = EditorData::new()
    .with_camera(EditorCamera::new())
    .with_viewport(ViewportSettings::new());
scene.set_editor_data(editor_data);

// Save to file
let loader = SceneLoader::new();
loader.save_to_file(&scene, "assets/scenes/my_scene.ron")?;

// Save to string
let ron_string = loader.save_to_string(&scene)?;
```

### Working with Editor Data

```rust
// Create editor data
let mut editor_data = EditorData::new();

// Set editor camera
let camera = EditorCamera::orbit(
    (0.0, 0.0, 0.0),  // Target
    15.0,              // Distance
    -0.5,              // Pitch
    1.2,               // Yaw
);
editor_data.camera = Some(camera);

// Set selected entities
editor_data.selected_entities = vec![
    "Player".to_string(),
    "MainCamera".to_string(),
];

// Set viewport settings
let mut viewport = ViewportSettings::new();
viewport.show_grid = true;
viewport.gizmo_mode = GizmoMode::Rotate;
editor_data.viewport = Some(viewport);

// Add to scene
scene.set_editor_data(editor_data);
```

### Creating Runtime Scenes

To create a scene without editor data (for runtime use):

```rust
// Remove editor data from existing scene
scene.clear_editor_data();

// Or create a runtime copy
let runtime_scene = scene.to_runtime_scene();
```

### Manual Migration

```rust
use praxis_scene::{migrate_scene, validate_scene};

// Load without automatic migration
let mut scene: SceneDefinition = ron::from_str(&ron_string)?;

// Manually migrate
let migrated = migrate_scene(&mut scene)?;
if migrated {
    println!("Scene was migrated to version {}", scene.version);
}

// Manually validate
validate_scene(&scene)?;
```

## Best Practices

### For Editor Developers

1. **Always save editor data** when saving scenes from the editor
2. **Restore editor state** from editor data when loading scenes
3. **Update selected entities list** when selection changes
4. **Save viewport settings** on scene save or editor exit
5. **Use preferences** for scene-specific editor settings

### For Game Developers

1. **Use runtime scenes** for shipped games (without editor data)
2. **Test migration** when upgrading scene format versions
3. **Validate scenes** in build pipeline to catch errors early
4. **Keep version up to date** when saving scenes programmatically
5. **Document metadata** for team collaboration

### Scene Organization

1. **Use meaningful names** for scenes and entities
2. **Add descriptions** in metadata for complex scenes
3. **Use tags** to categorize scenes (e.g., "level", "menu", "test")
4. **Maintain version** in metadata for game content versioning
5. **Organize hierarchies** logically for easier editing

## Examples

### Complete Scene Example

```ron
(
    version: 1,
    name: "Level 1",
    entities: [
        (
            name: Some("Player"),
            transform: Some((
                translation: (0.0, 1.0, 0.0),
                rotation: (0.0, 0.0, 0.0, 1.0),
                scale: (1.0, 1.0, 1.0),
            )),
            mesh: Some("character_mesh"),
            texture: Some("character_texture"),
            visible: Some(true),
            active: Some(true),
            children: [],
        ),
        (
            name: Some("MainCamera"),
            transform: Some((
                translation: (0.0, 5.0, 10.0),
                rotation: (0.0, 0.0, 0.0, 1.0),
                scale: (1.0, 1.0, 1.0),
            )),
            camera: Some((
                camera_type: Perspective,
                fov: Some(1.0472),
                aspect_ratio: Some(1.77778),
                near: 0.1,
                far: 1000.0,
                is_active: true,
                priority: 0,
            )),
            children: [],
        ),
        (
            name: Some("DirectionalLight"),
            directional_light: Some((
                direction: (0.5, -1.0, 0.3),
                color: (1.0, 0.95, 0.9),
                intensity: 1.5,
            )),
            children: [],
        ),
    ],
    metadata: (
        description: Some("First level of the game"),
        author: Some("Game Designer"),
        version: Some("1.0.0"),
        tags: ["level", "gameplay"],
    ),
    editor_data: Some((
        camera: Some((
            position: (10.0, 8.0, 15.0),
            target: (0.0, 1.0, 0.0),
            distance: 18.0,
            pitch: -0.4,
            yaw: 0.8,
            fov: 60.0,
            near_clip: 0.1,
            far_clip: 1000.0,
            mode: Orbit,
        )),
        selected_entities: ["Player"],
        viewport: Some((
            show_grid: true,
            show_gizmos: true,
            show_wireframe: false,
            show_bounds: false,
            show_lights: true,
            show_cameras: false,
            grid_size: 20,
            grid_spacing: 1.0,
            background_color: (0.118, 0.118, 0.137),
            gizmo_mode: Translate,
        )),
        preferences: Some((
            auto_save_enabled: true,
            auto_save_interval: 300.0,
            snap_to_grid: false,
            snap_size: 1.0,
            rotation_snap: 15.0,
            last_asset_path: Some("assets/levels/"),
            collapsed_hierarchy_nodes: [],
        )),
    )),
)
```

## Troubleshooting

### Scene Won't Load

**Problem**: `Failed to parse scene RON` error

**Solution**: 
- Check RON syntax (missing commas, brackets, etc.)
- Ensure all required fields are present
- Run through RON validator

### Validation Fails

**Problem**: Scene loads but validation fails

**Solution**:
- Check error message for specific field
- Verify camera near/far values
- Ensure all numeric values are in valid ranges
- Check entity component consistency

### Migration Issues

**Problem**: Old scene doesn't migrate properly

**Solution**:
- Check scene version field
- Verify migration path exists
- Look at migration logs for details
- Test migration with test scenes

### Editor State Not Restored

**Problem**: Editor doesn't restore camera/selection

**Solution**:
- Ensure editor_data is being saved
- Check selected_entities list format
- Verify camera mode and parameters
- Confirm viewport settings are present

## Future Enhancements

Potential future improvements:

- **Incremental loading** for large scenes
- **Streaming support** for open-world games
- **Compression** for scene files
- **Binary format** option for faster loading
- **Scene references** for modular scene composition
- **Prefab system** for reusable entity templates
- **Asset dependency tracking** for packaging
- **Undo/redo serialization** for editor history
