# Scene System Implementation

This document describes the complete scene management system implementation for Praxis.

## Components Implemented

### 1. Scene Components (`components.rs`)

- **`Scene`**: Component marking entities as belonging to a specific scene
- **`SceneHandle`**: Unique identifier for loaded scene instances
  - Auto-generates unique IDs using atomic counter
  - Supports custom IDs via `new()` constructor

### 2. Scene Definitions (`definition.rs`)

Core structures for defining scenes in RON format:

- **`SceneDefinition`**: Top-level scene structure with name, entities, and metadata
  - Methods: `new()`, `add_entity()`, `entity_count()`, `total_entity_count()`
  
- **`SceneMetadata`**: Optional scene metadata (description, author, version, tags)

- **`EntityDefinition`**: Defines an entity with all its components
  - Builder methods: `with_name()`, `with_transform()`, `with_mesh()`, `with_child()`
  - Helper constructors for common entity types:
    - `perspective_camera()`: Creates a perspective camera
    - `orthographic_camera()`: Creates an orthographic camera
    - `directional_light()`: Creates a directional light (sun)
    - `point_light()`: Creates a point light
    - `mesh_entity()`: Creates a mesh entity
    - `textured_mesh_entity()`: Creates a textured mesh entity

- **`TransformDef`**: Transform data (translation, rotation, scale) as tuples
  - Methods: `identity()`, `from_translation()`, `to_components()`

- **`CameraDef`**: Camera configuration (perspective or orthographic)
  - Methods: `perspective()`, `orthographic()`

- **`CameraType`**: Enum for camera types (Perspective, Orthographic)

- **`DirectionalLightDef`**: Directional light configuration

- **`PointLightDef`**: Point light configuration

All structures support Serde serialization/deserialization for RON format.

### 3. Scene Loader (`loader.rs`)

**`SceneLoader`**: Loads and saves scene definitions from/to RON files

- `new()`: Creates a new loader
- `with_base_path()`: Creates a loader with a base path for relative file resolution
- `load_from_file()`: Loads a scene from a RON file
- `load_from_string()`: Parses a scene from a RON string
- `save_to_file()`: Saves a scene definition to a RON file
- `save_to_string()`: Serializes a scene to a RON string with pretty formatting
- `set_base_path()`: Updates the base path
- `base_path()`: Gets the current base path

Includes comprehensive tests for loading, saving, and roundtrip serialization.

### 4. Scene Manager (`manager.rs`)

**`SceneManager`**: Manages loaded scene instances and spawns entities into the ECS world

- `new()`: Creates a new scene manager
- `spawn_scene()`: Spawns all entities from a scene definition into the world
  - Returns a `SceneHandle` for later reference
  - Recursively spawns hierarchical entities with proper Parent/Children relationships
  - Tags all spawned entities with the `Scene` component
  - Automatically adds appropriate ECS components based on definition:
    - Transform, GlobalTransform
    - Name
    - MeshHandle, TextureHandle
    - Camera, PerspectiveProjection, OrthographicProjection, CameraMatrices
    - DirectionalLight, PointLight
    - Visibility, Active
    - Parent/Children for hierarchy

- `unload_scene()`: Removes all entities belonging to a scene
  - Recursively despawns all children
  - Returns `true` if scene was found and unloaded

- `unload_all()`: Unloads all currently loaded scenes

- `is_scene_loaded()`: Checks if a scene is currently loaded
- `loaded_scene_count()`: Gets the number of loaded scenes
- `get_scene_entities()`: Gets the root entities for a scene

Includes tests for spawning, unloading, hierarchies, and multiple scenes.

### 5. Scene Graph Traversal (`traversal.rs`)

Utilities for traversing and querying the scene graph:

**`SceneGraphIterator`**: Iterator for traversing scene hierarchies
- Supports depth-first and breadth-first traversal
- `new(world, root, order)`: Creates an iterator from a root entity

**`TraversalOrder`**: Enum for traversal strategies
- `DepthFirst`: Visit parent before children
- `BreadthFirst`: Visit all siblings before descendants

**Query Functions**:
- `get_root_entities()`: Gets all entities without parents
- `get_all_children()`: Gets all descendants of an entity recursively
- `get_parent_chain()`: Gets the chain of parents up to the root
- `get_root_entity()`: Gets the root entity for any entity
- `is_ancestor_of()`: Checks if one entity is an ancestor of another
- `get_entity_depth()`: Gets the depth of an entity in the hierarchy (0 = root)
- `find_entities_by_name()`: Finds all entities with a given name
- `find_entity_by_name()`: Finds the first entity with a given name

All functions are fully tested with comprehensive unit tests.

## Supported Entity Components

The scene system can spawn entities with the following components from `praxis_ecs`:

1. **Transform**: Position, rotation, scale
2. **GlobalTransform**: World-space transform matrix
3. **Name**: Entity name for debugging/queries
4. **MeshHandle**: Reference to mesh asset
5. **TextureHandle**: Reference to texture asset
6. **Camera**: Camera marker with active state and priority
7. **PerspectiveProjection**: Perspective camera settings
8. **OrthographicProjection**: Orthographic camera settings
9. **CameraMatrices**: Computed camera matrices (view, projection)
10. **DirectionalLight**: Directional light (e.g., sun)
11. **PointLight**: Point light with position and range
12. **Visibility**: Visible/Hidden state
13. **Active**: Active marker component
14. **Parent**: Parent entity reference for hierarchy
15. **Children**: Child entities list (auto-managed)
16. **Scene**: Scene tag with handle reference

## RON Format Example

```ron
(
    name: "Example Scene",
    entities: [
        (
            name: Some("MainCamera"),
            transform: Some((
                translation: (0.0, 5.0, 10.0),
                rotation: (0.0, 0.0, 0.0, 1.0),
                scale: (1.0, 1.0, 1.0),
            )),
            camera: Some((
                camera_type: Perspective,
                fov: Some(1.22173),
                aspect_ratio: Some(1.77778),
                near: 0.1,
                far: 1000.0,
                is_active: true,
                priority: 0,
            )),
            children: [],
        ),
        (
            name: Some("Player"),
            transform: Some((
                translation: (0.0, 1.0, 0.0),
                rotation: (0.0, 0.0, 0.0, 1.0),
                scale: (1.0, 1.0, 1.0),
            )),
            mesh: Some("cube"),
            children: [
                (
                    name: Some("PlayerLight"),
                    transform: Some((
                        translation: (0.0, 0.5, 0.0),
                        rotation: (0.0, 0.0, 0.0, 1.0),
                        scale: (1.0, 1.0, 1.0),
                    )),
                    point_light: Some((
                        color: (1.0, 0.8, 0.6),
                        intensity: 5.0,
                        range: 10.0,
                    )),
                    children: [],
                ),
            ],
        ),
    ],
    metadata: (
        description: Some("An example scene"),
        author: Some("Your Name"),
        version: Some("1.0.0"),
        tags: ["example", "demo"],
    ),
)
```

## Files Created

### Source Files
- `crates/praxis_scene/src/components.rs` - Scene component types
- `crates/praxis_scene/src/definition.rs` - Scene definition structures
- `crates/praxis_scene/src/loader.rs` - Scene loading/saving
- `crates/praxis_scene/src/manager.rs` - Scene spawning/management
- `crates/praxis_scene/src/traversal.rs` - Scene graph utilities
- `crates/praxis_scene/src/lib.rs` - Public API and documentation

### Configuration
- `crates/praxis_scene/Cargo.toml` - Dependencies (serde, ron, praxis_ecs, praxis_math, praxis_utils)

### Examples & Assets
- `examples/scene_demo.rs` - Comprehensive demonstration of the scene system
- `assets/scenes/example_scene.ron` - Complex example scene with camera, lights, hierarchy
- `assets/scenes/simple_scene.ron` - Minimal example scene

### Documentation
- `crates/praxis_scene/README.md` - User-facing documentation
- `crates/praxis_scene/IMPLEMENTATION.md` - This file

### Integration
- Updated `Cargo.toml` root to include praxis_scene dependency and scene_demo example

## Dependencies Added

- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `ron = "0.8"` - RON format support

## Testing

All modules include comprehensive unit tests:

- **loader.rs**: Tests for loading from string, saving to string, roundtrip serialization
- **manager.rs**: Tests for scene spawning, hierarchical spawning, unloading, multiple scenes
- **traversal.rs**: Tests for depth-first iteration, child collection, parent chains, entity depth, name finding

Run tests with:
```bash
cargo test -p praxis_scene
```

## Example Usage

Run the demo:
```bash
cargo run --example scene_demo
```

The demo demonstrates:
1. Programmatic scene creation
2. Entity querying
3. Finding entities by name
4. Scene graph traversal (depth-first)
5. Loading scenes from RON files
6. Scene serialization
7. Scene unloading

## Integration with Praxis

The scene system integrates seamlessly with the existing Praxis architecture:

- Uses `praxis_ecs::World` for entity spawning
- Uses all existing ECS components (Transform, Camera, Lights, etc.)
- Uses `praxis_utils::Result` for error handling
- Uses `praxis_math` types (Vec3, Quat, Mat4)
- Follows Praxis coding conventions and documentation standards
- Includes proper `#[warn]` lints and rustdoc comments

## Future Enhancements

Potential areas for future expansion:

1. **Scene Prefabs**: Support for reusable entity templates
2. **Scene Transitions**: Fade-in/fade-out between scenes
3. **Lazy Loading**: Stream large scenes in chunks
4. **Scene Validation**: Validate scenes before spawning
5. **Scene Editor Integration**: Tools for visual scene editing
6. **Binary Format**: Optional binary serialization for production
7. **Hot Reloading**: Reload scenes at runtime for development
8. **Scene References**: Include other scenes as sub-scenes
9. **Custom Components**: Support for user-defined components in scenes
10. **Scene Queries**: Advanced query system for finding entities
