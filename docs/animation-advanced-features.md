# Advanced Animation Features

This document covers the advanced animation features in Praxis, including inverse kinematics (IK), animation retargeting, additive animation blending, and root motion extraction.

## Table of Contents

1. [Inverse Kinematics (IK)](#inverse-kinematics-ik)
2. [Animation Retargeting](#animation-retargeting)
3. [Additive Animation Blending](#additive-animation-blending)
4. [Root Motion Extraction](#root-motion-extraction)

---

## Inverse Kinematics (IK)

Inverse Kinematics allows you to procedurally position limbs by specifying target positions for end effectors (hands, feet, etc.), and the system automatically computes the required bone rotations.

### IK Constraint Types

#### Two-Bone IK

Used for arms and legs with three bones (shoulder-elbow-hand or hip-knee-foot).

```rust
use praxis_scene::{IkConstraint, IkController};
use praxis_math::Vec3;

// Create a two-bone IK constraint
let constraint = IkConstraint::new_two_bone(
    hand_bone_index,
    Vec3::new(2.0, 1.5, 0.0)  // Target position
)
.with_pole_target(Vec3::new(0.0, 1.0, 1.0))  // Control bend direction
.with_weight(1.0);  // Full weight

// Add to IK controller
let mut ik_controller = IkController::new();
ik_controller.add_constraint(constraint);

// Apply to pose (in your animation system)
ik_controller.apply(&mut pose, &skeleton);
```

**Key Features:**
- Uses analytic solution (fast and stable)
- Pole target for controlling elbow/knee direction
- Weight control for blending with animation
- Clamps to maximum reach distance

#### Chain IK

Used for multi-bone chains like spines, tails, or tentacles.

```rust
// Create a chain IK constraint using FABRIK algorithm
let constraint = IkConstraint::new_chain(
    end_bone_index,
    Vec3::new(3.0, 2.0, 1.0),
    20  // Max iterations
);

ik_controller.add_constraint(constraint);
```

**Key Features:**
- FABRIK (Forward And Backward Reaching Inverse Kinematics) algorithm
- Iterative solver with convergence threshold
- Works with arbitrary chain lengths
- Maintains bone lengths

#### Look-At IK

Used for head tracking, eye movement, or aiming.

```rust
// Create a look-at constraint
let constraint = IkConstraint::new_look_at(
    head_bone_index,
    Vec3::new(5.0, 2.0, 3.0)  // Look target
);

ik_controller.add_constraint(constraint);
```

**Key Features:**
- Single-bone orientation toward target
- Fast single-iteration solution
- Useful for cameras, turrets, eyes

### IK System Integration

```rust
use praxis_scene::{apply_ik_constraints, IkController, Skeleton, AnimatedPose};
use praxis_ecs::Query;

// In your ECS system, apply IK after animation evaluation
fn my_animation_system(
    mut query: Query<(&Skeleton, &IkController, &mut AnimatedPose)>
) {
    // First evaluate animations
    // ...
    
    // Then apply IK constraints
    apply_ik_constraints(&mut query);
}
```

### Real-World Examples

#### Foot IK for Uneven Terrain

```rust
// Left foot
let left_foot_constraint = IkConstraint::new_two_bone(
    left_foot_bone,
    left_ground_position
)
.with_pole_target(left_knee_hint);

// Right foot
let right_foot_constraint = IkConstraint::new_two_bone(
    right_foot_bone,
    right_ground_position
)
.with_pole_target(right_knee_hint);

ik_controller.add_constraint(left_foot_constraint);
ik_controller.add_constraint(right_foot_constraint);
```

#### Hand IK for Holding Objects

```rust
// Character grabs a door handle
let hand_constraint = IkConstraint::new_two_bone(
    right_hand_bone,
    door_handle_position
)
.with_weight(1.0);  // Full control

ik_controller.add_constraint(hand_constraint);
```

---

## Animation Retargeting

Animation retargeting allows you to apply animations created for one skeleton to a different skeleton with a potentially different bone structure.

### Automatic Bone Mapping

The system can automatically map bones between skeletons based on bone names:

```rust
use praxis_scene::{AnimationRetargeter, BoneMapping};

// Automatic mapping (case-insensitive name matching)
let retargeter = AnimationRetargeter::auto(&source_skeleton, &target_skeleton);

// Retarget an animation clip
let retargeted_clip = retargeter.retarget_clip(&walk_animation, &target_skeleton);
```

**Name Matching Rules:**
1. Exact match (case-insensitive)
2. Substring match (e.g., "LeftArm" matches "leftarm")
3. Contains match (e.g., "Left_Arm_01" contains "leftarm")

### Manual Bone Mapping

For more control, create manual bone mappings:

```rust
let mut mapping = BoneMapping::new();

// Map by bone index
mapping.map_bones(0, 0);  // Source bone 0 -> Target bone 0
mapping.map_bones(1, 2);  // Source bone 1 -> Target bone 2

// Map by bone name
mapping.map_bone_names("LeftArm".to_string(), "L_Arm".to_string());

// Create retargeter
let retargeter = AnimationRetargeter::new(mapping);
```

### Retargeting Animation Clips

```rust
// Retarget entire clip
let retargeted_clip = retargeter.retarget_clip(&source_clip, &target_skeleton);

// The retargeted clip preserves:
// - Animation duration
// - Keyframe timing
// - Transform values (translation, rotation, scale)
```

### Retargeting Poses

You can also retarget individual poses:

```rust
// Retarget a single pose
let target_pose = retargeter.retarget_pose(&source_pose, &target_skeleton);
```

### Use Cases

**1. Sharing Animations Across Characters**
```rust
// Retarget walk animation from male to female character
let female_walk = retargeter.retarget_clip(&male_walk, &female_skeleton);
```

**2. Using Mocap Data**
```rust
// Retarget motion capture to game character
let game_animation = retargeter.retarget_clip(&mocap_clip, &game_skeleton);
```

**3. Cross-Species Animation**
```rust
// Map humanoid animation to quadruped (with appropriate bone mapping)
let mut mapping = BoneMapping::new();
mapping.map_bones(spine_human, spine_dog);
mapping.map_bones(left_arm_human, left_front_leg_dog);
// ... etc
```

---

## Additive Animation Blending

Additive animation allows you to layer animations on top of a base animation by computing deltas from a reference pose.

### Creating Additive Animations

```rust
use praxis_scene::{AdditiveAnimation, AdditiveMode};

// Create additive animation
let mut additive = AdditiveAnimation::new(
    "Walk".to_string(),      // Base animation
    "Recoil".to_string()     // Additive animation
)
.with_weight(1.0)
.with_mode(AdditiveMode::Local);

// Compute reference pose (usually bind pose)
additive.compute_reference_from_skeleton(&skeleton);
```

### Applying Additive Animations

```rust
// Apply additive animation to a pose
additive.apply(
    &mut base_pose,
    &recoil_clip,
    0.25,  // Time in additive clip
    &skeleton
);
```

### Additive Modes

#### Local Space Additive
```rust
.with_mode(AdditiveMode::Local)
```
- Adds deltas in local (parent-relative) space
- Best for most character animations
- Default mode

#### World Space Additive
```rust
.with_mode(AdditiveMode::World)
```
- Adds deltas in world space
- Useful for global effects

### How It Works

1. **Reference Pose**: Typically the bind pose (rest position)
2. **Delta Calculation**: 
   - Translation: `delta = additive_value - reference_value`
   - Rotation: `delta = reference_rotation^-1 * additive_rotation`
   - Scale: `delta = additive_scale / reference_scale`
3. **Application**:
   - Translation: `final = base + delta * weight`
   - Rotation: `final = base * slerp(identity, delta, weight)`
   - Scale: `final = base * lerp(1, delta, weight)`

### Use Cases

**1. Weapon Recoil**
```rust
// Base: Walk animation
// Additive: Recoil animation (upper body only)
let recoil_additive = AdditiveAnimation::new("Walk".to_string(), "Recoil".to_string());
recoil_additive.compute_reference_from_skeleton(&skeleton);

// Character walks normally while upper body recoils
```

**2. Breathing Idle**
```rust
// Base: Idle stance
// Additive: Breathing motion
let breathing = AdditiveAnimation::new("Idle".to_string(), "Breathing".to_string())
    .with_weight(0.5);  // Subtle breathing
```

**3. Damage Reactions**
```rust
// Base: Running
// Additive: Hit reaction (shoulder flinch)
let hit_reaction = AdditiveAnimation::new("Run".to_string(), "HitReaction".to_string())
    .with_weight(damage_intensity);
```

**4. Layered Emotions**
```rust
// Base: Walk
// Additive: Sad expression
let sad_walk = AdditiveAnimation::new("Walk".to_string(), "SadFace".to_string());
```

### Best Practices

1. **Reference Pose Selection**: Use bind pose for most cases
2. **Weight Control**: Use weights for subtle blending
3. **Bone Masking**: Combine with layer masks for partial body additive
4. **Performance**: Additive is more expensive than simple blending

---

## Root Motion Extraction

Root motion extraction separates character movement (translation/rotation) from the animation, allowing it to be applied to a character controller for precise movement.

### Creating Root Motion Extractor

```rust
use praxis_scene::RootMotionExtractor;

let mut extractor = RootMotionExtractor::new(0)  // Root bone index
    .with_translation(true)   // Extract translation
    .with_rotation(true)      // Extract rotation
    .with_auto_apply(true);   // Apply to transform
```

### Extracting Root Motion

```rust
// In your animation system
fn animation_system(
    mut query: Query<(&Skeleton, &mut AnimatedPose, &mut RootMotionExtractor)>
) {
    for (skeleton, mut pose, mut extractor) in query.iter_mut() {
        // Extract motion from pose
        extractor.extract(&mut pose, skeleton);
        
        // Get the extracted motion
        let motion = extractor.motion();
        
        // Apply to character controller
        if !motion.consumed {
            character_controller.move_by(motion.translation);
            character_controller.rotate_by(motion.rotation);
            motion.consume();
        }
    }
}
```

### Root Motion Structure

```rust
pub struct RootMotion {
    pub translation: Vec3,   // Translation delta
    pub rotation: Quat,      // Rotation delta
    pub consumed: bool,      // Whether applied
}
```

### Extraction Modes

#### Translation Only
```rust
let extractor = RootMotionExtractor::new(0)
    .with_translation(true)
    .with_rotation(false);
```
- Character moves but doesn't rotate from animation
- Useful for strafing, side-stepping

#### Rotation Only
```rust
let extractor = RootMotionExtractor::new(0)
    .with_translation(false)
    .with_rotation(true);
```
- Character rotates but doesn't translate
- Useful for turning in place

#### Both (Default)
```rust
let extractor = RootMotionExtractor::new(0)
    .with_translation(true)
    .with_rotation(true);
```
- Full root motion extraction
- Most common for locomotion

### How It Works

1. **Delta Computation**: Calculates change in root bone from previous frame
2. **Motion Storage**: Stores translation and rotation deltas
3. **Bone Zeroing**: Removes motion from animation (optional)
4. **Transform Application**: Applied to character's world transform

### Use Cases

**1. Precise Character Movement**
```rust
// Animation dictates exact movement speed
let motion = extractor.motion();
transform.translation += motion.translation;
transform.rotation *= motion.rotation;
```

**2. Path Following**
```rust
// Use animation motion for natural movement along path
if !on_rails_sequence {
    apply_root_motion(&motion);
}
```

**3. Combat Movement**
```rust
// Extract motion from attack animations for lunges, dodges
let motion = extractor.motion();
if is_attacking {
    apply_motion_with_scale(motion, attack_mobility_factor);
}
```

**4. Climbing Animations**
```rust
// Extract precise hand/foot placement motion
extractor.with_translation(true).with_rotation(false);
let motion = extractor.motion();
physics_controller.move_kinematic(motion.translation);
```

### Best Practices

1. **Frame Independence**: Root motion is frame-rate independent
2. **Consumption Pattern**: Mark motion as consumed after applying
3. **Blending**: When blending animations, blend root motions too
4. **Physics Integration**: Apply root motion to physics controller, not transform directly
5. **Reset on Change**: Call `reset()` when changing animations

### Root Motion with Animation Blending

```rust
// When blending multiple animations
let motion1 = extractor1.motion();
let motion2 = extractor2.motion();

// Blend motions
let blended_translation = motion1.translation.lerp(motion2.translation, blend_weight);
let blended_rotation = motion1.rotation.slerp(motion2.rotation, blend_weight);

let final_motion = RootMotion::new(blended_translation, blended_rotation);
apply_to_controller(&final_motion);
```

---

## Complete Example

Here's a complete example combining all advanced features:

```rust
use praxis_scene::*;
use praxis_math::{Vec3, Quat};

fn advanced_animation_system(
    skeleton: &Skeleton,
    animation_player: &mut AnimationPlayer,
    ik_controller: &IkController,
    root_motion_extractor: &mut RootMotionExtractor,
    target_hand_position: Vec3,
) {
    // 1. Evaluate base animation
    let mut pose = animation_player.evaluate(skeleton);
    
    // 2. Apply additive animation (e.g., breathing)
    let mut breathing = AdditiveAnimation::new("Idle".to_string(), "Breathe".to_string());
    breathing.compute_reference_from_skeleton(skeleton);
    
    if let Some(breathe_clip) = animation_player.clip("Breathe") {
        breathing.apply(&mut pose, breathe_clip, 0.5, skeleton);
    }
    
    // 3. Apply IK (e.g., reaching for object)
    ik_controller.apply(&mut pose, skeleton);
    
    // 4. Extract root motion
    root_motion_extractor.extract(&mut pose, skeleton);
    let motion = root_motion_extractor.motion();
    
    // 5. Apply motion to character controller
    if !motion.consumed {
        // Apply to your character controller
        println!("Move by: {:?}", motion.translation);
        println!("Rotate by: {:?}", motion.rotation);
    }
    
    // Final pose is ready for rendering
}

// Using retargeting
fn retarget_animation_library(
    source_skeleton: &Skeleton,
    target_skeleton: &Skeleton,
    animations: &[AnimationClip],
) -> Vec<AnimationClip> {
    let retargeter = AnimationRetargeter::auto(source_skeleton, target_skeleton);
    
    animations
        .iter()
        .map(|clip| retargeter.retarget_clip(clip, target_skeleton))
        .collect()
}
```

---

## Performance Considerations

### IK
- Two-bone IK: ~1-2 µs per constraint
- Chain IK: ~10-50 µs per constraint (depends on chain length and iterations)
- Look-at IK: ~0.5-1 µs per constraint

### Animation Retargeting
- One-time cost when retargeting clips
- Runtime cost is same as regular animation playback
- Consider pre-retargeting and caching results

### Additive Animation
- ~20-30% more expensive than regular blending
- Reference pose lookup adds overhead
- Use sparingly (1-2 additive layers maximum)

### Root Motion
- Minimal overhead (~1-2 µs per extraction)
- Most cost is in the animation evaluation itself

---

## Troubleshooting

### IK Issues

**Problem**: IK doesn't reach target
- **Solution**: Check if target is within reach distance
- **Solution**: Increase weight if blending with animation
- **Solution**: Verify bone hierarchy is correct

**Problem**: Unnatural bending
- **Solution**: Use pole target to control bend direction
- **Solution**: Reduce weight for subtle IK influence

### Retargeting Issues

**Problem**: Animation looks wrong on target skeleton
- **Solution**: Check bone mapping is correct
- **Solution**: Verify bone hierarchies are similar
- **Solution**: May need manual mapping for different proportions

### Additive Issues

**Problem**: Animation looks exaggerated
- **Solution**: Reduce additive weight
- **Solution**: Check reference pose is correct (should be bind pose)

### Root Motion Issues

**Problem**: Character slides or doesn't move
- **Solution**: Verify root bone index is correct
- **Solution**: Check if motion is being consumed
- **Solution**: Ensure character controller applies the motion

---

## Summary

The advanced animation features in Praxis provide powerful tools for creating dynamic, responsive character animation:

- **IK**: Procedural limb positioning for adaptive interaction
- **Retargeting**: Share animations across different characters
- **Additive Blending**: Layer subtle animations on top of base animations
- **Root Motion**: Extract precise movement from animations

These features work seamlessly with the existing animation system and can be combined for sophisticated animation behaviors.
