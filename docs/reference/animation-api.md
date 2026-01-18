# Skeletal Animation System

This document provides a comprehensive guide to the skeletal animation system in Praxis, covering the architecture, algorithms, blending techniques, and GLTF workflow.

## Table of Contents

1. [Overview](#overview)
2. [Core Architecture](#core-architecture)
3. [Animation Data Structures](#animation-data-structures)
4. [Keyframe Interpolation](#keyframe-interpolation)
5. [Animation Blending System](#animation-blending-system)
6. [GLTF Animation Workflow](#gltf-animation-workflow)
7. [Implementation Details](#implementation-details)
8. [Performance Considerations](#performance-considerations)
9. [Examples and Usage Patterns](#examples-and-usage-patterns)

---

## Overview

The Praxis skeletal animation system provides a complete solution for character animation in games. It supports:

- **Hierarchical bone structures** with parent-child relationships
- **Keyframe animation** with smooth interpolation
- **Multiple animation playback** with weight-based blending
- **Advanced blending features**: cross-fades, blend trees, layered animation
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
AnimationBlender  // Advanced blending (optional)

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

## Animation Blending System

The animation blending system allows multiple animations to play simultaneously and blend smoothly between states.

### Basic Animation Blending

The `AnimationPlayer` supports simple weighted blending:

```text
Basic Blending
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Animation "Walk" (weight: 0.7)
  Bone 0 Translation: (1.0, 0.0, 0.0)
  +
Animation "Run" (weight: 0.3)
  Bone 0 Translation: (2.0, 0.0, 0.0)
  =
Blended Result:
  Translation = 0.7 × (1.0, 0.0, 0.0) + 0.3 × (2.0, 0.0, 0.0)
              = (0.7, 0, 0) + (0.6, 0, 0)
              = (1.3, 0.0, 0.0)

For each bone:
  - Sample both animations at current time
  - Blend translation: weighted LERP
  - Blend rotation: weighted SLERP
  - Blend scale: weighted LERP
```

**Implementation:**

```rust
impl AnimationPlayer {
    fn apply_clip_to_pose(
        clip: &AnimationClip,
        time: f32,
        weight: f32,
        pose: &mut AnimatedPose,
        skeleton: &Skeleton,
    ) {
        for (bone_index, track) in clip.bone_tracks() {
            let bone = skeleton.bone(*bone_index)?;
            
            // Sample animation
            let translation = track.sample_translation(time)
                .unwrap_or(bone.bind_pose_translation);
            let rotation = track.sample_rotation(time)
                .unwrap_or(bone.bind_pose_rotation);
            let scale = track.sample_scale(time)
                .unwrap_or(bone.bind_pose_scale);
            
            if weight >= 0.999 {
                // Full weight: just set directly
                let transform = Mat4::from_scale_rotation_translation(
                    scale, rotation, translation
                );
                pose.set_local_transform(*bone_index, transform);
            } else if weight > 0.001 {
                // Partial weight: blend with existing
                let current = pose.local_transform(*bone_index)?;
                
                // Extract current TRS
                let current_translation = current.translation();
                let current_rotation = Quat::from_mat4(&current);
                let current_scale = current.scale();
                
                // Blend each component
                let blended_translation = current_translation.lerp(translation, weight);
                let blended_rotation = current_rotation.slerp(rotation, weight);
                let blended_scale = current_scale.lerp(scale, weight);
                
                let blended = Mat4::from_scale_rotation_translation(
                    blended_scale, blended_rotation, blended_translation
                );
                pose.set_local_transform(*bone_index, blended);
            }
        }
    }
}
```

### Advanced Blending: AnimationBlender

The `AnimationBlender` component provides sophisticated blending features:

#### 1. Cross-Fade Transitions

Smooth transitions between animations over time:

```text
Cross-Fade Transition
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Time:     0.0      0.1      0.2      0.3      0.4
          │        │        │        │        │
Idle:     1.0      0.67     0.33     0.0      0.0
          ████████ █████░░░ ███░░░░░ ░░░░░░░░ ░░░░░░░░
          
Walk:     0.0      0.33     0.67     1.0      1.0
          ░░░░░░░░ ░░░████  ░░██████ ████████ ████████
          │        │        │        │        │
          Start    25%      50%      75%      Complete

Blend weight calculation:
  weight = elapsed / duration
  
  Idle weight = 1.0 - weight
  Walk weight = weight
  
Final pose = Idle × (1.0 - weight) + Walk × weight
```

**Usage:**

```rust
let mut blender = AnimationBlender::new();
blender.add_clip("Idle", idle_clip);
blender.add_clip("Walk", walk_clip);

blender.play("Idle");
// ... later
blender.cross_fade("Idle", "Walk", 0.3);  // 0.3 second transition
```

#### 2. Blend Trees

##### 1D Blend Trees

Blend animations along a single parameter (e.g., speed):

```text
1D Blend Tree
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Parameter: Speed (0.0 to 1.0)

Clips:
  Idle  at 0.0
  Walk  at 0.5
  Run   at 1.0

Current Speed: 0.75

Computation:
  0.75 is between Walk (0.5) and Run (1.0)
  
  Range = 1.0 - 0.5 = 0.5
  t = (0.75 - 0.5) / 0.5 = 0.5
  
  Walk weight = 1.0 - 0.5 = 0.5
  Run weight  = 0.5
  
Result: 50% Walk + 50% Run

Visual representation:
  0.0         0.5         1.0
  Idle        Walk        Run
  │           │     ●     │
  └───────────┴─────┴─────┘
                    ↑
               Speed = 0.75
```

**Usage:**

```rust
let mut blend_tree = BlendNode1D::new();
blend_tree.add_clip("Idle", 0.0);
blend_tree.add_clip("Walk", 0.5);
blend_tree.add_clip("Run", 1.0);

blender.add_blend_tree("Movement", blend_tree.into());
blender.activate_blend_tree("Movement");
blender.set_blend_parameter("Movement", 0.75);
```

##### 2D Blend Trees

Blend animations in a 2D space (e.g., directional movement):

```text
2D Blend Tree
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Parameter: Direction (X, Y)

Clip Positions:
        Forward
          (0, 1)
            │
            │
  Left ─────┼───── Right
  (-1, 0)   │     (1, 0)
            │
          Back
          (0, -1)

Current Position: (0.5, 0.5) - Forward-Right

Inverse Distance Weighting:
  For each clip, compute distance to current position
  Weight inversely proportional to squared distance
  
  d_forward = sqrt((0-0.5)² + (1-0.5)²) = 0.707
  d_right   = sqrt((1-0.5)² + (0-0.5)²) = 0.707
  d_left    = sqrt((-1-0.5)² + (0-0.5)²) = 1.58
  d_back    = sqrt((0-0.5)² + (-1-0.5)²) = 1.58
  
  w_forward = 1 / (0.707² + ε) ≈ 2.0
  w_right   = 1 / (0.707² + ε) ≈ 2.0
  w_left    = 1 / (1.58² + ε) ≈ 0.4
  w_back    = 1 / (1.58² + ε) ≈ 0.4
  
  Normalize:
    total = 2.0 + 2.0 + 0.4 + 0.4 = 4.8
    
    Forward: 2.0 / 4.8 ≈ 0.42
    Right:   2.0 / 4.8 ≈ 0.42
    Left:    0.4 / 4.8 ≈ 0.08
    Back:    0.4 / 4.8 ≈ 0.08

Result: 42% Forward + 42% Right + 8% Left + 8% Back
```

**Usage:**

```rust
let mut blend_tree = BlendNode2D::new();
blend_tree.add_clip("Forward", 0.0, 1.0);
blend_tree.add_clip("Back", 0.0, -1.0);
blend_tree.add_clip("Left", -1.0, 0.0);
blend_tree.add_clip("Right", 1.0, 0.0);

blender.add_blend_tree("Locomotion", blend_tree.into());
blender.activate_blend_tree("Locomotion");
blender.set_blend_parameters_2d("Locomotion", 0.5, 0.5);
```

#### 3. Layered Animation

Multiple animation layers with bone masking:

```text
Layered Animation
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Skeleton:
  Root
  ├─ Spine
  │  ├─ Left Arm
  │  └─ Right Arm
  ├─ Left Leg
  └─ Right Leg

Layer 0 (Base): Walk Animation
  Weight: 1.0
  Mask: All bones
  ████████████████  Full body walking

Layer 1 (Upper): Wave Animation
  Weight: 1.0
  Mask: Right Arm only
  ░░░░░░░░████░░░░  Only right arm waves
  
Result: Character walks with full body while right arm waves

Blend Algorithm:
  1. Evaluate base layer → pose_base
  2. For each additional layer:
     a. Evaluate layer animation → pose_layer
     b. For each bone:
        if mask.is_bone_enabled(bone_idx):
          if layer.blend_mode == Override:
            pose[bone] = lerp(pose_base[bone], pose_layer[bone], layer.weight)
          elif layer.blend_mode == Additive:
            delta = pose_layer[bone] - reference_pose[bone]
            pose[bone] = pose_base[bone] + delta × layer.weight
```

**Usage:**

```rust
let mut blender = AnimationBlender::new();
blender.add_clip("Walk", walk_clip);
blender.add_clip("Wave", wave_clip);

// Base layer: full body walk
blender.play("Walk");

// Create upper body mask
let mut upper_body_mask = BoneMask::with_bone_count(skeleton.bone_count());
upper_body_mask.enable_bone(right_arm_index);

// Add layer for waving
let mut upper_layer = AnimationLayer::new(1.0);
upper_layer.set_mask(upper_body_mask);
upper_layer.set_blend_mode(LayerBlendMode::Override);

blender.add_layer(upper_layer);
blender.play_on_layer(0, "Wave");
```

#### 4. Additive Blending

Add animation deltas on top of a base animation:

```text
Additive Blending
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Reference Pose (Bind Pose):
  Spine rotation: 0°

Additive Animation (Recoil):
  Spine rotation at t=0: 0°
  Spine rotation at t=0.2: -15° (leaning back)
  
  Delta = Current - Reference = -15° - 0° = -15°

Base Animation (Walk):
  Spine rotation: 5° (slight forward lean)

Additive Result:
  Final = Base + (Delta × Weight)
  Final = 5° + (-15° × 1.0) = -10°
  
Effect: Walking animation with recoil added on top

Benefits:
  - Additive animations are independent of base pose
  - Can be applied to any base animation
  - Useful for: recoil, flinch, breathing, procedural motion
```

**Usage:**

```rust
let mut additive_node = AdditiveBlendNode::new();
additive_node.set_base("Walk");
additive_node.set_additive("Recoil");
additive_node.set_weight(1.0);

blender.add_blend_tree("CombatMovement", additive_node.into());
blender.activate_blend_tree("CombatMovement");
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
  - CUBICSPLINE: Currently approximated as LINEAR (cubic interpolation is a potential future enhancement)
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

## Implementation Details

### Animation Update System

The animation system is driven by a system that updates all animated entities each frame:

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

For the advanced blender:

```rust
pub fn update_animation_blenders(
    delta_time: f32,
    query: &mut Query<(&Skeleton, &mut AnimationBlender, &mut AnimatedPose)>,
) {
    for (skeleton, mut blender, mut pose) in query.iter_mut() {
        blender.update(delta_time);
        *pose = blender.evaluate(skeleton);
    }
}
```

### Evaluation Pipeline

The complete evaluation pipeline for a single frame:

```text
Animation Evaluation Pipeline
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. Update Playback Time
   current_time += delta_time × speed
   if current_time >= duration:
     if looping: current_time %= duration
     else: stop()

2. Initialize Pose (Bind Pose)
   for each bone:
     local_transform[bone] = bind_pose_matrix[bone]

3. Sample All Playing Clips
   for each playing_clip in clips:
     for each bone_track in clip:
       translation = sample_translation(current_time)
       rotation = sample_rotation(current_time)
       scale = sample_scale(current_time)
       
       blend into pose using clip.weight

4. Propagate Transforms (Hierarchy)
   for each bone in depth-first order:
     if bone has parent:
       world_transform[bone] = world_transform[parent] × local_transform[bone]
     else:
       world_transform[bone] = local_transform[bone]

5. Compute Skinning Matrices
   for each bone:
     skinning_matrix[bone] = world_transform[bone] × inverse_bind_matrix[bone]

6. Upload to GPU
   Update uniform buffer with skinning_matrices
   Bind to vertex shader for skinning
```

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

---

## Performance Considerations

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

4. **Bone Mask Optimization**
   - Only evaluate bones enabled in mask
   - Reduces work for layered animations

5. **SIMD Operations**
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

Example: 50-bone character, 2 animations blending
  - 50 bones × 2 clips × (sample + interpolate) ≈ 2-5 μs
  - Pose propagation (50 bones) ≈ 1-2 μs
  - Total per entity: ~5-10 μs

At 60 FPS with 100 animated characters:
  - Animation update: 0.5-1.0 ms
  - ~1-2% of 16.67ms frame budget

Bottlenecks to watch:
  - Large numbers of blend tree clips (O(clips) evaluation)
  - Deep bone hierarchies (O(N²) worst case for propagation)
  - Many layered animations (O(layers × bones))
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

Animation Clips per Entity:
  ✓ 1-3 clips: Excellent (typical)
  ✓ 3-8 clips: Good (complex blending)
  ⚠ 8-16 clips: Acceptable (blend trees)
  ✗ >16 clips: Optimize (reduce active clips)

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

## Examples and Usage Patterns

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

### Example 2: Cross-Fade Transition

```rust
use praxis_scene::AnimationBlender;

let mut blender = AnimationBlender::new();
blender.add_clip("Idle", idle_clip);
blender.add_clip("Walk", walk_clip);

// Start with idle
blender.play("Idle");

// Later, smoothly transition to walk
blender.cross_fade("Idle", "Walk", 0.3);  // 0.3 second transition
```

### Example 3: 1D Blend Tree

```rust
use praxis_scene::BlendNode1D;

let mut blend_tree = BlendNode1D::new();
blend_tree.add_clip("Idle", 0.0);
blend_tree.add_clip("Walk", 0.5);
blend_tree.add_clip("Run", 1.0);

blender.add_blend_tree("Movement", blend_tree.into());
blender.activate_blend_tree("Movement");

// Change speed dynamically
blender.set_blend_parameter("Movement", 0.75); // 75% between walk and run
```

### Example 4: Layered Animation

```rust
use praxis_scene::{AnimationLayer, BoneMask, LayerBlendMode};

// Play walk on base layer
blender.play("Walk");

// Create upper body mask
let mut mask = BoneMask::with_bone_count(skeleton.bone_count());
mask.enable_bone_and_children_with_skeleton(spine_bone_index, &skeleton);

// Add layer for upper body action
let mut layer = AnimationLayer::new(1.0);
layer.set_mask(mask);
layer.set_blend_mode(LayerBlendMode::Override);

blender.add_layer(layer);
blender.play_on_layer(0, "Aim");

// Result: Character walks while aiming with upper body
```

### Example 5: Loading from GLTF

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

## Summary

The Praxis skeletal animation system provides:

- **Robust foundation**: Hierarchical bones, keyframe interpolation, smooth blending
- **Advanced features**: Cross-fades, blend trees, layered animation, additive blending
- **Industry compatibility**: GLTF file support for standard workflows
- **Performance**: ECS-based design, SIMD optimizations, efficient memory layout
- **Flexibility**: Multiple blending modes, bone masking, dynamic parameter control

The system is designed to handle everything from simple character animations to complex multi-layered, parameter-driven animation states typical in modern games.

---

## Advanced Animation Features

For more sophisticated animation techniques, see [Advanced Animation Features](animation-advanced-features.md):

- **Inverse Kinematics (IK)**: Procedural limb positioning for hands reaching objects, feet on terrain, head tracking, etc.
  - Two-bone IK for arms/legs
  - Chain IK for spines/tails
  - Look-at IK for aiming/tracking

- **Animation Retargeting**: Apply animations from one skeleton to another
  - Automatic bone mapping by name
  - Manual mapping for custom rigs
  - Cross-species animation support

- **Enhanced Additive Blending**: Layer animations with reference poses
  - Weapon recoil, breathing, damage reactions
  - Local and world-space modes
  - Weight-based blending

- **Root Motion Extraction**: Extract character movement from animations
  - Precise character controller movement
  - Translation and rotation extraction
  - Frame-rate independent motion
