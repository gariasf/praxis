# praxis_scene

Scene graph and animation system for Praxis engine.

## Overview

Manages transform hierarchies, skeletal animation, and scene organization.

## Features

### Transform Hierarchy

- **Transform**: Local position, rotation, scale
- **GlobalTransform**: Computed world-space transform
- **Parent/Children**: Hierarchy relationships
- Automatic propagation from parent to children

### Skeletal Animation

- **Skeleton**: Bone hierarchy
- **AnimationClip**: Keyframe data
- **AnimationPlayer**: Playback control
- **Blend Trees**: Combine multiple animations
- **Layered Animation**: Override specific bones
- **Cross-fading**: Smooth transitions

## Example

```rust
use praxis_scene::{Transform, Parent, Children};

// Transform hierarchy
let parent = commands.spawn(Transform::from_xyz(0.0, 1.0, 0.0)).id();
let child = commands.spawn((
    Transform::from_xyz(1.0, 0.0, 0.0),
    Parent(parent),
)).id();

// Animation
let mut player = AnimationPlayer::new();
player.play(animation_clip).set_speed(1.5).repeat();
```

## Dependencies

- `serde`: Serialization support
- `rustc-hash`: Fast hash maps

## Usage

```toml
praxis_scene = { path = "../praxis_scene", version = "0.1.0" }
```
