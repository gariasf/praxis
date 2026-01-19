# Praxis Scene

Scene management, transform hierarchy, and skeletal animation for the Praxis game engine.

## Overview

Complete scene graph with serialization, animation, and entity hierarchies.

**Key Features:**
- RON-based scene definitions
- Transform hierarchy with automatic propagation
- Skeletal animation with blending
- Scene serialization/deserialization
- Scene graph traversal and queries

## Quick Start

### Scene Loading

```rust
use praxis_scene::{SceneLoader, SceneManager};
use praxis_ecs::World;
use color_eyre::Result;

fn load_scene(world: &mut World) -> Result<()> {
    // Initialize scene manager
    let mut scene_manager = SceneManager::new();
    
    // Create scene loader
    let scene_loader = SceneLoader::new();
    
    // Load scene from file
    let scene = scene_loader.load_from_file("assets/scenes/level1.ron")?;
    
    // Spawn scene entities into world
    let scene_handle = scene_manager.spawn_scene(world, scene)?;
    
    Ok(())
}
```

### Skeletal Animation

```rust
use praxis_scene::{Skeleton, AnimationPlayer, AnimatedPose, AnimationClip};
use praxis_ecs::{World, Transform, GlobalTransform};
use praxis_assets::GltfLoader;
use color_eyre::Result;

fn setup_animated_character(world: &mut World) -> Result<()> {
    // Load GLTF with animations
    let loader = GltfLoader::new();
    let gltf_asset = loader.load_gltf("assets/models/character.gltf")?;
    
    // Extract skeleton from first skin
    let skeleton = gltf_asset.skins.first()
        .expect("GLTF should have skin")
        .skeleton.clone();
    
    // Create animation player and add clips
    let mut player = AnimationPlayer::new();
    for animation in &gltf_asset.animations {
        let name = animation.name.clone()
            .unwrap_or_else(|| "unnamed".to_string());
        player.add_clip(name.clone(), animation.clip.clone());
    }
    
    // Start playing first animation
    if let Some(first_anim) = gltf_asset.animations.first() {
        if let Some(name) = &first_anim.name {
            player.play(name);
        }
    }
    
    // Create animated pose for bone transforms
    let bone_count = skeleton.bone_count();
    let pose = AnimatedPose::new(bone_count);
    
    // Spawn entity with all animation components
    world.spawn((
        Transform::default(),
        GlobalTransform::default(),
        skeleton,
        player,
        pose,
    ));
    
    Ok(())
}
```

### Transform Hierarchy

```rust
use praxis_scene::{Parent, Children};
use praxis_ecs::{World, Transform, GlobalTransform};
use praxis_math::{Vec3, Quat};

fn create_hierarchy(world: &mut World) {
    // Spawn parent entity
    let parent = world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        Children::default(),
    )).id();
    
    // Spawn child entity with relative transform
    let child = world.spawn((
        Transform::from_xyz(1.0, 0.0, 0.0),  // 1 unit to the right of parent
        GlobalTransform::default(),
        Parent(parent),
    )).id();
    
    // Add child to parent's children list
    if let Some(mut parent_children) = world.get_mut::<Children>(parent) {
        parent_children.0.push(child);
    }
    
    // When parent moves, child automatically moves with it
    // due to transform propagation system
}
```

## Scene Format

Scenes use RON format for human-readable definitions:

```ron
// Example scene file: assets/scenes/level1.ron
(
    name: "Example Scene",
    entities: [
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
                    name: Some("PlayerWeapon"),
                    transform: Some((
                        translation: (0.5, 0.5, 0.0),
                        rotation: (0.0, 0.0, 0.0, 1.0),
                        scale: (0.5, 0.5, 0.5),
                    )),
                    mesh: Some("sword"),
                ),
            ],
        ),
        (
            name: Some("Ground"),
            transform: Some((
                translation: (0.0, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0, 1.0),
                scale: (100.0, 1.0, 100.0),
            )),
            mesh: Some("plane"),
        ),
    ],
)
```

## Animation Blending

```rust
use praxis_scene::{AnimationPlayer, Skeleton, AnimatedPose};
use praxis_ecs::Query;

fn update_animation_blending(
    mut query: Query<(&Skeleton, &mut AnimationPlayer, &mut AnimatedPose)>
) {
    for (skeleton, mut player, mut pose) in query.iter_mut() {
        // Update animation player with delta time
        let delta_time = 0.016; // 60 FPS
        player.update(delta_time);
        
        // Evaluate current animation state and update pose
        *pose = player.evaluate(skeleton);
    }
}
```

## Documentation

**Comprehensive Guides:**
- [Animation Guide](../../docs/guides/animation.md) - Animation system overview
- [Skeletal Animation Basics](../../docs/guides/animation/skeletal-basics.md)
- [Animation Blending](../../docs/guides/animation/blending.md)
- [Advanced Animation Features](../../docs/guides/animation/advanced-features.md)
- [Scene Serialization](../../docs/guides/serialization.md)

**Concepts:**
- [Transform Hierarchy](../../docs/concepts/transform-hierarchy.md)
- [Animation Concepts](../../docs/concepts/animation.md)

**Reference:**
- [Scene Format Reference](../../docs/reference/scene-format.md)
- [Animation API](../../docs/reference/animation-api.md)

## Examples

```bash
# Basic scene rendering
cargo run --example scene_demo

# Skeletal animation
cargo run --example skeletal_animation_demo

# Animation blending
cargo run --example animation_blending_demo

# Scene serialization
cargo run --example scene_serialization_demo
```

## Dependencies

- `ron` 0.8: Scene serialization
- `serde` 1.0: Serialization framework
- `bevy_ecs` 0.14: ECS integration
