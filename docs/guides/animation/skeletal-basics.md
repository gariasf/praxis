# Skeletal Animation Basics

This guide covers the fundamentals of skeletal animation in Praxis, including core architecture, data structures, keyframe interpolation, and GLTF workflow.

## Table of Contents

1. [Overview](#overview)
2. [Core Architecture](#core-architecture)
3. [Animation Data Structures](#animation-data-structures)
4. [Keyframe Interpolation](#keyframe-interpolation)
5. [GLTF Animation Workflow](#gltf-animation-workflow)
6. [Basic Usage Examples](#basic-usage-examples)

---

## Overview

The Praxis skeletal animation system provides a complete solution for character animation in games. It supports:

- **Hierarchical bone structures** with parent-child relationships
- **Keyframe animation** with smooth interpolation
- **Multiple animation playback** with weight-based blending
- **GLTF file support** for loading industry-standard animated models
- **ECS integration** for efficient animation updates

The system is built on the Entity-Component-System (ECS) architecture using `bevy_ecs`, ensuring optimal performance and data-oriented design.

### High-Level Animation Flow

```text
Animation System Flow
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────────────────────────────────────────────┐
│                    Asset Loading                             │
│  GLTF File → Skeleton + AnimationClips                       │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                 ECS Components                               │
│  Entity: Skeleton + AnimationPlayer + AnimatedPose          │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│              Animation Update (per frame)                    │
│  1. Update playback time                                     │
│  2. Sample keyframes at current time                         │
│  3. Interpolate between keyframes                            │
│  4. Blend multiple animations                                │
│  5. Compute final bone transforms                            │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                 Pose Computation                             │
│  Local Transforms → World Transforms → Skinning Matrices    │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                  GPU Skinning                                │
│  Vertex Shader applies bone transforms to mesh vertices     │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Architecture

### Component Hierarchy

The animation system consists of several ECS components that work together:

```rust
// Core components
Skeleton          // Defines bone hierarchy and bind poses
AnimationPlayer   // Controls playback of animation clips
AnimatedPose      // Stores computed bone transforms

// Supporting data structures
Bone              // Individual bone with parent reference
AnimationClip     // Keyframe data for an animation
BoneTrack         // Keyframes for a single bone
```

### Skeleton Component

A skeleton defines the hierarchical structure of bones:

```text
Skeleton Structure
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Example: Humanoid Character

Root (pelvis)
 │
 ├─► Spine
 │    │
 │    ├─► Left Arm
 │    │    └─► Left Hand
 │    │
 │    └─► Right Arm
 │         └─► Right Hand
 │
 ├─► Left Leg
 │    └─► Left Foot
 │
 └─► Right Leg
      └─► Right Foot

Each bone stores:
  - Name (for identification)
  - Parent index (None for root bones)
  - Bind pose (rest position/rotation/scale)
  - World transform (computed from hierarchy)
  - Inverse bind matrix (for skinning)
```

**Key Concepts:**

- **Bind Pose**: The default "rest" position of the skeleton
- **Local Transform**: Position/rotation/scale relative to parent bone
- **World Transform**: Absolute position in world space (computed from hierarchy)
- **Inverse Bind Matrix**: Transforms from world space to bone local space (used for skinning)

### Bone Hierarchy and Transforms

Bones form a tree structure where child bones are positioned relative to their parents:

```text
Transform Hierarchy
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Parent Bone (Shoulder)
  Local Transform: T_parent
  World Transform: W_parent = T_parent
  │
  └─► Child Bone (Elbow)
       Local Transform: T_child (relative to parent)
       World Transform: W_child = W_parent × T_child
       │
       └─► Grandchild Bone (Hand)
            Local Transform: T_grandchild
            World Transform: W_grandchild = W_child × T_grandchild

Matrix Multiplication Chain:
  W_hand = W_shoulder × T_elbow × T_hand

When shoulder rotates:
  - All children (elbow, hand) move with it
  - Local transforms stay the same
  - World transforms update automatically
```

**Implementation:**

```rust
pub struct Skeleton {
    bones: Vec<Bone>,
    bone_name_to_index: HashMap<String, usize>,
    inverse_bind_matrices: Vec<Mat4>,
}

pub struct Bone {
    pub name: String,
    pub parent_index: Option<usize>,
    pub bind_pose_translation: Vec3,
    pub bind_pose_rotation: Quat,
    pub bind_pose_scale: Vec3,
}

impl Skeleton {
    pub fn new(bones: Vec<Bone>) -> Self {
        // Build name lookup
        let bone_name_to_index = bones.iter()
            .enumerate()
            .map(|(i, bone)| (bone.name.clone(), i))
            .collect();
        
        // Compute inverse bind matrices
        let inverse_bind_matrices = Self::compute_inverse_bind_matrices(&bones);
        
        Self {
            bones,
            bone_name_to_index,
            inverse_bind_matrices,
        }
    }
    
    fn compute_inverse_bind_matrices(bones: &[Bone]) -> Vec<Mat4> {
        let mut world_transforms = vec![Mat4::IDENTITY; bones.len()];
        
        // Compute world space bind pose for each bone
        for i in 0..bones.len() {
            let bone = &bones[i];
            let local = bone.bind_pose_matrix();
            
            world_transforms[i] = match bone.parent_index {
                Some(parent) => world_transforms[parent] * local,
                None => local,
            };
        }
        
        // Invert to get bone space from world space
        world_transforms.iter().map(|m| m.inverse()).collect()
    }
}
```

---

## Animation Data Structures

### AnimationClip

An animation clip stores keyframe data for multiple bones:

```text
AnimationClip Structure
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

AnimationClip "Walk" (duration: 2.0s)
│
├─► Bone 0 (Root) Track
│    ├─► Translation Keyframes: [(0.0, (0,0,0)), (1.0, (1,0,0)), (2.0, (2,0,0))]
│    ├─► Rotation Keyframes: [(0.0, Identity), (1.0, Quat(0,0.1,0,1)), ...]
│    └─► Scale Keyframes: [(0.0, (1,1,1)), ...]
│
├─► Bone 1 (Spine) Track
│    ├─► Translation Keyframes: [...]
│    ├─► Rotation Keyframes: [...]
│    └─► Scale Keyframes: [...]
│
└─► Bone N Track
     └─► ...

Each track can animate:
  - Translation (position)
  - Rotation (orientation)
  - Scale (size)
  
Keyframes are sorted by time for efficient sampling.
```

**Implementation:**

```rust
pub struct AnimationClip {
    name: String,
    duration: f32,
    bone_tracks: HashMap<usize, BoneTrack>,  // Indexed by bone
}

pub struct BoneTrack {
    pub translation_keyframes: Vec<Keyframe<Vec3>>,
    pub rotation_keyframes: Vec<Keyframe<Quat>>,
    pub scale_keyframes: Vec<Keyframe<Vec3>>,
}

pub struct Keyframe<T> {
    pub time: f32,
    pub value: T,
}

impl AnimationClip {
    pub fn new(name: String, duration: f32) -> Self {
        Self {
            name,
            duration,
            bone_tracks: HashMap::new(),
        }
    }
    
    pub fn add_translation_keyframe(&mut self, bone_idx: usize, time: f32, value: Vec3) {
        self.add_bone_track(bone_idx)
            .add_translation_keyframe(time, value);
    }
    
    // Similar methods for rotation and scale...
}
```

### AnimatedPose

The `AnimatedPose` component stores the final computed transforms for rendering:

```text
AnimatedPose Storage
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

For N bones:

Local Transforms (Vec<Mat4>):
  [Bone 0 local matrix, Bone 1 local matrix, ..., Bone N local matrix]
  ↓ Apply hierarchy (propagate from parents to children)

World Transforms (Vec<Mat4>):
  [Bone 0 world matrix, Bone 1 world matrix, ..., Bone N world matrix]
  ↓ Multiply by inverse bind matrices

Skinning Matrices (Vec<Mat4>):
  [Bone 0 skin matrix, Bone 1 skin matrix, ..., Bone N skin matrix]
  ↓ Upload to GPU for vertex skinning
```

**Implementation:**

```rust
pub struct AnimatedPose {
    local_transforms: Vec<Mat4>,    // Relative to parent
    world_transforms: Vec<Mat4>,    // Absolute in world space
    skinning_matrices: Vec<Mat4>,   // Final GPU matrices
}

impl AnimatedPose {
    pub fn update_world_transforms(&mut self, skeleton: &Skeleton) {
        for i in 0..self.local_transforms.len() {
            if let Some(bone) = skeleton.bone(i) {
                self.world_transforms[i] = match bone.parent_index {
                    Some(parent) => 
                        self.world_transforms[parent] * self.local_transforms[i],
                    None => 
                        self.local_transforms[i],
                };
            }
        }
    }
    
    pub fn update_skinning_matrices(&mut self, skeleton: &Skeleton) {
        for i in 0..self.world_transforms.len() {
            if let Some(inverse_bind) = skeleton.inverse_bind_matrix(i) {
                self.skinning_matrices[i] = 
                    self.world_transforms[i] * inverse_bind;
            }
        }
    }
}
```

---

## Keyframe Interpolation

Keyframe interpolation is the process of computing bone transforms between keyframes to create smooth animation.

### Interpolation Methods

Different transform components use different interpolation methods:

```text
Interpolation Types
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. Linear Interpolation (LERP) - Translation & Scale
   
   Time:      0.0        0.5        1.0
   Value:     A          ?          B
   
   Result at t=0.5:  lerp(A, B, 0.5) = A + 0.5 × (B - A)
   
   Formula: V(t) = V₀ + t × (V₁ - V₀)
   
   Properties:
   - Simple and fast
   - Maintains constant speed between keyframes
   - Works well for position and scale


2. Spherical Linear Interpolation (SLERP) - Rotation
   
   Rotation:  R₀         ?          R₁
   Time:      0.0        0.5        1.0
   
   Result at t=0.5:  slerp(R₀, R₁, 0.5)
   
   Formula: R(t) = sin((1-t)θ)/sin(θ) × R₀ + sin(tθ)/sin(θ) × R₁
   where θ = angle between R₀ and R₁
   
   Properties:
   - Maintains constant angular velocity
   - Shortest path on quaternion sphere
   - Essential for smooth rotation
   - More expensive than LERP


3. Keyframe Search Algorithm
   
   Given time t, find surrounding keyframes:
   
   Keyframes: [0.0, 0.5, 1.0, 1.5, 2.0]
   Query: t = 1.3
   
   Search:
     before = 1.0 (largest keyframe ≤ t)
     after  = 1.5 (smallest keyframe ≥ t)
     
   Interpolate:
     weight = (1.3 - 1.0) / (1.5 - 1.0) = 0.6
     result = interpolate(keyframe[1.0], keyframe[1.5], 0.6)
```

### Implementation

```rust
impl BoneTrack {
    pub fn sample_translation(&self, time: f32) -> Option<Vec3> {
        if self.translation_keyframes.is_empty() {
            return None;
        }
        
        // Find surrounding keyframes
        let mut before = None;
        let mut after = None;
        
        for keyframe in &self.translation_keyframes {
            if keyframe.time <= time {
                before = Some(keyframe);
            }
            if keyframe.time >= time && after.is_none() {
                after = Some(keyframe);
            }
        }
        
        match (before, after) {
            (Some(b), Some(a)) if b.time != a.time => {
                // Interpolate between keyframes
                let t = (time - b.time) / (a.time - b.time);
                Some(b.value.lerp(a.value, t))
            }
            (Some(k), _) | (_, Some(k)) => {
                // At or beyond a keyframe
                Some(k.value)
            }
            _ => None,
        }
    }
    
    pub fn sample_rotation(&self, time: f32) -> Option<Quat> {
        // Similar to translation, but uses SLERP instead of LERP
        // ...
        match (before, after) {
            (Some(b), Some(a)) if b.time != a.time => {
                let t = (time - b.time) / (a.time - b.time);
                Some(b.value.slerp(a.value, t))  // SLERP for rotations
            }
            // ...
        }
    }
}
```

### Interpolation Visualization

```text
Translation Interpolation (LERP)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Keyframe 0 (t=0.0):  Position (0, 0, 0)
Keyframe 1 (t=1.0):  Position (10, 5, 0)

Interpolated positions:
  t=0.00:  (0.0, 0.0, 0.0)
  t=0.25:  (2.5, 1.25, 0.0)  ← Linear progression
  t=0.50:  (5.0, 2.5, 0.0)
  t=0.75:  (7.5, 3.75, 0.0)
  t=1.00:  (10.0, 5.0, 0.0)

Path: Straight line from start to end


Rotation Interpolation (SLERP)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Keyframe 0 (t=0.0):  Rotation 0° around Y
Keyframe 1 (t=1.0):  Rotation 180° around Y

SLERP ensures:
  t=0.00:  0°    ┐
  t=0.25:  45°   │ Equal angular steps
  t=0.50:  90°   │ Shortest path on sphere
  t=0.75:  135°  │ Constant angular velocity
  t=1.00:  180°  ┘

Naive LERP would produce uneven rotation speed!
```

---

## GLTF Animation Workflow

GLTF (GL Transmission Format) is an industry-standard format for 3D assets, including skeletal animations.

### GLTF File Structure

```text
GLTF Animation Structure
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

GLTF File
├─ Meshes
│  └─ Character mesh with vertex weights
│
├─ Skins
│  ├─ Skeleton definition
│  ├─ Bone hierarchy (nodes)
│  └─ Inverse bind matrices
│
├─ Animations
│  ├─ Animation 1: "Walk"
│  │  ├─ Channel (bone 0): Translation
│  │  │  └─ Sampler: Input times + Output values
│  │  ├─ Channel (bone 1): Rotation
│  │  │  └─ Sampler: Input times + Output quaternions
│  │  └─ ...
│  │
│  └─ Animation 2: "Run"
│     └─ ...
│
└─ Nodes (Transform hierarchy)
   ├─ Root
   ├─ Spine (child of Root)
   └─ ...
```

### Loading GLTF Animations

The `GltfLoader` extracts animation data from GLTF files:

```rust
use praxis_assets::GltfLoader;

let loader = GltfLoader::new();
let asset = loader.load_gltf("assets/models/character.gltf")?;

// Access skeleton
for skin in &asset.skins {
    println!("Skeleton: {:?}", skin.name);
    println!("  Bones: {}", skin.skeleton.bone_count());
}

// Access animations
for animation in &asset.animations {
    println!("Animation: {:?}", animation.name);
    println!("  Duration: {:.2}s", animation.duration);
    println!("  Tracks: {}", animation.clip.track_count());
}
```

### GLTF Animation Mapping

The loader converts GLTF data to Praxis structures:

```text
GLTF to Praxis Mapping
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

GLTF Skin
  └─► Praxis Skeleton
       ├─ GLTF nodes → Bone hierarchy
       ├─ GLTF joint names → Bone names
       └─ GLTF inverseBindMatrices → inverse_bind_matrices

GLTF Animation
  └─► Praxis AnimationClip
       ├─ GLTF channel target → bone_index
       ├─ GLTF sampler input → keyframe times
       ├─ GLTF sampler output → keyframe values
       │  ├─ "translation" path → translation_keyframes
       │  ├─ "rotation" path → rotation_keyframes (GLTF uses XYZW, convert to Quat)
       │  └─ "scale" path → scale_keyframes
       └─ max(sampler input times) → clip duration

GLTF Interpolation Modes:
  - LINEAR: Direct support (default)
  - STEP: Set keyframes at exact times, no interpolation
  - CUBICSPLINE: Currently approximated as LINEAR (TODO: cubic support)
```

### Example: Loading and Playing GLTF Animation

```rust
use praxis_assets::GltfLoader;
use praxis_ecs::World;
use praxis_scene::{AnimationPlayer, AnimatedPose};

// Load GLTF file
let loader = GltfLoader::new();
let asset = loader.load_gltf("assets/models/animated_character.gltf")?;

// Create animation player
let mut player = AnimationPlayer::new();
for animation in &asset.animations {
    let name = animation.name.clone()
        .unwrap_or_else(|| format!("Animation{}", player.clips().len()));
    player.add_clip(name.clone(), animation.clip.clone());
}

// Spawn animated entity
let mut world = World::new();
if let Some(skin) = asset.skins.first() {
    let skeleton = skin.skeleton.clone();
    let pose = AnimatedPose::new(skeleton.bone_count());
    
    world.spawn((skeleton, player, pose));
}

// Play first animation
// (Access player component and call player.play("AnimationName"))
```

---

## Basic Usage Examples

### Example 1: Simple Animation

```rust
use praxis_scene::{Skeleton, AnimationClip, AnimationPlayer, AnimatedPose, Bone};
use praxis_math::{Vec3, Quat};
use praxis_ecs::World;

// Create skeleton
let skeleton = Skeleton::new(vec![
    Bone::with_bind_pose(
        "Root".to_string(),
        None,
        Vec3::ZERO,
        Quat::IDENTITY,
        Vec3::ONE,
    ),
    Bone::with_bind_pose(
        "Arm".to_string(),
        Some(0),
        Vec3::new(1.0, 0.0, 0.0),
        Quat::IDENTITY,
        Vec3::ONE,
    ),
]);

// Create animation
let mut clip = AnimationClip::new("Wave".to_string(), 1.0);
clip.add_rotation_keyframe(1, 0.0, Quat::IDENTITY);
clip.add_rotation_keyframe(1, 0.5, Quat::from_rotation_z(PI / 2.0));
clip.add_rotation_keyframe(1, 1.0, Quat::IDENTITY);

// Setup player
let mut player = AnimationPlayer::new();
player.add_clip("Wave".to_string(), clip);
player.play("Wave");

// Spawn entity
let mut world = World::new();
let pose = AnimatedPose::new(skeleton.bone_count());
world.spawn((skeleton, player, pose));
```

### Example 2: Animation Update System

```rust
pub fn update_animations(
    delta_time: f32,
    query: &mut Query<(&Skeleton, &mut AnimationPlayer, &mut AnimatedPose)>,
) {
    for (skeleton, mut player, mut pose) in query.iter_mut() {
        // Update animation playback times
        player.update(delta_time);
        
        // Evaluate animations and update pose
        *pose = player.evaluate(skeleton);
    }
}
```

### Example 3: Loading from GLTF

```rust
use praxis_assets::GltfLoader;

let loader = GltfLoader::new();
let asset = loader.load_gltf("assets/models/character.gltf")?;

// Extract animations
let mut player = AnimationPlayer::new();
for animation in &asset.animations {
    let name = animation.name.clone().unwrap_or_default();
    player.add_clip(name, animation.clip.clone());
}

// Use skeleton
if let Some(skin) = asset.skins.first() {
    let skeleton = skin.skeleton.clone();
    let pose = AnimatedPose::new(skeleton.bone_count());
    world.spawn((skeleton, player, pose));
}
```

---

## Performance Considerations

### Memory Layout

The animation system uses contiguous memory for cache efficiency:

```text
Memory Layout
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

AnimatedPose (for N bones):

┌─────────────────────────────────────────────────────────┐
│ local_transforms: Vec<Mat4>                             │
│ [Mat4, Mat4, Mat4, ..., Mat4]                          │
│  ^                                                      │
│  └─ N × 64 bytes (4×4 matrix of f32)                   │
│     Total: N × 64 bytes                                │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ world_transforms: Vec<Mat4>                             │
│ [Mat4, Mat4, Mat4, ..., Mat4]                          │
│  Total: N × 64 bytes                                   │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ skinning_matrices: Vec<Mat4>                            │
│ [Mat4, Mat4, Mat4, ..., Mat4]                          │
│  Total: N × 64 bytes                                   │
└─────────────────────────────────────────────────────────┘

Total memory per animated entity: N × 192 bytes

Example: 50-bone character = 50 × 192 = 9,600 bytes ≈ 9.4 KB

Cache considerations:
  - Sequential access patterns for vector operations
  - Bones processed in order (parent before children)
  - SIMD-friendly Mat4 operations
```

### Optimization Strategies

1. **Early Exit for Stopped Animations**
   ```rust
   if playing.state != PlaybackState::Playing {
       continue;  // Skip evaluation
   }
   ```

2. **Weight Threshold**
   ```rust
   if weight < 0.001 {
       continue;  // Skip negligible contributions
   }
   ```

3. **Keyframe Search Optimization**
   - Keyframes sorted by time (binary search possible)
   - Current implementation uses linear search (fine for <100 keyframes)
   - For huge animations: implement binary search

4. **SIMD Operations**
   - `glam` crate uses SIMD for vector/matrix math
   - Mat4 multiplication, lerp, slerp all use SIMD when available

### Performance Metrics

```text
Performance Characteristics
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Operation               Cost (CPU)    Notes
────────────────────────────────────────────────────────────
Sample keyframe         ~10-50 ns     Linear search
LERP (Vec3)            ~2-5 ns       SIMD optimized
SLERP (Quat)           ~10-20 ns     More expensive than LERP
Mat4 multiply          ~10-20 ns     SIMD optimized
Pose propagation       O(N bones)    Depth-first traversal

Example: 50-bone character, single animation
  - 50 bones × (sample + interpolate) ≈ 1-2 µs
  - Pose propagation (50 bones) ≈ 1-2 µs
  - Total per entity: ~2-4 µs

At 60 FPS with 100 animated characters:
  - Animation update: 0.2-0.4 ms
  - ~1-2% of 16.67ms frame budget
```

### Scalability

```text
Scalability Guidelines
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Number of Bones:
  ✓ <50 bones: Excellent (typical humanoid)
  ✓ 50-100 bones: Good (detailed character)
  ⚠ 100-200 bones: Acceptable (facial animation)
  ✗ >200 bones: May need LOD or culling

Animated Entities:
  ✓ <100 entities: Excellent
  ✓ 100-500 entities: Good
  ⚠ 500-1000 entities: May need LOD
  ✗ >1000 entities: Definitely need LOD/culling

Optimization Techniques:
  - Distance-based LOD (simpler skeletons far away)
  - Frustum culling (don't update offscreen)
  - Update frequency reduction (animate at 30Hz far away)
  - GPU skinning (offload to GPU)
```

---

## Next Steps

- **[Animation Blending](blending.md)**: Learn about cross-fades, blend trees, and layered animation
- **[Advanced Features](advanced-features.md)**: Explore IK, retargeting, additive blending, and root motion
- **[Examples](../../examples/)**: See working examples in the `examples/` directory
