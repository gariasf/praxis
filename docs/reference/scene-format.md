# Scene Format Version 2

This document describes the extended scene serialization format introduced in Version 2 of the Praxis engine scene system.

## Overview

Scene Format Version 2 extends the original scene definition system to capture full runtime state including:

- **Physics Components**: RigidBody, Collider, PhysicsVelocity, Mass, Friction, Restitution
- **Audio Components**: AudioSource with spatial audio support
- **Animation Components**: AnimationPlayer and Skeleton with full keyframe data
- **Material Components**: Material handles and PBR material properties

This enables complete scene state preservation, including all runtime gameplay data that was previously lost during serialization.

## Version Migration

The scene system includes automatic migration from older versions:

- **Version 0 → 1**: Adds scene versioning and editor data support
- **Version 1 → 2**: Adds physics, audio, animation, and material support

Old scene files are automatically migrated when loaded, ensuring backward compatibility.

```rust
use praxis_scene::{SceneLoader, migrate_scene};

let mut scene_def = scene_loader.load_from_file("old_scene.ron")?;
// Automatically migrates from v1 to v2
migrate_scene(&mut scene_def)?;
```

## New Component Definitions

### Physics Components

#### RigidBody

Defines the physics body type:

```ron
rigid_body: Some(Dynamic)  // Dynamic, Static, or Kinematic
```

#### Collider

Defines collision shapes:

```ron
// Box collider
collider: Some(Cuboid(hx: 1.0, hy: 1.0, hz: 1.0))

// Sphere collider
collider: Some(Sphere(radius: 0.5))

// Capsule collider (Y-aligned)
collider: Some(CapsuleY(half_height: 0.5, radius: 0.4))

// Capsule collider (X or Z-aligned)
collider: Some(CapsuleX(half_height: 0.5, radius: 0.4))
collider: Some(CapsuleZ(half_height: 0.5, radius: 0.4))

// Cylinder collider
collider: Some(CylinderY(half_height: 1.0, radius: 0.5))
```

#### PhysicsVelocity

Initial velocities for dynamic bodies:

```ron
physics_velocity: Some((
    linear: (1.0, 0.0, 0.0),   // units per second
    angular: (0.0, 0.5, 0.0),  // radians per second
))
```

#### Mass, Friction, Restitution

Physical material properties:

```ron
mass: Some((
    mass: 10.0,
    angular_inertia: 10.0,
))
friction: Some(0.6)      // Friction coefficient
restitution: Some(0.4)   // Bounciness (0.0 = no bounce, 1.0 = perfect bounce)
```

### Audio Components

#### AudioSource

Defines audio playback with spatial support:

```ron
audio_source: Some((
    path: "assets/audio/sound.ogg",
    volume: 0.8,
    spatial: true,           // Enable 3D spatial audio
    looping: true,           // Loop continuously
    auto_play: true,         // Start playing on spawn
    max_distance: 20.0,      // Maximum audible distance
    reference_distance: 1.0, // Distance at full volume
))
```

### Animation Components

#### Skeleton

Defines the bone hierarchy:

```ron
skeleton: Some((
    bones: [
        (
            name: "Root",
            parent_index: None,
            bind_pose_translation: (0.0, 0.0, 0.0),
            bind_pose_rotation: (0.0, 0.0, 0.0, 1.0),
            bind_pose_scale: (1.0, 1.0, 1.0),
        ),
        (
            name: "Spine",
            parent_index: Some(0),  // Child of Root
            bind_pose_translation: (0.0, 1.0, 0.0),
            bind_pose_rotation: (0.0, 0.0, 0.0, 1.0),
            bind_pose_scale: (1.0, 1.0, 1.0),
        ),
    ],
))
```

#### AnimationPlayer

Defines animation clips with keyframe data:

```ron
animation_player: Some((
    clips: [
        (
            name: "Walk",
            duration: 2.0,
            tracks: [
                (
                    bone_index: 0,
                    translation_keyframes: [
                        (time: 0.0, value: (0.0, 0.0, 0.0)),
                        (time: 1.0, value: (0.0, 0.1, 0.0)),
                        (time: 2.0, value: (0.0, 0.0, 0.0)),
                    ],
                    rotation_keyframes: [
                        (time: 0.0, value: (0.0, 0.0, 0.0, 1.0)),
                        (time: 1.0, value: (0.0, 0.1, 0.0, 0.995)),
                        (time: 2.0, value: (0.0, 0.0, 0.0, 1.0)),
                    ],
                    scale_keyframes: [],
                ),
            ],
        ),
    ],
    auto_play: Some("Walk"),  // Optional: clip to play on spawn
))
```

### Material Components

#### Material Handle

Reference to a named material:

```ron
material: Some("brick")
```

#### Material Properties

PBR material properties:

```ron
material_properties: Some((
    base_color: [0.8, 0.8, 0.8, 1.0],  // RGBA tint
    metallic: 0.9,                      // 0.0 = dielectric, 1.0 = metal
    roughness: 0.1,                     // 0.0 = mirror, 1.0 = matte
    emissive_strength: 0.0,             // Self-illumination
))
```

## Complete Example

See `examples/scene_v2_demo.ron` for a complete example demonstrating all new features:

- Dynamic physics objects with various collider shapes
- Static ground with friction properties
- Bouncy objects with restitution
- Character with capsule collider and skeleton
- Ambient and spatial audio sources
- Metallic and emissive materials
- Animated objects with keyframe data

## Use Cases

### Game State Serialization

Save complete game state including physics velocities, animation states, and material properties:

```rust
use praxis_scene::SceneDefinition;

fn save_game_state(world: &World) -> SceneDefinition {
    let mut scene = SceneDefinition::new("SavedGame");
    
    // Serialize all entities with full runtime state
    for entity in world.iter() {
        let mut entity_def = EntityDefinition::new();
        
        // Capture physics state
        if let Some(rb) = world.get::<RigidBody>(entity) {
            entity_def.rigid_body = Some(serialize_rigid_body(rb));
        }
        if let Some(vel) = world.get::<PhysicsVelocity>(entity) {
            entity_def.physics_velocity = Some(serialize_velocity(vel));
        }
        
        // Capture animation state
        if let Some(player) = world.get::<AnimationPlayer>(entity) {
            entity_def.animation_player = Some(serialize_animation(player));
        }
        
        scene.add_entity(entity_def);
    }
    
    scene
}
```

### Editor Scene Persistence

The editor can save complete scene state including editor-specific data:

```rust
// Save scene with editor data
scene.set_editor_data(EditorData::new()
    .with_camera(editor_camera_state)
    .with_selected_entities(selected)
    .with_viewport(viewport_settings)
);

scene_loader.save_to_file(&scene, "scene.ron")?;
```

### Prefab System

Create reusable prefabs with full physics, audio, and animation:

```ron
// prefabs/explosive_barrel.ron
(
    version: 2,
    name: "Explosive Barrel Prefab",
    entities: [
        (
            name: Some("Barrel"),
            mesh: Some("barrel"),
            material: Some("metal"),
            rigid_body: Some(Dynamic),
            collider: Some(CylinderY(half_height: 0.5, radius: 0.3)),
            mass: Some((mass: 50.0, angular_inertia: 50.0)),
            audio_source: Some((
                path: "assets/audio/explosion.ogg",
                volume: 1.0,
                spatial: true,
                looping: false,
                auto_play: false,
                max_distance: 50.0,
                reference_distance: 1.0,
            )),
        ),
    ],
)
```

## Migration Notes

When migrating from Version 1 to Version 2:

1. **Backward Compatible**: All v1 scenes load correctly in v2 with new fields set to `None`
2. **No Manual Migration Required**: The migration system handles everything automatically
3. **Optional Fields**: All new fields use `#[serde(default)]` and are optional
4. **Format Evolution**: Future versions can add more fields using the same pattern

## Performance Considerations

### Serialization

- Physics state adds minimal overhead (~32 bytes per physics entity)
- Animation data size depends on keyframe count
- Material properties are compact (32 bytes)
- Audio sources are lightweight (path + metadata)

### Deserialization

- Lazy loading: Components are only instantiated if present
- Efficient parsing: RON format is human-readable but fast to parse
- Memory usage scales with scene complexity

## Best Practices

1. **Use Material Handles**: Reference shared materials by name instead of duplicating properties
2. **Minimize Keyframes**: Use only necessary keyframes; interpolation handles the rest
3. **Spatial Audio**: Enable spatial audio only when needed for better performance
4. **Editor Data**: Clear editor data for runtime scenes using `to_runtime_scene()`
5. **Version Control**: Include version field in all scene files for future compatibility

## RON Format Reference

The scene format uses RON (Rusty Object Notation) for human readability and Git-friendly diffs:

```ron
(
    version: 2,
    name: "Scene Name",
    metadata: (
        description: Some("Scene description"),
        author: Some("Author Name"),
        version: Some("1.0"),
        tags: ["tag1", "tag2"],
    ),
    entities: [
        // Entity definitions
    ],
    editor_data: Some((
        // Editor-specific data
    )),
)
```

## Validation

The scene system validates loaded scenes:

```rust
use praxis_scene::validate_scene;

let scene = scene_loader.load_from_file("scene.ron")?;
validate_scene(&scene)?;  // Checks for invalid data
```

Validation checks:
- Valid version number
- Non-empty scene name
- Valid component data (e.g., positive mass, valid keyframe times)
- Consistent parent-child relationships
- Valid camera parameters
- Proper collider dimensions

## Future Extensions

Version 3 may include:
- Constraint definitions (joints, springs)
- Particle system configurations
- Terrain heightmap data
- Navigation mesh data
- Light probe data
- Custom component support

The versioned migration system ensures all future additions remain backward compatible.
