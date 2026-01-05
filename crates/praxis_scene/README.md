# Praxis Scene

Scene management system for the Praxis game engine.

## Features

- **Scene Definitions**: Define scenes in RON (Rusty Object Notation) format
- **Scene Loading/Unloading**: Load scene definitions from files and spawn entities into the ECS world
- **Entity Spawning**: Automatically spawn entities with transforms, meshes, cameras, lights, and hierarchical relationships
- **Scene Management**: Track and manage multiple loaded scene instances
- **Scene Graph Traversal**: Utilities for traversing and querying the scene hierarchy
- **Entity Finding**: Find entities by name in the scene graph
- **Animation Support**: Skeletal animation and animation clip loading
- **Scene Serialization**: Save and load scenes to/from RON format

## Scene Definition Format (RON)

Scenes are defined using RON format, which provides a human-readable and editable way to define game scenes:

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

## Usage

### Loading and Spawning a Scene

```rust
use praxis_scene::{SceneLoader, SceneManager};
use praxis_ecs::World;

let mut world = World::new();
let mut scene_manager = SceneManager::new();
let scene_loader = SceneLoader::new();

// Load a scene from a RON file
let scene_def = scene_loader.load_from_file("assets/scenes/level1.ron")?;

// Spawn the scene into the world
let scene_handle = scene_manager.spawn_scene(&mut world, scene_def)?;
```

### Creating Scenes Programmatically

```rust
use praxis_scene::{SceneDefinition, EntityDefinition, TransformDef, CameraDef, CameraType};

let mut scene = SceneDefinition::new("My Scene");

// Add a camera entity
let camera = EntityDefinition::new()
    .with_name("MainCamera")
    .with_transform(TransformDef::from_translation(0.0, 5.0, 10.0))
    .with_camera(CameraDef::perspective(
        70.0_f32.to_radians(),
        16.0 / 9.0,
        0.1,
        1000.0
    ));

scene.add_entity(camera);

// Spawn the scene
let handle = scene_manager.spawn_scene(&mut world, scene)?;
```

### Scene Graph Traversal

```rust
use praxis_scene::{SceneGraphIterator, TraversalOrder, get_all_children, find_entity_by_name};

// Traverse a scene graph depth-first
for entity in SceneGraphIterator::new(&world, root_entity, TraversalOrder::DepthFirst) {
    println!("Visiting entity: {:?}", entity);
}

// Get all descendants of an entity
let descendants = get_all_children(&world, parent_entity);

// Find an entity by name
if let Some(player) = find_entity_by_name(&world, "Player", None) {
    println!("Found player: {:?}", player);
}
```

### Unloading Scenes

```rust
// Unload a specific scene
scene_manager.unload_scene(&mut world, &scene_handle);

// Unload all scenes
scene_manager.unload_all(&mut world);
```

## Supported Entity Components

The scene system supports the following components from `praxis_ecs`:

- **Transform**: Position, rotation, and scale
- **Name**: Entity name for identification
- **MeshHandle**: Reference to a mesh asset
- **TextureHandle**: Reference to a texture asset
- **Camera**: Camera component with perspective or orthographic projection
- **DirectionalLight**: Directional light (e.g., sun)
- **PointLight**: Omnidirectional light with position and range
- **Visibility**: Visible/Hidden state
- **Active**: Active/Inactive state
- **Hierarchical relationships**: Parent-child relationships via `Parent` and `Children` components

## Animation Support

The scene system includes support for skeletal animation:

```rust
use praxis_scene::{Skeleton, AnimationClip, AnimationPlayer};

// Load skeleton and animations from GLTF
let skeleton = Skeleton::from_gltf(&gltf_asset)?;
let animation = AnimationClip::from_gltf(&gltf_asset, 0)?;

// Create animation player
let mut player = AnimationPlayer::new(skeleton);
player.play_animation(animation);

// Update animation each frame
player.update(delta_time);
```

## Examples

Run the scene demos:

```bash
# Basic scene demo
cargo run --example scene_demo

# Comprehensive scene with all features
cargo run --example comprehensive_scene_demo

# Scene serialization
cargo run --example scene_serialization_demo

# Animation demos
cargo run --example animation_demo
cargo run --example skeletal_animation_demo
cargo run --example animation_blending_demo
cargo run --example gltf_animation_loader_demo
```

## Architecture

- **`components.rs`**: Scene component and handle types
- **`definition.rs`**: Scene definition structures that can be serialized/deserialized
- **`loader.rs`**: Scene loading from/to RON files
- **`manager.rs`**: Scene spawning, tracking, and unloading
- **`traversal.rs`**: Scene graph traversal and query utilities
- **`animation.rs`**: Animation system for skeletal animation
- **`skeleton.rs`**: Skeleton and bone hierarchy
- **`clip.rs`**: Animation clip data structures

## Dependencies

- `ron`: Rusty Object Notation for scene serialization
- `serde`: Serialization framework
- `bevy_ecs` 0.14: ECS integration
- `praxis_ecs`: Transform and component systems
- `praxis_math`: Math types (Vec3, Quat, Mat4)
- `praxis_assets`: Asset loading for GLTF animations
- `praxis_utils`: Error handling

## See Also

- [Scene System Guide](../../docs/guides/scenes.md)
- [Animation System](../../docs/animation_system.md)
- [GLTF Assets](../praxis_assets/README.md)
- [ECS System](../praxis_ecs/README.md)
