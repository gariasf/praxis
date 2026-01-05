# Skeletal Animation Guide

## Overview

Skeletal animation enables realistic character and object animation through a hierarchical bone structure. This guide covers the architecture, implementation, and best practices for skeletal animation in Praxis.

## Architecture

### Core Components

The skeletal animation system consists of several interconnected components:

```
Skeleton (structure)
    ↓
AnimationClip (keyframe data)
    ↓
AnimationPlayer (playback control)
    ↓
AnimatedPose (computed transforms)
    ↓
Rendering (GPU skinning)
```

### Component Breakdown

#### 1. `Bone`

Represents a single bone in the skeleton:

```rust
pub struct Bone {
    pub name: String,
    pub parent_index: Option<usize>,
    pub bind_pose_translation: Vec3,
    pub bind_pose_rotation: Quat,
    pub bind_pose_scale: Vec3,
}
```

**Key Concepts**:
- **Bind Pose**: Rest position where mesh is authored
- **Parent Index**: Defines hierarchy (None = root bone)
- **Local Space**: Transform relative to parent bone

#### 2. `Skeleton`

ECS component defining the complete bone hierarchy:

```rust
#[derive(Component)]
pub struct Skeleton {
    bones: Vec<Bone>,
    bone_name_to_index: HashMap<String, usize>,
    inverse_bind_matrices: Vec<Mat4>,
}
```

**Key Features**:
- Stores all bones in linear array
- Fast bone lookup by name
- Precomputed inverse bind matrices for GPU skinning

#### 3. `Keyframe<T>`

Stores a value at a specific time:

```rust
pub struct Keyframe<T> {
    pub time: f32,
    pub value: T,
}
```

Generic over Vec3 (translation/scale) or Quat (rotation).

#### 4. `BoneTrack`

Animation data for a single bone:

```rust
pub struct BoneTrack {
    translation_keys: Vec<Keyframe<Vec3>>,
    rotation_keys: Vec<Keyframe<Quat>>,
    scale_keys: Vec<Keyframe<Vec3>>,
}
```

**Key Features**:
- Three independent channels (TRS)
- Automatic interpolation between keyframes
- Optional channels (can animate only rotation, for example)

#### 5. `AnimationClip`

A complete animation sequence:

```rust
pub struct AnimationClip {
    name: String,
    duration: f32,
    bone_tracks: HashMap<usize, BoneTrack>,
}
```

**Key Features**:
- Named clips for easy reference
- Duration in seconds
- Sparse storage (only animated bones)

#### 6. `AnimationPlayer`

ECS component controlling playback:

```rust
#[derive(Component)]
pub struct AnimationPlayer {
    clips: HashMap<String, AnimationClip>,
    playing_clips: HashMap<String, PlayingClip>,
}
```

**Playback Controls**:
- Play, pause, resume, stop
- Speed control (2x, 0.5x, etc.)
- Looping mode
- Animation blending with weights

#### 7. `AnimatedPose`

ECS component storing computed transforms:

```rust
#[derive(Component)]
pub struct AnimatedPose {
    local_transforms: Vec<Mat4>,
    world_transforms: Vec<Mat4>,
    skinning_matrices: Vec<Mat4>,
}
```

**Transform Spaces**:
- **Local**: Relative to parent bone
- **World**: Accumulated from root
- **Skinning**: Final matrices for GPU (world × inverse_bind)

## Creating Skeletons

### Manual Skeleton Creation

```rust
use praxis_scene::{Skeleton, Bone};
use praxis_math::{Vec3, Quat};

// Define bones
let skeleton = Skeleton::new(vec![
    Bone::with_bind_pose(
        "Root".to_string(),
        None,  // Root has no parent
        Vec3::ZERO,
        Quat::IDENTITY,
        Vec3::ONE,
    ),
    Bone::with_bind_pose(
        "Spine".to_string(),
        Some(0),  // Parent is Root (index 0)
        Vec3::new(0.0, 1.0, 0.0),
        Quat::IDENTITY,
        Vec3::ONE,
    ),
    Bone::with_bind_pose(
        "Head".to_string(),
        Some(1),  // Parent is Spine (index 1)
        Vec3::new(0.0, 0.5, 0.0),
        Quat::IDENTITY,
        Vec3::ONE,
    ),
]);
```

### Loading from GLTF

```rust
use praxis_assets::load_gltf;

// Load model with skeleton
let (meshes, skeleton, animations) = load_gltf("models/character.gltf")?;

// Skeleton and animations are automatically extracted
```

## Creating Animation Clips

### Manual Keyframe Animation

```rust
use praxis_scene::AnimationClip;

// Create 2-second walk animation
let mut walk_clip = AnimationClip::new("Walk".to_string(), 2.0);

// Animate root bone translation
let root_bone = skeleton.find_bone("Root").unwrap();
walk_clip.add_translation_keyframe(root_bone, 0.0, Vec3::ZERO);
walk_clip.add_translation_keyframe(root_bone, 1.0, Vec3::new(1.0, 0.0, 0.0));
walk_clip.add_translation_keyframe(root_bone, 2.0, Vec3::new(2.0, 0.0, 0.0));

// Animate spine rotation
let spine_bone = skeleton.find_bone("Spine").unwrap();
walk_clip.add_rotation_keyframe(spine_bone, 0.0, Quat::IDENTITY);
walk_clip.add_rotation_keyframe(
    spine_bone,
    1.0,
    Quat::from_rotation_y(0.2)
);
walk_clip.add_rotation_keyframe(spine_bone, 2.0, Quat::IDENTITY);
```

### Procedural Animation

```rust
// Generate bouncing animation
let mut bounce_clip = AnimationClip::new("Bounce".to_string(), 1.0);
let head_bone = skeleton.find_bone("Head").unwrap();

for i in 0..10 {
    let time = i as f32 * 0.1;
    let height = (time * std::f32::consts::TAU).sin() * 0.2;
    bounce_clip.add_translation_keyframe(
        head_bone,
        time,
        Vec3::new(0.0, height, 0.0)
    );
}
```

## Animation Playback

### Basic Playback

```rust
use praxis_scene::AnimationPlayer;

// Create player and add clips
let mut player = AnimationPlayer::new();
player.add_clip("Walk".to_string(), walk_clip);
player.add_clip("Run".to_string(), run_clip);
player.add_clip("Idle".to_string(), idle_clip);

// Play animation
player.play("Walk");

// Control playback
player.pause("Walk");
player.resume("Walk");
player.stop("Walk");
```

### Looping and Speed

```rust
// Enable looping
player.set_looping("Walk", true);

// Adjust speed
player.set_speed("Walk", 2.0);  // 2x speed
player.set_speed("Idle", 0.5);  // Half speed
```

### Animation Blending

Blend multiple animations together with weights:

```rust
// Play multiple animations
player.play("Walk");
player.play("Wave");

// Set blend weights (must sum to 1.0 for proper blending)
player.set_weight("Walk", 0.7);
player.set_weight("Wave", 0.3);

// Result: 70% walk, 30% wave animation
```

**Common Blending Scenarios**:

- **Walk to Run**: Gradually shift weight from walk (1.0 → 0.0) to run (0.0 → 1.0)
- **Upper Body Override**: Blend full body animation with upper body only animation
- **Additive Animation**: Add small procedural movements (breathing, idle fidget)

## Keyframe Interpolation

The system automatically interpolates between keyframes using:

### Linear Interpolation (Translation and Scale)

```rust
fn lerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a + (b - a) * t
}
```

### Spherical Linear Interpolation (Rotation)

```rust
fn slerp(a: Quat, b: Quat, t: f32) -> Quat {
    a.slerp(b, t)
}
```

**Why SLERP?**: Ensures constant angular velocity and avoids gimbal lock.

### Finding Keyframes

Binary search finds surrounding keyframes:

```rust
// Find keyframes at time = 1.3s
// Returns keyframes at t=1.0 and t=2.0
// Interpolation factor: (1.3 - 1.0) / (2.0 - 1.0) = 0.3
```

## Transform Evaluation

### Hierarchy Evaluation Order

Transforms are evaluated parent-to-child:

```
1. Sample keyframes for current time
2. Build local transform (TRS decomposition)
3. Multiply with parent world transform
4. Store as world transform
5. Repeat for children
```

### Transform Composition

```rust
// TRS composition
let local_transform = 
    Mat4::from_translation(translation) *
    Mat4::from_quat(rotation) *
    Mat4::from_scale(scale);

// World transform
let world_transform = parent_world_transform * local_transform;

// Skinning matrix
let skinning_matrix = world_transform * inverse_bind_matrix;
```

## ECS Integration

### Complete Setup

```rust
use praxis_ecs::{World, Query};
use praxis_scene::{Skeleton, AnimationPlayer, AnimatedPose, update_animations};

// Spawn animated entity
let mut world = World::new();
let entity = world.spawn((
    skeleton,
    AnimationPlayer::new(),
    AnimatedPose::new(skeleton.bone_count()),
    Transform::default(),
));

// Update system (call every frame)
fn animation_system(
    delta_time: f32,
    mut query: Query<(&Skeleton, &mut AnimationPlayer, &mut AnimatedPose)>
) {
    update_animations(delta_time, &mut query);
}
```

### Update Function

The `update_animations` function:
1. Updates playback time for all playing clips
2. Evaluates keyframes at current time
3. Blends multiple animations if active
4. Computes world transforms from hierarchy
5. Generates skinning matrices for GPU

## GPU Skinning

### Shader Integration

Pass skinning matrices to vertex shader:

```glsl
layout(set = 2, binding = 0) uniform SkinningMatrices {
    mat4 matrices[256];  // Max 256 bones
} skinning;

void main() {
    // Weighted skinning (up to 4 bones per vertex)
    mat4 skin_matrix =
        skinning.matrices[bone_ids.x] * bone_weights.x +
        skinning.matrices[bone_ids.y] * bone_weights.y +
        skinning.matrices[bone_ids.z] * bone_weights.z +
        skinning.matrices[bone_ids.w] * bone_weights.w;
    
    vec4 skinned_pos = skin_matrix * vec4(position, 1.0);
    gl_Position = projection * view * model * skinned_pos;
}
```

### Inverse Bind Matrices

Pre-multiply to avoid per-vertex computation:

```rust
// CPU: Precompute final matrices
for (bone_index, world_transform) in world_transforms.iter().enumerate() {
    let inverse_bind = skeleton.inverse_bind_matrix(bone_index);
    skinning_matrices[bone_index] = world_transform * inverse_bind;
}

// GPU: Direct matrix multiplication (no inverse needed)
vec4 skinned_pos = skinning_matrix * vec4(position, 1.0);
```

## Advanced Techniques

### Animation Events

Trigger events at specific times (footsteps, sounds):

```rust
// Define event markers
struct AnimationEvent {
    time: f32,
    event_type: String,
}

// Check for events
if player.current_time() >= event.time && !event.triggered {
    match event.event_type.as_str() {
        "footstep" => play_footstep_sound(),
        "land" => spawn_dust_particles(),
        _ => {}
    }
    event.triggered = true;
}
```

### Bone Attachment

Attach objects to bones (weapons, accessories):

```rust
// Get bone world transform
let hand_bone = skeleton.find_bone("RightHand").unwrap();
let hand_transform = pose.world_transform(hand_bone);

// Position weapon
weapon_transform.set_from_matrix(hand_transform);
```

### Partial Animation

Animate only specific bones (upper body only):

```rust
// Create upper body mask
let upper_body_bones = ["Spine", "LeftArm", "RightArm", "Head"];
let mut clip = AnimationClip::new("UpperBodyWave".to_string(), 1.0);

// Only add keyframes for upper body bones
for bone_name in upper_body_bones {
    let bone_id = skeleton.find_bone(bone_name).unwrap();
    clip.add_rotation_keyframe(bone_id, 0.0, start_rotation);
    // ... more keyframes
}
```

### Animation Compression

Reduce memory for long animations:

```rust
// Remove redundant keyframes
fn optimize_keyframes(track: &mut BoneTrack, threshold: f32) {
    let mut i = 1;
    while i < track.rotation_keys.len() - 1 {
        let prev = &track.rotation_keys[i - 1];
        let curr = &track.rotation_keys[i];
        let next = &track.rotation_keys[i + 1];
        
        // Check if current keyframe can be interpolated
        let t = (curr.time - prev.time) / (next.time - prev.time);
        let interpolated = prev.value.slerp(next.value, t);
        
        if interpolated.dot(curr.value) > 1.0 - threshold {
            track.rotation_keys.remove(i);
        } else {
            i += 1;
        }
    }
}
```

## Performance Optimization

### Bone Count Limits

- **Target**: 100-150 bones per character
- **Maximum**: 256 bones (GPU uniform limit)
- **Optimization**: Remove unused bones, merge small bones

### Update Frequency

```rust
// Update animations at lower frequency
const ANIMATION_UPDATE_HZ: f32 = 30.0;
let animation_dt = 1.0 / ANIMATION_UPDATE_HZ;

if accumulated_time >= animation_dt {
    update_animations(animation_dt, query);
    accumulated_time -= animation_dt;
}
```

### LOD (Level of Detail)

```rust
// Reduce animation quality at distance
let distance = (camera_pos - entity_pos).length();
if distance > 50.0 {
    // Update at half rate
    if frame_count % 2 == 0 {
        update_animations(delta_time * 2.0, query);
    }
} else {
    // Full rate
    update_animations(delta_time, query);
}
```

### Culling

```rust
// Don't update animations for off-screen entities
if !frustum.contains(entity_bounds) {
    continue;  // Skip animation update
}
```

## Troubleshooting

### Animation Not Playing

- Verify clip is added to player: `player.has_clip("Walk")`
- Check clip duration is > 0
- Ensure `update_animations()` is called each frame
- Verify delta_time is not zero

### Mesh Deforming Incorrectly

- Check bone indices in mesh data match skeleton
- Verify bone weights sum to 1.0 per vertex
- Ensure inverse bind matrices are computed correctly
- Check skinning matrices are uploaded to GPU

### Performance Issues

- Profile animation update time
- Reduce bone count
- Lower update frequency
- Implement culling for off-screen entities
- Use animation compression

## Example: Complete Character Animation

```rust
use praxis_scene::*;
use praxis_ecs::*;
use praxis_math::*;

// Create character skeleton
let skeleton = Skeleton::new(vec![
    Bone::with_bind_pose("Root".into(), None, Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
    Bone::with_bind_pose("Hips".into(), Some(0), Vec3::Y, Quat::IDENTITY, Vec3::ONE),
    Bone::with_bind_pose("Spine".into(), Some(1), Vec3::Y * 0.3, Quat::IDENTITY, Vec3::ONE),
    // ... more bones
]);

// Create walk cycle
let mut walk = AnimationClip::new("Walk".into(), 1.0);
walk.add_translation_keyframe(0, 0.0, Vec3::ZERO);
walk.add_translation_keyframe(0, 0.5, Vec3::new(0.0, 0.1, 0.0));
walk.add_translation_keyframe(0, 1.0, Vec3::ZERO);

// Setup animation player
let mut player = AnimationPlayer::new();
player.add_clip("Walk".into(), walk);
player.play("Walk");
player.set_looping("Walk", true);

// Spawn entity
let mut world = World::new();
world.spawn((
    skeleton.clone(),
    player,
    AnimatedPose::new(skeleton.bone_count()),
    Transform::default(),
));

// Update loop
loop {
    let delta = 0.016;  // 60 FPS
    update_animations(delta, &mut world.query());
}
```

## See Also

- [Animation Blending Guide](blending.md)
- [Advanced Animation Features](advanced-features.md)
- [GLTF Loading Guide](../gltf_loading.md)
- [Rendering Guide](../rendering.md)

## References

- "Game Programming Gems" - Animation Systems
- "Game Engine Architecture" - Jason Gregory, Chapter 11
- Khronos GLTF 2.0 Specification - Skinning
