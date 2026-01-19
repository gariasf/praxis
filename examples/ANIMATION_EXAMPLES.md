# Animation Examples - Learning Guide

This document explains the consolidated animation example suite and provides a recommended learning path.

## Overview

The animation examples have been streamlined into **three focused examples** covering beginner, intermediate, and advanced use cases. This consolidation eliminates redundancy while maintaining comprehensive coverage of the animation system.

## Consolidation Changes

### What Was Removed
- **`animation_demo.rs`** - Removed as it was redundant with `animation_blending_demo.rs`

### Why It Was Removed
The original `animation_demo.rs` covered:
- 10-bone humanoid skeleton
- Cross-fade transitions
- 1D blend trees

All of these features are also covered in `animation_blending_demo.rs`, which additionally includes:
- 2D blend trees
- Layered animation with bone masking
- Additive blending via blend tree nodes

Since `animation_blending_demo.rs` is a superset of `animation_demo.rs`, we removed the simpler version to reduce maintenance burden and avoid confusion about which example to use.

## Recommended Learning Path

### 1. Skeletal Animation Demo (Beginner) ⭐ START HERE

**File:** `skeletal_animation_demo.rs`

**What You'll Learn:**
- Core skeletal animation concepts
- Creating skeletons with bone hierarchies
- Defining animation clips with keyframes
- Using the `AnimationPlayer` API
- Animation playback control (play, stop, loop)
- Simple weight-based blending between animations

**Key Features:**
- Simple 3-bone skeleton (Root, Spine, Head)
- Two basic animations (Walk and Idle)
- Visual bone markers for understanding hierarchy
- Interactive controls to switch animations

**Run It:**
```bash
cargo run --example skeletal_animation_demo
```

**Controls:**
- `1` - Play Walk animation
- `2` - Play Idle animation  
- `3` - Play both animations blended
- `WASD` - Move camera
- `Space/Ctrl` - Move up/down
- `Mouse` - Look around
- `ESC` - Exit

**Code Highlights:**
```rust
// Simple AnimationPlayer usage
let mut player = AnimationPlayer::new();
player.add_clip("Walk", create_walk_animation());
player.play("Walk");
player.set_looping("Walk", true);
```

### 2. Animation Blending Demo (Intermediate)

**File:** `animation_blending_demo.rs`

**What You'll Learn:**
- Production-ready blending techniques
- Cross-fade transitions between animations
- 1D blend trees for parameter-based blending (speed)
- 2D blend trees for directional movement
- Layered animation with bone masking
- Additive blending for combining animations

**Key Features:**
- 10-bone humanoid skeleton
- Multiple animations (Idle, Walk, Run, Wave, directional movement)
- Uses the `AnimationBlender` API (more powerful than `AnimationPlayer`)
- Interactive parameter adjustment
- Visual demonstration of all blending modes

**Run It:**
```bash
cargo run --example animation_blending_demo
```

**Controls:**
- `1` - Idle animation
- `2` - Walk animation
- `3` - Run animation
- `4` - Cross-fade to next animation
- `5` - 1D blend tree (speed-based)
- `6` - 2D blend tree (directional)
- `7` - Toggle layered animation (wave + walk)
- `8` - Toggle additive blending
- `Arrow Keys` - Adjust blend parameters
- `WASD` - Move camera
- `ESC` - Exit

**Code Highlights:**
```rust
// AnimationBlender with blend trees
let mut blender = AnimationBlender::new();

// 1D blend tree for speed
let mut blend_tree_1d = BlendNode1D::new();
blend_tree_1d.add_clip("Idle", 0.0);
blend_tree_1d.add_clip("Walk", 0.5);
blend_tree_1d.add_clip("Run", 1.0);
blender.add_blend_tree("SpeedBlend", blend_tree_1d.into());

// 2D blend tree for directional movement
let mut blend_tree_2d = BlendNode2D::new();
blend_tree_2d.add_clip("Forward", 0.0, 1.0);
blend_tree_2d.add_clip("Backward", 0.0, -1.0);
blend_tree_2d.add_clip("Left", -1.0, 0.0);
blend_tree_2d.add_clip("Right", 1.0, 0.0);

// Cross-fade between animations
blender.cross_fade("Walk", "Run", 0.3);
```

### 3. Animation Advanced Demo (Advanced)

**File:** `animation_advanced_demo.rs`

**What You'll Learn:**
- Inverse Kinematics (IK) for procedural limb positioning
- Animation retargeting between different skeletons
- Manual additive animation with reference poses
- Advanced animation techniques for interactive environments

**Key Features:**
- Three separate demonstrations running side-by-side
- IK: Arm reaching for a moving target
- Retargeting: Animations transferred between skeletons with different proportions
- Additive: Manual additive blending (Walk base + Recoil additive)

**Run It:**
```bash
cargo run --example animation_advanced_demo
```

**Controls:**
- `1` - Focus on IK demo (left character)
- `2` - Focus on Retargeting demo (center character)
- `3` - Focus on Additive demo (right character)
- `WASD` - Move camera
- `ESC` - Exit

**Code Highlights:**
```rust
// Inverse Kinematics
let mut ik_controller = IkController::new();
let constraint = IkConstraint::new_two_bone(end_bone, target_pos);
ik_controller.add_constraint(constraint);
ik_controller.apply(&mut pose, &skeleton);

// Animation Retargeting
let retargeter = AnimationRetargeter::auto(&source_skeleton, &target_skeleton);
let retargeted_clip = retargeter.retarget_clip(&source_clip, &target_skeleton);

// Additive Animation
let mut additive = AdditiveAnimation::new("Walk", "Recoil")
    .with_weight(1.0)
    .with_mode(AdditiveMode::Local);
additive.apply(&mut pose, &recoil_clip, time, &skeleton);
```

## Additional Resources

### GLTF Animation Loader Demo

**File:** `gltf_animation_loader_demo.rs`

For real-world asset integration, this example shows how to load animations from GLTF files:

```bash
cargo run --example gltf_animation_loader_demo
```

## API Comparison: AnimationPlayer vs AnimationBlender

### AnimationPlayer (Beginner)
- **Use When:** Simple animation playback is sufficient
- **Features:**
  - Play/stop individual clips
  - Basic weight-based blending
  - Animation looping
  - Simple API, easy to understand

```rust
let mut player = AnimationPlayer::new();
player.add_clip("Walk", walk_clip);
player.play("Walk");
player.set_weight("Walk", 0.7);
```

### AnimationBlender (Intermediate/Advanced)
- **Use When:** You need production-ready animation systems
- **Features:**
  - Everything AnimationPlayer has
  - Cross-fade transitions
  - 1D and 2D blend trees
  - Layered animation with bone masking
  - Additive blending nodes
  - Blend tree parameter control

```rust
let mut blender = AnimationBlender::new();
blender.add_clip("Walk", walk_clip);

// Cross-fade
blender.cross_fade("Walk", "Run", 0.3);

// Blend trees
blender.activate_blend_tree("SpeedBlend");
blender.set_blend_parameter("SpeedBlend", 0.5);

// Layers
blender.add_layer(upper_body_layer);
blender.play_on_layer(0, "Wave");
```

## Common Questions

### Q: Which example should I start with?
**A:** Always start with `skeletal_animation_demo.rs`. It covers the fundamentals you'll need for all other examples.

### Q: Can I skip the intermediate example?
**A:** No. The concepts in `animation_blending_demo.rs` (blend trees, cross-fades, layering) are essential for production-quality animation systems. The advanced example builds on these.

### Q: What if I only need basic animations?
**A:** Use `AnimationPlayer` (from the beginner example). It's simpler and has less overhead for basic use cases.

### Q: When do I need IK or retargeting?
**A:** 
- **IK:** When characters need to interact with the environment (reaching for objects, foot placement on uneven terrain)
- **Retargeting:** When sharing animations between characters with different skeleton proportions

### Q: What happened to the old animation_demo.rs?
**A:** It was consolidated into `animation_blending_demo.rs` to reduce redundancy. All its features are present in the blending demo.

## Documentation References

For deeper understanding of the animation system, see:
- `docs/guides/animation/skeletal-basics.md` - Skeletal animation fundamentals
- `docs/guides/animation/blending.md` - Blend trees and cross-fading
- `docs/guides/animation/advanced-features.md` - IK, retargeting, and more
- `crates/praxis_scene/README.md` - Animation system API reference

## Summary

The three-tier structure provides a clear learning progression:
1. **Beginner:** Learn the fundamentals with simple examples
2. **Intermediate:** Master production techniques with comprehensive blending
3. **Advanced:** Explore procedural techniques for dynamic animation

Each example builds on the previous one, ensuring a smooth learning curve while covering the full spectrum of animation capabilities.
