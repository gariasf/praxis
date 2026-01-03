# Animation Blending System Implementation

## Overview

This document describes the implementation of the comprehensive animation blending system for the Praxis game engine. The system provides advanced animation features including cross-fade transitions, blend trees, layered animation, and additive blending.

## Architecture

### Core Components

#### AnimationBlender
The main component that manages all animation blending features:
- **Animation Library**: Stores all available animation clips
- **Base Layer**: Primary animation playback (layer 0)
- **Cross-Fade Transitions**: Smooth transitions between animations
- **Blend Trees**: Parameter-driven animation blending (1D, 2D, additive)
- **Animation Layers**: Multiple simultaneous animations with bone masking

### Features

#### 1. Cross-Fade Transitions

**Purpose**: Smooth transitions between two animations over a specified duration.

**Implementation**:
- `CrossFadeTransition` struct tracks:
  - Source and target animations
  - Transition duration
  - Elapsed time
  - Starting playback times
- Blend weight computed as `elapsed / duration`
- Animations blended using lerp/slerp based on weight

**Usage**:
```rust
blender.cross_fade("Walk", "Run", 0.3); // 0.3 second transition
```

**Key Methods**:
- `blend_weight()`: Returns current blend weight (0.0 to 1.0)
- `is_complete()`: Checks if transition finished
- `update(delta_time)`: Advances transition time

#### 2. Blend Trees

**Purpose**: Parameter-driven blending for smooth transitions between multiple animations.

##### BlendNode1D
1D blend space for single-parameter blending (e.g., speed).

**Implementation**:
- Stores clips with their parameter values
- Finds two nearest clips based on current parameter
- Computes linear blend weights between them

**Usage**:
```rust
let mut blend_tree = BlendNode1D::new();
blend_tree.add_clip("Idle", 0.0);
blend_tree.add_clip("Walk", 0.5);
blend_tree.add_clip("Run", 1.0);

blender.set_blend_parameter("Movement", 0.75); // Blend between Walk and Run
```

##### BlendNode2D
2D blend space for dual-parameter blending (e.g., directional movement).

**Implementation**:
- Stores clips with 2D positions
- Uses inverse distance weighting for blending
- Filters out clips with very small weights (<0.01)
- Normalizes weights to sum to 1.0

**Usage**:
```rust
let mut blend_tree = BlendNode2D::new();
blend_tree.add_clip("Forward", 0.0, 1.0);
blend_tree.add_clip("Back", 0.0, -1.0);
blend_tree.add_clip("Left", -1.0, 0.0);
blend_tree.add_clip("Right", 1.0, 0.0);

blender.set_blend_parameters_2d("Locomotion", 0.7, 0.7); // Forward-right
```

##### AdditiveBlendNode
Additive blending for layering animations on top of base animations.

**Implementation**:
- Maintains base animation and additive animation
- Weight controls additive contribution (0.0 to 1.0)
- Additive animation delta added to base animation

**Usage**:
```rust
let mut additive = AdditiveBlendNode::new();
additive.set_base("Walk");
additive.set_additive("Recoil");
additive.set_weight(1.0);
```

#### 3. Layered Animation

**Purpose**: Play multiple animations simultaneously on different parts of the skeleton.

**Components**:
- `AnimationLayer`: Individual layer with weight, mask, and blend mode
- `BoneMask`: Controls which bones are affected by a layer
- `LayerBlendMode`: Override or Additive blending

**Implementation**:
- Each layer has independent playback state
- Bone masks filter which bones are affected
- Layers evaluated in order after base layer
- Final transforms accumulated through blending

**Usage**:
```rust
// Base layer: full body walk
blender.play("Walk");

// Layer 1: upper body aim
let mut upper_body_mask = BoneMask::with_bone_count(skeleton.bone_count());
upper_body_mask.enable_bone_and_children_with_skeleton(spine_index, &skeleton);

let mut layer = AnimationLayer::new(1.0);
layer.set_mask(upper_body_mask);
layer.set_blend_mode(LayerBlendMode::Override);

blender.add_layer(layer);
blender.play_on_layer(0, "Aim");
```

### Animation Evaluation Pipeline

1. **Initialize Pose**: Start with skeleton bind pose
2. **Evaluate Base Layer**:
   - Cross-fade: Blend between two animations
   - Blend Tree: Evaluate and blend multiple animations
   - Single Clip: Apply single animation
3. **Evaluate Layers**: Apply each layer with masking and blending
4. **Update Transforms**: Compute world transforms and skinning matrices

### Blending Mathematics

#### Transform Blending
For each bone, transforms are decomposed into Translation, Rotation, Scale (TRS):
- **Translation**: Linear interpolation (lerp)
  ```
  result = a + (b - a) * weight
  ```
- **Rotation**: Spherical linear interpolation (slerp)
  ```
  result = slerp(a, b, weight)
  ```
- **Scale**: Linear interpolation (lerp)
  ```
  result = a + (b - a) * weight
  ```

#### 2D Blend Weights (Inverse Distance Weighting)
```
weight_i = 1 / distance_squared_i
normalized_weight_i = weight_i / sum(all_weights)
```

## API Reference

### AnimationBlender

#### Creation
```rust
let blender = AnimationBlender::new();
```

#### Clip Management
```rust
blender.add_clip("Walk", walk_clip);
blender.clip("Walk"); // Get clip reference
```

#### Basic Playback
```rust
blender.play("Walk");
blender.set_speed(1.5);
blender.set_looping(true);
```

#### Cross-Fade
```rust
blender.cross_fade("Idle", "Walk", 0.3);
```

#### Blend Trees
```rust
// 1D
blender.add_blend_tree("Movement", blend_node_1d.into());
blender.activate_blend_tree("Movement");
blender.set_blend_parameter("Movement", 0.5);

// 2D
blender.add_blend_tree("Locomotion", blend_node_2d.into());
blender.activate_blend_tree("Locomotion");
blender.set_blend_parameters_2d("Locomotion", 0.5, 0.5);
```

#### Layers
```rust
blender.add_layer(layer);
blender.play_on_layer(0, "Wave");
blender.layer(0); // Get layer reference
blender.layer_mut(0); // Get mutable layer reference
```

#### Query Methods
```rust
blender.current_clip(); // Current base clip
blender.current_time(); // Current base time
blender.is_cross_fading(); // Check if cross-fading
blender.active_blend_tree(); // Active blend tree name
blender.layer_count(); // Number of layers
```

#### Update
```rust
blender.update(delta_time);
let pose = blender.evaluate(&skeleton);
```

### BlendNode1D

```rust
let mut node = BlendNode1D::new();
node.add_clip("Idle", 0.0);
node.add_clip("Walk", 0.5);
node.add_clip("Run", 1.0);
node.set_parameter(0.75);
let weights = node.compute_weights(); // Vec<(String, f32)>
```

### BlendNode2D

```rust
let mut node = BlendNode2D::new();
node.add_clip("Forward", 0.0, 1.0);
node.add_clip("Back", 0.0, -1.0);
node.add_clip("Left", -1.0, 0.0);
node.add_clip("Right", 1.0, 0.0);
node.set_parameters(0.5, 0.5);
let weights = node.compute_weights(); // Vec<(String, f32)>
```

### BoneMask

```rust
let mut mask = BoneMask::with_bone_count(skeleton.bone_count());
mask.enable_bone(1);
mask.disable_bone(2);
mask.enable_bone_and_children_with_skeleton(3, &skeleton);
let is_enabled = mask.is_bone_enabled(1); // true/false
let weight = mask.bone_weight(1); // 0.0 or 1.0
```

### AnimationLayer

```rust
let mut layer = AnimationLayer::new(1.0); // Weight
layer.set_mask(bone_mask);
layer.set_blend_mode(LayerBlendMode::Override);
layer.play("Wave");
layer.set_speed(1.5);
layer.set_looping(true);

// Query
layer.current_clip(); // Option<&str>
layer.time(); // f32
layer.weight(); // f32
```

## System Integration

### ECS System

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

### Spawning Entity with Blender

```rust
use praxis_ecs::World;
use praxis_scene::{Skeleton, AnimationBlender, AnimatedPose};

let skeleton = create_skeleton();
let mut blender = AnimationBlender::new();
blender.add_clip("Idle", idle_clip);
blender.add_clip("Walk", walk_clip);
blender.play("Idle");

let pose = AnimatedPose::new(skeleton.bone_count());

world.spawn((skeleton, blender, pose));
```

## Example

See `examples/animation_blending_demo.rs` for a comprehensive demonstration of all features:
- Cross-fade transitions
- 1D blend trees (speed-based)
- 2D blend trees (directional)
- Layered animation with bone masking
- Additive blending

Run with:
```bash
cargo run --example animation_blending_demo
```

## Performance Considerations

### Optimization Strategies

1. **Weight Filtering**: Clips with very small weights (<0.001) are skipped during blending
2. **2D Blend Filtering**: Clips with weights <0.01 are filtered out and remaining weights renormalized
3. **Direct Assignment**: When weight is very close to 1.0 (≥0.999), transforms are directly assigned without blending
4. **Bone Masking**: Layers only process bones enabled in their mask
5. **Transform Decomposition**: Cached during blending to avoid repeated calculations

### Memory Usage

- Each AnimationBlender stores a copy of all clips it uses
- Layers are lightweight (mainly state + optional mask)
- BoneMasks use `Vec<bool>` for enabled bones
- Cross-fade stores minimal state (just two clip names and timing info)

## Future Enhancements

Potential improvements for future development:

1. **Animation State Machines**: Formal state machine system with transition rules
2. **IK (Inverse Kinematics)**: Procedural animation for limb positioning
3. **Animation Events**: Trigger events at specific keyframes
4. **Animation Curves**: Custom blending curves for cross-fades
5. **Root Motion**: Extract and apply root bone movement to entity transform
6. **Blend Space Triangulation**: More accurate 2D blending using Delaunay triangulation
7. **Animation Compression**: Reduce memory footprint of animation data
8. **GPU Skinning**: Move bone transform computation to GPU

## Testing

The animation blending system includes comprehensive unit tests:

```bash
cargo test -p praxis_scene
```

Tests cover:
- Cross-fade transition weight calculation
- 1D blend weight computation
- 2D blend weight computation
- Bone mask enable/disable
- Animation layer creation and configuration
- AnimationBlender basic operations

## Documentation

Full API documentation available via:
```bash
cargo doc --open
```

Navigate to `praxis_scene::animation` module for detailed API docs.
