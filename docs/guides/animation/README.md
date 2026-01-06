# Animation Guides

Comprehensive documentation for the Praxis skeletal animation system.

**Looking for a structured learning path?** See [Animation Learning Path](../../learning-paths/animation.md) for a beginner → intermediate → advanced progression with exercises and time estimates.

## Overview

The Praxis animation system provides a complete solution for character animation in games, from basic skeletal animation to advanced features like IK, retargeting, and root motion extraction.

## Documentation Structure

### [Skeletal Basics](skeletal-basics.md)

Learn the fundamentals of skeletal animation in Praxis:

- **Core Architecture**: Component hierarchy, skeleton structure, and bone transforms
- **Animation Data Structures**: AnimationClip, BoneTrack, and AnimatedPose
- **Keyframe Interpolation**: LERP, SLERP, and keyframe sampling algorithms
- **GLTF Workflow**: Loading and using industry-standard animated models
- **Performance Considerations**: Memory layout, optimization strategies, and scalability

**Start here if:** You're new to the animation system or want to understand how it works under the hood.

### [Blending](blending.md)

Master animation blending techniques:

- **Cross-Fade Transitions**: Smooth transitions between animation states
- **1D Blend Trees**: Speed-based parameter blending (idle → walk → run)
- **2D Blend Trees**: Directional movement blending with inverse distance weighting
- **Layered Animation**: Play different animations on different body parts with bone masking
- **Additive Blending**: Layer subtle effects on top of base animations
- **Implementation Details**: Evaluation pipeline and performance optimization

**Start here if:** You want to create smooth, responsive character movement and transitions.

### [Advanced Features](advanced-features.md)

Explore advanced animation capabilities:

### [Advanced Integration](advanced-integration.md)

Integration with other engine systems:

- Physics-animation integration
- Scripting-animation integration
- Complete character controller examples

### Quick References

- **[Quick Start](quick-start.md)** - Get up and running quickly
- **[Quick Reference](quick-reference.md)** - Common patterns cheat sheet

### [Skeletal Animation](skeletal-animation.md)

Complete skeletal animation system guide:



- **Inverse Kinematics (IK)**: Procedural limb positioning for adaptive interaction
  - Two-bone IK for arms and legs
  - Chain IK for spines and tails
  - Look-at IK for head tracking
- **Animation Retargeting**: Share animations across different character skeletons
- **Additive Animation**: Layer effects like recoil, breathing, and reactions
- **Root Motion Extraction**: Extract precise character movement from animations

**Start here if:** You need advanced features like foot IK on terrain, weapon aiming, or precise character movement.

## Quick Navigation

### By Task

**I want to...**
- Load an animated character → [GLTF Workflow](skeletal-basics.md#gltf-animation-workflow)
- Smoothly transition between animations → [Cross-Fade Transitions](blending.md#cross-fade-transitions)
- Create speed-based movement → [1D Blend Trees](blending.md#1d-blend-trees)
- Play walking + aiming simultaneously → [Layered Animation](blending.md#layered-animation)
- Make feet stick to terrain → [Two-Bone IK](advanced-features.md#two-bone-ik)
- Use mocap on my character → [Animation Retargeting](advanced-features.md#animation-retargeting)
- Add weapon recoil → [Additive Blending](advanced-features.md#additive-animation-blending)
- Get movement from animation → [Root Motion](advanced-features.md#root-motion-extraction)

### By Experience Level

**Beginner**: Start with [Skeletal Basics](skeletal-basics.md) → Read [Basic Usage Examples](skeletal-basics.md#basic-usage-examples)

**Intermediate**: Review [Blending](blending.md) → Try [1D and 2D Blend Trees](blending.md#blend-trees)

**Advanced**: Dive into [Advanced Features](advanced-features.md) → Use [IK](advanced-features.md#inverse-kinematics-ik) and [Retargeting](advanced-features.md#animation-retargeting)

## Examples

Working code examples demonstrating animation features:

```bash
# Basic skeletal animation
cargo run --example skeletal_animation_demo

# Animation blending and transitions
cargo run --example animation_blending_demo

# Advanced features (IK, retargeting, root motion)
cargo run --example animation_advanced_demo
```

## Related Documentation

- **[Animation Guide](../animation.md)**: Quick start guide with practical examples
- **[Animation Concepts](../../concepts/animation.md)**: Theory and conceptual overview
- **[praxis_scene](../../../crates/praxis_scene/README.md)**: API documentation

## Quick Reference

### Core Components

```rust
Skeleton          // Bone hierarchy and bind poses
AnimationPlayer   // Basic playback control
AnimationBlender  // Advanced blending features
AnimatedPose      // Computed bone transforms
IkController      // Inverse kinematics
```

### Common Patterns

**Simple playback:**
```rust
player.play("Walk");
player.set_looping(true);
```

**Cross-fade transition:**
```rust
blender.cross_fade("Idle", "Walk", 0.3);
```

**Speed-based blending:**
```rust
blender.set_blend_parameter("Movement", speed);
```

**Foot IK:**
```rust
ik.add_constraint(IkConstraint::new_two_bone(foot_bone, ground_pos));
```

---

## Getting Help

- Check the [Troubleshooting](advanced-features.md#troubleshooting) section
- Review [Performance Considerations](skeletal-basics.md#performance-considerations)
- Look at working examples in `examples/`
- Read API docs: `cargo doc --open`
