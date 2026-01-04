# Skeletal Animation System Implementation

This document describes the skeletal animation system implemented in `praxis_scene`.

## Overview

The skeletal animation system provides a complete solution for keyframe-based skeletal animation in the Praxis game engine. It supports:

- Hierarchical bone structures with parent-child relationships
- Keyframe animation with automatic interpolation
- Multiple animation clips per entity
- Animation playback control (play, pause, resume, stop)
- Animation blending with weights
- Looping and speed control
- Efficient bone hierarchy evaluation

## Architecture

### Core Components

#### 1. Bone (`praxis_scene::Bone`)

Represents a single bone in a skeleton with:
- Name for identification
- Optional parent bone index
- Bind pose (rest position) as translation, rotation, and scale

```rust
pub struct Bone {
    pub name: String,
    pub parent_index: Option<usize>,
    pub bind_pose_translation: Vec3,
    pub bind_pose_rotation: Quat,
    pub bind_pose_scale: Vec3,
}
```

#### 2. Skeleton (`praxis_scene::Skeleton`)

ECS component defining the complete bone hierarchy:
- Vector of bones with parent-child relationships
- Bone name to index mapping for quick lookups
- Precomputed inverse bind matrices for GPU skinning

```rust
#[derive(Component)]
pub struct Skeleton {
    bones: Vec<Bone>,
    bone_name_to_index: HashMap<String, usize>,
    inverse_bind_matrices: Vec<Mat4>,
}
```

#### 3. Keyframe (`praxis_scene::Keyframe<T>`)

Stores a value at a specific time:
- Time in seconds
- Generic value (Vec3, Quat, etc.)

```rust
pub struct Keyframe<T> {
    pub time: f32,
    pub value: T,
}
```

#### 4. BoneTrack (`praxis_scene::BoneTrack`)

Animation data for a single bone:
- Translation keyframes (Vec3)
- Rotation keyframes (Quat)
- Scale keyframes (Vec3)

Each channel is optional and independently animated. Provides sampling methods with automatic interpolation.

#### 5. AnimationClip (`praxis_scene::AnimationClip`)

A complete animation sequence:
- Name and duration
- Map of bone indices to BoneTrack instances
- Methods to add keyframes for any bone

```rust
pub struct AnimationClip {
    name: String,
    duration: f32,
    bone_tracks: HashMap<usize, BoneTrack>,
}
```

#### 6. AnimationPlayer (`praxis_scene::AnimationPlayer`)

ECS component controlling animation playback:
- Library of animation clips
- Currently playing animations with state
- Playback control methods
- Animation blending support

```rust
#[derive(Component)]
pub struct AnimationPlayer {
    clips: HashMap<String, AnimationClip>,
    playing_clips: HashMap<String, PlayingClip>,
}
```

#### 7. AnimatedPose (`praxis_scene::AnimatedPose`)

ECS component storing computed bone transforms:
- Local space transforms (relative to parent)
- World space transforms (accumulated from root)
- Final skinning matrices for GPU rendering

```rust
#[derive(Component)]
pub struct AnimatedPose {
    local_transforms: Vec<Mat4>,
    world_transforms: Vec<Mat4>,
    skinning_matrices: Vec<Mat4>,
}
```

## Key Features

### 1. Keyframe Interpolation

The system automatically interpolates between keyframes:

- **Translation**: Linear interpolation (lerp) for smooth position changes
- **Rotation**: Spherical linear interpolation (slerp) for smooth orientation changes
- **Scale**: Linear interpolation for smooth size changes

Interpolation finds the two keyframes surrounding the current time and blends them based on the fractional position.

### 2. Bone Hierarchy Evaluation

Bone transforms are evaluated in parent-to-child order:
1. Sample keyframes for current time
2. Build local transform matrix (TRS)
3. Accumulate world transform by multiplying with parent
4. Compute skinning matrix by multiplying with inverse bind matrix

This ensures child bones move correctly with their parents.

### 3. Animation Blending

Multiple animations can play simultaneously with weights:

```rust
player.play("Walk");
player.set_weight("Walk", 0.7);
player.play("Idle");
player.set_weight("Idle", 0.3);
```

The system blends transforms using weighted lerp/slerp, allowing smooth transitions between animations.

### 4. Playback Control

Full control over animation playback:
- **Play**: Start an animation from the beginning
- **Pause**: Freeze at current time
- **Resume**: Continue from paused time
- **Stop**: Stop and remove from playing list
- **Looping**: Repeat animation indefinitely or play once
- **Speed**: Control playback rate (2x, 0.5x, etc.)

### 5. Inverse Bind Matrices

The skeleton precomputes inverse bind matrices for efficient GPU skinning:
1. Compute world-space bind pose for each bone
2. Invert to get bone-to-world space transform
3. Store for use in skinning shader

This allows vertex skinning to transform vertices from bind pose to animated pose.

## Usage Example

```rust
use praxis_scene::{Skeleton, AnimationClip, AnimationPlayer, AnimatedPose, Bone, update_animations};
use praxis_ecs::{World, Query};
use praxis_math::{Vec3, Quat};

// 1. Create skeleton
let skeleton = Skeleton::new(vec![
    Bone::with_bind_pose(
        "Root".to_string(),
        None,
        Vec3::ZERO,
        Quat::IDENTITY,
        Vec3::ONE,
    ),
    Bone::with_bind_pose(
        "Spine".to_string(),
        Some(0),
        Vec3::new(0.0, 1.0, 0.0),
        Quat::IDENTITY,
        Vec3::ONE,
    ),
]);

// 2. Create animation clip
let mut clip = AnimationClip::new("Walk".to_string(), 2.0);
clip.add_translation_keyframe(0, 0.0, Vec3::ZERO);
clip.add_translation_keyframe(0, 2.0, Vec3::new(2.0, 0.0, 0.0));

// 3. Setup animation player
let mut player = AnimationPlayer::new();
player.add_clip("Walk".to_string(), clip);
player.play("Walk");

// 4. Spawn entity
let mut world = World::new();
let pose = AnimatedPose::new(skeleton.bone_count());
world.spawn((skeleton, player, pose));

// 5. Update in game loop
fn animation_system(mut query: Query<(&Skeleton, &mut AnimationPlayer, &mut AnimatedPose)>) {
    let delta_time = 0.016; // From timing system
    update_animations(delta_time, &mut query);
}
```

## API Reference

### Skeleton

- `new(bones: Vec<Bone>) -> Self`: Create skeleton from bone list
- `bone_count() -> usize`: Get number of bones
- `bone(&self, index: usize) -> Option<&Bone>`: Get bone by index
- `find_bone(&self, name: &str) -> Option<usize>`: Find bone by name
- `inverse_bind_matrix(&self, index: usize) -> Option<Mat4>`: Get inverse bind matrix

### AnimationClip

- `new(name: String, duration: f32) -> Self`: Create new clip
- `add_bone_track(&mut self, bone_index: usize)`: Add track for bone
- `add_translation_keyframe(&mut self, bone_index: usize, time: f32, translation: Vec3)`: Add translation key
- `add_rotation_keyframe(&mut self, bone_index: usize, time: f32, rotation: Quat)`: Add rotation key
- `add_scale_keyframe(&mut self, bone_index: usize, time: f32, scale: Vec3)`: Add scale key

### AnimationPlayer

- `new() -> Self`: Create empty player
- `add_clip(&mut self, name: String, clip: AnimationClip)`: Add clip to library
- `play(&mut self, name: &str)`: Start playing animation
- `pause(&mut self, name: &str)`: Pause animation
- `resume(&mut self, name: &str)`: Resume paused animation
- `stop(&mut self, name: &str)`: Stop animation
- `set_looping(&mut self, name: &str, looping: bool)`: Set loop mode
- `set_speed(&mut self, name: &str, speed: f32)`: Set playback speed
- `set_weight(&mut self, name: &str, weight: f32)`: Set blend weight
- `update(&mut self, delta_time: f32)`: Update playback time
- `evaluate(&self, skeleton: &Skeleton) -> AnimatedPose`: Produce animated pose

### AnimatedPose

- `new(bone_count: usize) -> Self`: Create pose for skeleton
- `local_transforms(&self) -> &[Mat4]`: Get local transforms
- `world_transforms(&self) -> &[Mat4]`: Get world transforms
- `skinning_matrices(&self) -> &[Mat4]`: Get skinning matrices
- `update_world_transforms(&mut self, skeleton: &Skeleton)`: Update hierarchy
- `update_skinning_matrices(&mut self, skeleton: &Skeleton)`: Update skinning data

## Example Demonstration

The `examples/skeletal_animation_demo.rs` example demonstrates:

1. Creating a 3-bone skeleton (Root → Spine → Head)
2. Defining two animation clips:
   - "Walk": Translation and rotation animation
   - "Idle": Subtle bobbing motion
3. Animation playback control
4. Pause/resume functionality
5. Speed modification (2x playback)
6. Animation blending with weights

Run with:
```bash
cargo run --example skeletal_animation_demo
```

## Implementation Files

- `crates/praxis_scene/src/animation.rs`: Complete implementation (940+ lines)
- `examples/skeletal_animation_demo.rs`: Comprehensive example
- `CLAUDE.md`: Updated documentation
- `examples/README.md`: Example listing

## Future Enhancements

Possible extensions to the system:

1. **Animation Events**: Trigger callbacks at specific times
2. **Animation Layers**: Separate upper/lower body animations
3. **Inverse Kinematics (IK)**: Procedural bone positioning
4. **Animation Compression**: Reduce memory usage for long clips
5. **Animation State Machine**: FSM for animation transitions
6. **Root Motion**: Extract root movement from animation
7. **Animation Retargeting**: Apply animations to different skeletons
8. **GPU Skinning**: Vertex shader bone deformation

## Testing

The implementation includes comprehensive unit tests:
- Bone creation and bind pose matrices
- Skeleton bone lookup and hierarchy
- Keyframe interpolation (translation, rotation, scale)
- Animation clip track management
- Animation player playback control
- Animation looping and speed control
- Pose evaluation and transform updates

Run tests with:
```bash
cargo test -p praxis_scene
```

## Performance Considerations

- Inverse bind matrices are precomputed at skeleton creation
- Keyframes are kept sorted for efficient binary search
- Transform hierarchy is evaluated iteratively (no recursion)
- Animation blending uses early-out for weight extremes (0.0 or 1.0)
- Local transforms cached in AnimatedPose for reuse

## Conclusion

The skeletal animation system provides a solid foundation for animated characters and objects in the Praxis engine. It implements industry-standard techniques (keyframe interpolation, bone hierarchies, animation blending) in a clean, well-documented API that integrates seamlessly with the ECS architecture.
