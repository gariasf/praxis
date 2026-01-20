# Scene Serialization Enhancement Changelog

## Summary

This update implements a comprehensive scene serialization system with versioning, migration, editor-only data support, and validation.

## New Features

### 1. Versioning System

- **`CURRENT_SCENE_VERSION`** constant tracks the current scene format version (currently version 1)
- **`version` field** added to `SceneDefinition` for format tracking
- Default version for new scenes is automatically set to current version
- Version 0 represents legacy format without version field

### 2. Migration System (`migration.rs`)

- **`migrate_scene()`** - Automatically migrates scenes from older versions
- **Sequential migration path** - Applies migrations one version at a time
- **`migrate_to_v1()`** - Migrates version 0 to version 1
- Extensible system for future format changes
- Logs migration progress for debugging

### 3. Validation System

- **`validate_scene()`** - Validates complete scene structure
- **Entity validation** - Checks components, cameras, lights
- **Hierarchy validation** - Recursively validates child entities
- **Editor data validation** - Validates camera settings, viewport, preferences
- **Detailed error messages** - Shows path to invalid data

### 4. Editor-Only Data

New types for preserving complete editor state:

#### `EditorData`
- Container for all editor-specific data
- Skipped in serialization if not present
- Not used at runtime

#### `EditorCamera`
- Editor camera position and orientation
- Camera modes: Orbit, Free, Fly
- FOV, near/far clipping planes
- Camera control parameters (distance, pitch, yaw)

#### `ViewportSettings`
- Visual display toggles (grid, gizmos, wireframe, bounds, lights, cameras)
- Grid configuration (size, spacing)
- Background color
- Active gizmo mode (Translate, Rotate, Scale)

#### `EditorPreferences`
- Auto-save configuration
- Grid snapping settings
- Rotation snapping
- Last used asset paths
- Collapsed hierarchy nodes

### 5. Enhanced `SceneDefinition` API

New methods:
- `set_editor_data()` - Sets editor data
- `editor_data()` - Gets reference to editor data
- `editor_data_mut()` - Gets mutable reference
- `clear_editor_data()` - Removes editor data
- `has_editor_data()` - Checks if editor data present
- `to_runtime_scene()` - Creates runtime copy without editor data

### 6. Automatic Migration in `SceneLoader`

- `load_from_file()` and `load_from_string()` now automatically migrate and validate
- Transparent to users - scenes are always loaded at current version
- Validation errors provide detailed feedback

### 7. Builder Pattern for Editor Data

Fluent API for creating editor data:
```rust
EditorData::new()
    .with_camera(camera)
    .with_selected_entities(entities)
    .with_viewport(viewport)
    .with_preferences(preferences)
```

## Files Added

1. **`crates/praxis_scene/src/migration.rs`**
   - Migration functions
   - Validation functions
   - Comprehensive tests

2. **`crates/praxis_scene/SCENE_SERIALIZATION.md`**
   - Complete documentation
   - API usage examples
   - Best practices
   - Troubleshooting guide

3. **`examples/scene_serialization_demo.rs`**
   - Demonstrates all features
   - Shows editor data usage
   - Runtime scene creation

4. **`crates/praxis_scene/CHANGELOG_SERIALIZATION.md`** (this file)

## Files Modified

1. **`crates/praxis_scene/src/definition.rs`**
   - Added version field to `SceneDefinition`
   - Added `editor_data` field
   - Added `EditorData` type and related types
   - Added new helper methods
   - Added comprehensive tests

2. **`crates/praxis_scene/src/loader.rs`**
   - Updated to call migration and validation
   - Enhanced documentation
   - Added tests for new features

3. **`crates/praxis_scene/src/lib.rs`**
   - Exported migration module
   - Exported new types

## Breaking Changes

**None** - This is fully backwards compatible:
- Old scenes (version 0) are automatically migrated to version 1
- New fields use `Option` types with serde defaults
- Existing API unchanged, only additions

## Migration Path

For developers maintaining old scenes:

1. **No action required** - Old scenes load automatically
2. Migration happens transparently on load
3. Save migrated scenes to update them permanently
4. Validation provides early error detection

## Usage Example

```rust
use praxis_scene::{
    SceneDefinition, SceneLoader, EditorData, EditorCamera,
    ViewportSettings, EditorPreferences,
};

// Create scene with editor data
let mut scene = SceneDefinition::new("My Scene");

let editor_data = EditorData::new()
    .with_camera(EditorCamera::new())
    .with_viewport(ViewportSettings::new())
    .with_preferences(EditorPreferences::new());

scene.set_editor_data(editor_data);

// Save (includes editor data)
let loader = SceneLoader::new();
loader.save_to_file(&scene, "scene.ron")?;

// Load (automatic migration & validation)
let loaded = loader.load_from_file("scene.ron")?;

// Create runtime version (no editor data)
let runtime = loaded.to_runtime_scene();
```

## Testing

All new functionality is fully tested:
- 15+ new tests in `migration.rs`
- 10+ new tests in `definition.rs`
- 7+ new tests in `loader.rs`
- All existing tests still pass

## Future Work

Potential enhancements:
- Binary serialization format for faster loading
- Scene references for modular composition
- Prefab system for reusable templates
- Asset dependency tracking
- Compression for large scenes
- Incremental loading for open worlds

## Version History

- **Version 1** (Current) - Initial versioned format with editor data support
- **Version 0** (Legacy) - Original format without version field
