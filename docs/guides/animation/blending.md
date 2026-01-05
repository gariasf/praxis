# Animation Blending

This guide covers animation blending techniques in Praxis, including cross-fade transitions, blend trees, layered animation, and additive blending.

## Table of Contents

1. [Overview](#overview)
2. [Basic Animation Blending](#basic-animation-blending)
3. [Cross-Fade Transitions](#cross-fade-transitions)
4. [Blend Trees](#blend-trees)
5. [Layered Animation](#layered-animation)
6. [Additive Blending](#additive-blending)
7. [Implementation Details](#implementation-details)
8. [Examples](#examples)

---

## Overview

The animation blending system allows multiple animations to play simultaneously and blend smoothly between states. This enables:

- **Smooth transitions** between different animation states
- **Parameter-driven blending** for responsive character movement
- **Multi-layer animation** for playing animations on different body parts
- **Additive effects** for layering subtle animations on top of base animations

The `AnimationBlender` component provides advanced blending features beyond the basic `AnimationPlayer`.

---

## Basic Animation Blending

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

---

## Cross-Fade Transitions

Cross-fades provide smooth transitions between animations over a specified duration.

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

### Usage

```rust
let mut blender = AnimationBlender::new();
blender.add_clip("Idle", idle_clip);
blender.add_clip("Walk", walk_clip);

blender.play("Idle");
// ... later
blender.cross_fade("Idle", "Walk", 0.3);  // 0.3 second transition
```

### Implementation

Cross-fade transitions are tracked by the `CrossFadeTransition` struct:

```rust
struct CrossFadeTransition {
    from: String,           // Source animation name
    to: String,             // Target animation name
    duration: f32,          // Total transition time
    elapsed: f32,           // Time elapsed
    from_time: f32,         // Starting time in source
    to_time: f32,           // Starting time in target
}

impl CrossFadeTransition {
    fn blend_weight(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }
    
    fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }
    
    fn update(&mut self, delta_time: f32) {
        self.elapsed += delta_time;
    }
}
```

---

## Blend Trees

Blend trees provide parameter-driven blending for smooth transitions between multiple animations.

### 1D Blend Trees

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

### 2D Blend Trees

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

### Blend Tree Implementation

```rust
pub struct BlendNode1D {
    clips: Vec<(String, f32)>,  // (name, parameter value)
    parameter: f32,
}

impl BlendNode1D {
    pub fn compute_weights(&self) -> Vec<(String, f32)> {
        // Find two nearest clips
        let mut before = None;
        let mut after = None;
        
        for (name, value) in &self.clips {
            if *value <= self.parameter {
                before = Some((name.clone(), *value));
            }
            if *value >= self.parameter && after.is_none() {
                after = Some((name.clone(), *value));
            }
        }
        
        match (before, after) {
            (Some((name1, val1)), Some((name2, val2))) if val1 != val2 => {
                let t = (self.parameter - val1) / (val2 - val1);
                vec![
                    (name1, 1.0 - t),
                    (name2, t),
                ]
            }
            // ... handle edge cases
        }
    }
}
```

---

## Layered Animation

Layered animation allows multiple animations to play on different parts of the skeleton simultaneously.

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

### Usage

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

### Bone Masks

Bone masks control which bones are affected by a layer:

```rust
pub struct BoneMask {
    enabled: Vec<bool>,
}

impl BoneMask {
    pub fn with_bone_count(count: usize) -> Self {
        Self {
            enabled: vec![false; count],
        }
    }
    
    pub fn enable_bone(&mut self, bone_index: usize) {
        if bone_index < self.enabled.len() {
            self.enabled[bone_index] = true;
        }
    }
    
    pub fn enable_bone_and_children_with_skeleton(
        &mut self,
        bone_index: usize,
        skeleton: &Skeleton,
    ) {
        self.enable_bone(bone_index);
        
        // Recursively enable children
        for i in 0..skeleton.bone_count() {
            if let Some(bone) = skeleton.bone(i) {
                if bone.parent_index == Some(bone_index) {
                    self.enable_bone_and_children_with_skeleton(i, skeleton);
                }
            }
        }
    }
    
    pub fn is_bone_enabled(&self, bone_index: usize) -> bool {
        self.enabled.get(bone_index).copied().unwrap_or(false)
    }
}
```

### Animation Layers

```rust
pub struct AnimationLayer {
    weight: f32,
    mask: Option<BoneMask>,
    blend_mode: LayerBlendMode,
    current_clip: Option<String>,
    time: f32,
    speed: f32,
    looping: bool,
}

pub enum LayerBlendMode {
    Override,  // Replace base pose
    Additive,  // Add delta to base pose
}
```

---

## Additive Blending

Additive blending adds animation deltas on top of a base animation:

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

### Usage with Blend Nodes

```rust
let mut additive_node = AdditiveBlendNode::new();
additive_node.set_base("Walk");
additive_node.set_additive("Recoil");
additive_node.set_weight(1.0);

blender.add_blend_tree("CombatMovement", additive_node.into());
blender.activate_blend_tree("CombatMovement");
```

### Additive Mathematics

For each transform component:

**Translation:**
```
delta = additive_translation - reference_translation
final = base_translation + delta × weight
```

**Rotation:**
```
delta = reference_rotation^-1 × additive_rotation
final = base_rotation × slerp(identity, delta, weight)
```

**Scale:**
```
delta = additive_scale / reference_scale
final = base_scale × lerp(1.0, delta, weight)
```

---

## Implementation Details

### Animation Evaluation Pipeline

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

3. Evaluate Base Layer
   - Cross-fade: Blend between two animations
   - Blend Tree: Evaluate and blend multiple animations
   - Single Clip: Apply single animation

4. Evaluate Additional Layers
   for each layer in layers:
     if layer has mask:
       apply only to masked bones
     if layer.blend_mode == Override:
       blend with layer weight
     elif layer.blend_mode == Additive:
       add delta with layer weight

5. Propagate Transforms (Hierarchy)
   for each bone in depth-first order:
     if bone has parent:
       world_transform[bone] = world_transform[parent] × local_transform[bone]
     else:
       world_transform[bone] = local_transform[bone]

6. Compute Skinning Matrices
   for each bone:
     skinning_matrix[bone] = world_transform[bone] × inverse_bind_matrix[bone]

7. Upload to GPU
   Update uniform buffer with skinning_matrices
   Bind to vertex shader for skinning
```

### Blending System Integration

```rust
use praxis_ecs::Query;
use praxis_scene::{Skeleton, AnimationBlender, AnimatedPose, update_animation_blenders};

fn blending_system(
    mut query: Query<(&Skeleton, &mut AnimationBlender, &mut AnimatedPose)>
) {
    let delta_time = 0.016; // From timing system
    update_animation_blenders(delta_time, &mut query);
}
```

### Performance Optimizations

1. **Weight Filtering**: Clips with very small weights (<0.001) are skipped during blending
2. **2D Blend Filtering**: Clips with weights <0.01 are filtered out and remaining weights renormalized
3. **Direct Assignment**: When weight is very close to 1.0 (≥0.999), transforms are directly assigned without blending
4. **Bone Masking**: Layers only process bones enabled in their mask

---

## Examples

### Example 1: Cross-Fade Transition

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

### Example 2: 1D Blend Tree

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

### Example 3: 2D Blend Tree

```rust
use praxis_scene::BlendNode2D;

let mut blend_tree = BlendNode2D::new();
blend_tree.add_clip("Forward", 0.0, 1.0);
blend_tree.add_clip("Back", 0.0, -1.0);
blend_tree.add_clip("Left", -1.0, 0.0);
blend_tree.add_clip("Right", 1.0, 0.0);

blender.add_blend_tree("Locomotion", blend_tree.into());
blender.activate_blend_tree("Locomotion");
blender.set_blend_parameters_2d("Locomotion", 0.5, 0.5);
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

### Example 5: Complete Animation System

```rust
use praxis_scene::*;
use praxis_ecs::World;

let skeleton = load_skeleton();
let mut blender = AnimationBlender::new();

// Add animations
blender.add_clip("Idle", idle_clip);
blender.add_clip("Walk", walk_clip);
blender.add_clip("Run", run_clip);

// Setup 1D blend tree for movement
let mut movement_tree = BlendNode1D::new();
movement_tree.add_clip("Idle", 0.0);
movement_tree.add_clip("Walk", 0.5);
movement_tree.add_clip("Run", 1.0);
blender.add_blend_tree("Movement", movement_tree.into());

// Start with idle
blender.activate_blend_tree("Movement");
blender.set_blend_parameter("Movement", 0.0);

// Spawn entity
let mut world = World::new();
let pose = AnimatedPose::new(skeleton.bone_count());
world.spawn((skeleton, blender, pose));

// In game loop, change speed based on input
blender.set_blend_parameter("Movement", player_speed);
```

---

## Performance Considerations

### Blending Costs

```text
Performance Impact
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Operation                    Cost (CPU)    Notes
──────────────────────────────────────────────────────────────
Cross-fade (2 clips)         ~5-10 µs      50-bone character
1D blend tree (3 clips)      ~8-15 µs      50-bone character
2D blend tree (4 clips)      ~10-20 µs     50-bone character
Layer with mask              ~3-8 µs       Per layer, masked bones
Additive blending            ~4-10 µs      Additional 20-30% overhead

Example: 50-bone character with 2-clip blend tree
  - Base evaluation: ~2-4 µs
  - Blend tree eval: ~8-15 µs
  - Total: ~10-19 µs

At 60 FPS with 100 blended characters:
  - Animation update: 1.0-1.9 ms
  - ~6-11% of 16.67ms frame budget
```

### Scalability Guidelines

```text
Scalability for Blending
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Number of Active Clips:
  ✓ 1-3 clips: Excellent (typical blending)
  ✓ 3-8 clips: Good (complex blend trees)
  ⚠ 8-16 clips: Acceptable (large blend spaces)
  ✗ >16 clips: Optimize (reduce active clips)

Number of Layers:
  ✓ 1-2 layers: Excellent (base + upper body)
  ✓ 2-4 layers: Good (complex layering)
  ⚠ 4-8 layers: Acceptable (very detailed)
  ✗ >8 layers: Optimize (consider alternatives)

Optimization Tips:
  - Use weight thresholds to skip negligible contributions
  - Prefer 1D blend trees over 2D when possible
  - Use bone masks to limit layer processing
  - Cache blend tree weights when parameters don't change
```

---

## Next Steps

- **[Skeletal Basics](skeletal-basics.md)**: Review core animation concepts
- **[Advanced Features](advanced-features.md)**: Explore IK, retargeting, additive blending, and root motion
- **[Examples](../../examples/)**: See `animation_blending_demo.rs` for complete examples
