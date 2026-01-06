# Quick Reference: Advanced Animation Features

Quick reference guide for Praxis advanced animation features.

## Inverse Kinematics (IK)

### Two-Bone IK (Arms/Legs)
```rust
use praxis_scene::{IkConstraint, IkController};

let constraint = IkConstraint::new_two_bone(hand_bone_idx, target_position)
    .with_pole_target(elbow_hint)
    .with_weight(1.0);

let mut ik = IkController::new();
ik.add_constraint(constraint);
ik.apply(&mut pose, &skeleton);
```

### Chain IK (Spine/Tail)
```rust
let constraint = IkConstraint::new_chain(end_bone_idx, target_position, 20);
ik.add_constraint(constraint);
```

### Look-At IK (Head/Eyes)
```rust
let constraint = IkConstraint::new_look_at(head_bone_idx, look_target);
ik.add_constraint(constraint);
```

### System Integration
```rust
use praxis_scene::apply_ik_constraints;

fn my_system(query: &mut Query<(&Skeleton, &IkController, &mut AnimatedPose)>) {
    apply_ik_constraints(query);
}
```

---

## Animation Retargeting

### Automatic Retargeting
```rust
use praxis_scene::AnimationRetargeter;

let retargeter = AnimationRetargeter::auto(&source_skeleton, &target_skeleton);
let new_clip = retargeter.retarget_clip(&source_clip, &target_skeleton);
```

### Manual Bone Mapping
```rust
use praxis_scene::BoneMapping;

let mut mapping = BoneMapping::new();
mapping.map_bones(0, 0);  // source_idx, target_idx
mapping.map_bone_names("LeftArm".to_string(), "L_Arm".to_string());

let retargeter = AnimationRetargeter::new(mapping);
```

### Retarget Pose
```rust
let target_pose = retargeter.retarget_pose(&source_pose, &target_skeleton);
```

---

## Additive Animation Blending

### Basic Setup
```rust
use praxis_scene::{AdditiveAnimation, AdditiveMode};

let mut additive = AdditiveAnimation::new("Base".to_string(), "Additive".to_string())
    .with_weight(1.0)
    .with_mode(AdditiveMode::Local);

additive.compute_reference_from_skeleton(&skeleton);
```

### Apply Additive
```rust
additive.apply(&mut base_pose, &additive_clip, time, &skeleton);
```

### Common Patterns
```rust
// Weapon recoil
let recoil = AdditiveAnimation::new("Walk".to_string(), "Recoil".to_string());

// Breathing
let breathing = AdditiveAnimation::new("Idle".to_string(), "Breathe".to_string())
    .with_weight(0.3);

// Hit reaction
let hit = AdditiveAnimation::new("Run".to_string(), "Hit".to_string())
    .with_weight(damage_intensity);
```

---

## Root Motion Extraction

### Basic Setup
```rust
use praxis_scene::RootMotionExtractor;

let mut extractor = RootMotionExtractor::new(root_bone_idx)
    .with_translation(true)
    .with_rotation(true);
```

### Extract and Apply
```rust
extractor.extract(&mut pose, &skeleton);

let motion = extractor.motion();
if !motion.consumed {
    character_controller.move_by(motion.translation);
    character_controller.rotate_by(motion.rotation);
    motion.consume();
}
```

### Configuration Options
```rust
// Translation only
let extractor = RootMotionExtractor::new(0)
    .with_translation(true)
    .with_rotation(false);

// Rotation only
let extractor = RootMotionExtractor::new(0)
    .with_translation(false)
    .with_rotation(true);

// Disable auto-apply to transform
let extractor = RootMotionExtractor::new(0)
    .with_auto_apply(false);
```

---

## Common Patterns

### Foot IK on Terrain
```rust
// Get ground positions
let left_ground = raycast_ground(left_foot_position);
let right_ground = raycast_ground(right_foot_position);

// Create IK constraints
let left_ik = IkConstraint::new_two_bone(left_foot_bone, left_ground)
    .with_pole_target(left_knee_hint)
    .with_weight(1.0);

let right_ik = IkConstraint::new_two_bone(right_foot_bone, right_ground)
    .with_pole_target(right_knee_hint)
    .with_weight(1.0);

ik_controller.add_constraint(left_ik);
ik_controller.add_constraint(right_ik);
```

### Retarget Animation Library
```rust
fn retarget_all(
    source_skeleton: &Skeleton,
    target_skeleton: &Skeleton,
    clips: &[AnimationClip],
) -> Vec<AnimationClip> {
    let retargeter = AnimationRetargeter::auto(source_skeleton, target_skeleton);
    clips.iter().map(|c| retargeter.retarget_clip(c, target_skeleton)).collect()
}
```

### Layered Additive Animations
```rust
// Base animation
let mut pose = player.evaluate(&skeleton);

// Layer 1: Breathing
breathing_additive.apply(&mut pose, &breathe_clip, breathe_time, &skeleton);

// Layer 2: Weapon sway
sway_additive.apply(&mut pose, &sway_clip, sway_time, &skeleton);

// Layer 3: Hit reaction (conditional)
if taking_damage {
    hit_additive.apply(&mut pose, &hit_clip, hit_time, &skeleton);
}
```

### Root Motion with Blending
```rust
// Extract from multiple animations
let motion1 = extractor1.motion();
let motion2 = extractor2.motion();

// Blend motions
let blended_translation = motion1.translation.lerp(motion2.translation, weight);
let blended_rotation = motion1.rotation.slerp(motion2.rotation, weight);

// Apply blended motion
controller.move_by(blended_translation);
controller.rotate_by(blended_rotation);
```

---

## Performance Tips

### IK
- Use two-bone IK when possible (faster than chain)
- Reduce max iterations for chain IK if performance is critical
- Cache constraint results when targets don't change frequently

### Retargeting
- Pre-retarget animations at load time, don't retarget every frame
- Cache `AnimationRetargeter` instances
- Use automatic mapping for prototyping, manual for production

### Additive
- Limit to 2-3 additive layers maximum
- Use weight < 1.0 for subtle effects
- Consider bone masks to apply to specific body parts only

### Root Motion
- Extraction is very fast, use freely
- Reset extractor when changing animations
- Blend root motions when blending animations

---

## Debugging

### IK Not Working
```rust
// Check target is reachable
let max_reach = upper_length + lower_length;
if target_distance > max_reach {
    println!("Target unreachable: {} > {}", target_distance, max_reach);
}

// Visualize IK target and pole
debug_draw_sphere(constraint.target(), 0.1, Color::RED);
if let Some(pole) = constraint.pole_target {
    debug_draw_sphere(pole, 0.05, Color::BLUE);
}
```

### Retargeting Issues
```rust
// Print bone mapping
let mapping = retargeter.bone_mapping();
for source_idx in 0..source_skeleton.bone_count() {
    if let Some(target_idx) = mapping.get_target_bone(source_idx) {
        println!("{} -> {}", 
            source_skeleton.bone(source_idx).unwrap().name,
            target_skeleton.bone(target_idx).unwrap().name
        );
    }
}
```

### Root Motion Debugging
```rust
// Log motion deltas
let motion = extractor.motion();
println!("Translation: {:?}", motion.translation);
println!("Rotation: {:?}", motion.rotation);
println!("Consumed: {}", motion.consumed);

// Visualize motion vectors
debug_draw_arrow(character_pos, character_pos + motion.translation, Color::GREEN);
```

---

## Component Summary

| Component | Purpose | Add to Entity |
|-----------|---------|---------------|
| `IkController` | Manage IK constraints | Entities needing procedural posing |
| `RootMotionExtractor` | Extract movement from animation | Animated characters that move |
| `AnimationRetargeter` | Utility for retargeting (not a component) | N/A - use as needed |
| `AdditiveAnimation` | Utility for additive blending (not a component) | N/A - use as needed |

---

## Common Mistakes

❌ **Applying IK before animation evaluation**
```rust
// Wrong order
ik_controller.apply(&mut pose, &skeleton);
pose = player.evaluate(&skeleton);  // IK results overwritten!
```

✅ **Apply IK after animation evaluation**
```rust
// Correct order
pose = player.evaluate(&skeleton);
ik_controller.apply(&mut pose, &skeleton);
```

❌ **Forgetting to consume root motion**
```rust
let motion = extractor.motion();
apply_to_controller(motion);
// Motion applied again next frame!
```

✅ **Always consume root motion after applying**
```rust
let motion = extractor.motion();
if !motion.consumed {
    apply_to_controller(motion);
    motion.consume();
}
```

❌ **Retargeting every frame**
```rust
// Don't do this in update loop!
let clip = retargeter.retarget_clip(&source_clip, &target_skeleton);
```

✅ **Retarget once at load time**
```rust
// Do this at initialization
let retargeted_clips: HashMap<String, AnimationClip> = source_clips
    .iter()
    .map(|(name, clip)| {
        (name.clone(), retargeter.retarget_clip(clip, &target_skeleton))
    })
    .collect();
```

---

## Full Integration Example

```rust
use praxis_scene::*;

struct AnimatedCharacter {
    skeleton: Skeleton,
    player: AnimationPlayer,
    ik: IkController,
    root_motion: RootMotionExtractor,
}

impl AnimatedCharacter {
    fn update(&mut self, delta_time: f32, hand_target: Vec3) {
        // 1. Update animation
        self.player.update(delta_time);
        let mut pose = self.player.evaluate(&self.skeleton);
        
        // 2. Apply IK for hand reaching
        let hand_constraint = IkConstraint::new_two_bone(
            self.hand_bone_idx(),
            hand_target
        );
        let mut ik = IkController::new();
        ik.add_constraint(hand_constraint);
        ik.apply(&mut pose, &self.skeleton);
        
        // 3. Extract root motion
        self.root_motion.extract(&mut pose, &self.skeleton);
        
        // 4. Use the pose for rendering
        // pose now contains final transforms
    }
    
    fn get_and_consume_motion(&mut self) -> RootMotion {
        let motion = *self.root_motion.motion();
        self.root_motion.motion_mut().consume();
        motion
    }
}
```

---

See [animation-advanced-features.md](animation-advanced-features.md) for complete documentation.
