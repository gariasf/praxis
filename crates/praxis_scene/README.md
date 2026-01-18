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

let mut scene_manager = SceneManager::new();
let scene_loader = SceneLoader::new();

let scene = scene_loader.load_from_file("assets/scenes/level1.ron")?;
let handle = scene_manager.spawn_scene(&mut world, scene)?;
```

### Skeletal Animation

```rust
use praxis_scene::{Skeleton, AnimationPlayer, AnimatedPose};

let skeleton = Skeleton::from_gltf(&gltf_asset)?;
let mut player = AnimationPlayer::new();

world.spawn((
    Transform::default(),
    GlobalTransform::default(),
    skeleton,
    player,
    AnimatedPose::new(bone_count),
));
```

## Scene Format

Scenes use RON format for human-readable definitions:

```ron
(
    name: "Example Scene",
    entities: [
        (
            name: Some("Player"),
            transform: Some((translation: (0.0, 1.0, 0.0), ...)),
            mesh: Some("cube"),
            children: [...],
        ),
    ],
)
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
cargo run --example scene_demo
cargo run --example skeletal_animation_demo
cargo run --example animation_blending_demo
cargo run --example scene_serialization_demo
```

## Dependencies

- `ron` 0.8: Scene serialization
- `serde` 1.0: Serialization framework
- `bevy_ecs` 0.14: ECS integration
