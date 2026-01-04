# Quick Start: Advanced Animation Features

Get started with advanced animation features in under 5 minutes.

## Installation

No installation needed! All features are included in `praxis_scene`.

```rust
use praxis_scene::*;
```

## 1. Inverse Kinematics (30 seconds)

Make a character's hand reach for an object:

```rust
// Setup IK
let mut ik = IkController::new();
ik.add_constraint(
    IkConstraint::new_two_bone(hand_bone_index, target_position)
);

// Apply to pose (after animation evaluation)
ik.apply(&mut pose, &skeleton);
```

**Try it now:**
```bash
cargo run --example animation_advanced_demo
```

## 2. Animation Retargeting (1 minute)

Share animations between different characters:

```rust
// Auto-retarget based on bone names
let retargeter = AnimationRetargeter::auto(&source_skeleton, &target_skeleton);

// Retarget a clip
let new_clip = retargeter.retarget_clip(&old_clip, &target_skeleton);

// Use the new clip
animation_player.add_clip("Walk".to_string(), new_clip);
```

**Use case:** Take mocap data and apply it to your game character.

## 3. Additive Animation (2 minutes)

Layer animations for effects like breathing or recoil:

```rust
// Setup additive
let mut additive = AdditiveAnimation::new(
    "Walk".to_string(),      // Base
    "Recoil".to_string()     // Effect
);
additive.compute_reference_from_skeleton(&skeleton);

// Apply to pose
additive.apply(&mut pose, &recoil_clip, time, &skeleton);
```

**Result:** Character walks while upper body recoils from weapon fire.

## 4. Root Motion (1 minute)

Extract character movement from animation:

```rust
// Setup extractor
let mut extractor = RootMotionExtractor::new(0);  // Root bone index

// Extract motion (after animation evaluation)
extractor.extract(&mut pose, &skeleton);

// Apply to character
let motion = extractor.motion();
character_position += motion.translation;
character_rotation *= motion.rotation;
motion.consume();
```

**Result:** Precise character movement that matches animation.

---

## Complete Example (5 minutes)

Here's a complete animated character with all features:

```rust
use praxis_scene::*;
use praxis_math::{Vec3, Quat};

struct AnimatedCharacter {
    skeleton: Skeleton,
    animation_player: AnimationPlayer,
    ik_controller: IkController,
    root_motion: RootMotionExtractor,
    pose: AnimatedPose,
}

impl AnimatedCharacter {
    fn new(skeleton: Skeleton) -> Self {
        Self {
            skeleton: skeleton.clone(),
            animation_player: AnimationPlayer::new(),
            ik_controller: IkController::new(),
            root_motion: RootMotionExtractor::new(0),
            pose: AnimatedPose::new(skeleton.bone_count()),
        }
    }
    
    fn update(&mut self, delta_time: f32, hand_target: Option<Vec3>) {
        // 1. Update animation
        self.animation_player.update(delta_time);
        self.pose = self.animation_player.evaluate(&self.skeleton);
        
        // 2. Apply IK if reaching for something
        if let Some(target) = hand_target {
            self.ik_controller.clear_constraints();
            self.ik_controller.add_constraint(
                IkConstraint::new_two_bone(
                    self.hand_bone_index(),
                    target
                )
            );
            self.ik_controller.apply(&mut self.pose, &self.skeleton);
        }
        
        // 3. Extract root motion
        self.root_motion.extract(&mut self.pose, &self.skeleton);
    }
    
    fn get_motion(&self) -> (Vec3, Quat) {
        let motion = self.root_motion.motion();
        (motion.translation, motion.rotation)
    }
    
    fn hand_bone_index(&self) -> usize {
        self.skeleton.find_bone("RightHand").unwrap_or(0)
    }
}

// Usage
fn main() {
    let skeleton = load_skeleton();
    let mut character = AnimatedCharacter::new(skeleton);
    
    // Game loop
    loop {
        let delta_time = 0.016;
        let hand_target = Some(Vec3::new(2.0, 1.5, 0.0));
        
        character.update(delta_time, hand_target);
        
        let (translation, rotation) = character.get_motion();
        // Apply to character transform
    }
}

fn load_skeleton() -> Skeleton {
    // Your skeleton loading code
    Skeleton::new(vec![])
}
```

---

## Next Steps

### Learn More
- [Full Documentation](animation_advanced_features.md)
- [Quick Reference](quick_reference_advanced_animation.md)
- [Integration Guide](guides/advanced_animation_integration.md)

### Try Examples
```bash
# Complete demo of all features
cargo run --example animation_advanced_demo

# See existing animation examples
cargo run --example skeletal_animation_demo
cargo run --example animation_blending_demo
```

### Common Use Cases

**Foot IK on Terrain:**
```rust
let ground_pos = raycast_ground(foot_position);
let ik = IkConstraint::new_two_bone(foot_bone, ground_pos)
    .with_pole_target(knee_hint);
```

**Character Aiming:**
```rust
let look_at = IkConstraint::new_look_at(head_bone, target_position);
```

**Weapon Recoil:**
```rust
let recoil = AdditiveAnimation::new("Idle".into(), "Recoil".into());
recoil.compute_reference_from_skeleton(&skeleton);
recoil.apply(&mut pose, &recoil_clip, time, &skeleton);
```

**Precise Movement:**
```rust
extractor.extract(&mut pose, &skeleton);
let motion = extractor.motion();
transform.translation += motion.translation;
```

---

## Tips

1. **IK:** Always apply IK *after* evaluating animations
2. **Retargeting:** Do it once at load time, not every frame
3. **Additive:** Keep weight < 1.0 for subtle effects
4. **Root Motion:** Remember to call `.consume()` after applying

---

## Troubleshooting

**IK not working?**
- Check if target is within reach
- Verify bone indices are correct
- Ensure IK is applied after animation

**Animation looks wrong after retargeting?**
- Check bone names match (case-insensitive)
- Verify skeleton hierarchies are similar
- Try manual bone mapping for different rigs

**Root motion not applying?**
- Ensure motion is extracted every frame
- Check if motion is being consumed
- Verify root bone index is correct

---

## Help & Resources

- 📖 [Full Documentation](animation_advanced_features.md)
- 💡 [Code Examples](../examples/animation_advanced_demo.rs)
- 🔍 [Quick Reference](quick_reference_advanced_animation.md)
- 🚀 [Integration Guide](guides/advanced_animation_integration.md)

**Happy Animating!** 🎮
