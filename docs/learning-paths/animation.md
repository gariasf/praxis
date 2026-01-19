# Animation Learning Path

Master character animation from skeletal basics to advanced inverse kinematics and retargeting.

## Path Overview

**Time Investment**: 2-4 weeks depending on animation experience  
**Prerequisites**: Understanding of transforms and hierarchies  
**Final Goal**: Create production-ready character animation systems

## Progression Map

```
Beginner (1 week)
├── Transform hierarchies
├── Skeletal structure
├── Animation clips
└── Basic playback
    ↓
Intermediate (1-2 weeks)
├── Cross-fade blending
├── Blend trees
├── Layered animation
└── State machines
    ↓
Advanced (1-2 weeks)
├── Inverse Kinematics (IK)
├── Animation retargeting
├── Additive blending
├── Root motion
└── Performance optimization
```

---

## Beginner: Skeletal Animation Basics

**Goal**: Load and play skeletal animations with proper hierarchy management.

### Prerequisites

- ✓ Understanding of 3D transforms (position, rotation, scale)
- ✓ Basic knowledge of parent-child relationships
- ✓ Completed [Getting Started](../getting-started/README.md)

### Step 1: Understand Hierarchies

**Theory** (2-3 hours):
1. Read [Transform Hierarchy Concepts](../concepts/transform-hierarchy.md)
   - Local vs global transforms
   - Parent-child relationships
   - Transform propagation

2. Read [Beginner's Guide: Transform Propagation](../beginners-guide.md#transform-hierarchy-propagation)
   - How transforms flow through hierarchy
   - Matrix multiplication
   - Recursive updates

**Key Concepts**:
- Local transform: Relative to parent
- Global transform: In world space
- Children move with parents

**Visual Exercise**: 
```
Character (root)
├── Spine
│   ├── Chest
│   │   ├── Left Shoulder → Left Elbow → Left Hand
│   │   └── Right Shoulder → Right Elbow → Right Hand
│   └── Neck → Head
└── Pelvis
    ├── Left Hip → Left Knee → Left Foot
    └── Right Hip → Right Knee → Right Foot
```

### Step 2: Skeletal Structure

**Theory** (2-3 hours):
1. Read [Animation Concepts](../concepts/animation.md)
   - Skeleton definition
   - Bones and joints
   - Bind pose vs animated pose

2. Read [Skeletal Basics Guide](../guides/animation/skeletal-basics.md)
   - Core architecture
   - Skeleton component
   - Joint hierarchy

**Practice** (2 hours):
1. Run example:
   ```bash
   cargo run --example skeletal_animation_demo
   ```
2. Observe skeleton structure
3. Identify bones and joints

**Understanding Goal**: How skeletons store hierarchy

### Step 3: Animation Clips

**Theory** (2 hours):
1. Continue [Skeletal Basics: Animation Clips](../guides/animation/skeletal-basics.md#animation-clips)
   - Keyframes
   - Interpolation (linear, cubic)
   - Animation duration
   - Looping

**Key Data Structure**:
```rust
AnimationClip {
    duration: f32,
    channels: Vec<JointChannel>,  // Per-bone animation
}

JointChannel {
    joint_index: usize,
    keyframes: Vec<Keyframe>,
}

Keyframe {
    time: f32,
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}
```

**Practice** (3 hours):
1. Load animation from GLTF file
2. Inspect animation channels
3. Play animation

**Exercise**: Load a walk cycle animation

### Step 4: Animation Playback

**Practice** (4-5 hours):
1. Read [Animation Overview Guide](../guides/animation.md)
2. Implement basic playback:
   - Start animation
   - Update time
   - Sample keyframes
   - Apply to skeleton

**Code Pattern**:
```rust
use praxis_scene::{AnimationPlayer, AnimationClip};

// Create player
let mut player = AnimationPlayer::new();
player.play(&walk_animation);

// Update each frame
player.update(delta_time);

// Apply to skeleton
player.apply_to_skeleton(&mut skeleton);
```

**Exercises**:
1. Play walk animation
2. Play run animation
3. Control playback speed
4. Pause and resume
5. Loop animation

### Step 5: GLTF Integration

**Practice** (3-4 hours):
1. Read [Assets Guide](../guides/assets.md) - Animation section
2. Load skinned mesh from GLTF
3. Load animations from GLTF
4. Connect mesh to skeleton

**Example**:
```bash
cargo run --example gltf_animation_loader_demo
```

**Exercises**:
1. Load character with multiple animations
2. Switch between animations
3. Adjust animation speed
4. Export custom GLTF with Blender

### Beginner Checkpoint

**Self-Assessment**:
- [ ] Understand transform hierarchies
- [ ] Can explain skeleton structure
- [ ] Know how animation clips work
- [ ] Can load and play animations
- [ ] Comfortable with GLTF workflow

**Capstone Project**: Create animated character with:
- Loaded from GLTF file
- Multiple animations (idle, walk, run)
- Ability to switch between animations
- Proper skeleton visualization

**Time to Complete**: 15-20 hours

---

## Intermediate: Animation Blending & Control

**Goal**: Create smooth, responsive animation systems with blending.

### Prerequisites

- ✓ Completed Beginner section
- ✓ Can load and play animations
- ✓ Understanding of keyframe animation

### Step 1: Cross-Fade Blending

**Theory** (2-3 hours):
1. Read [Blending Guide](../guides/animation/blending.md)
   - Why blending matters
   - Linear interpolation between poses
   - Blend weights
   - Transition timing

**Math Foundation**:
```
Blended Pose = Pose A × (1 - weight) + Pose B × weight

weight: 0.0 = fully Pose A
weight: 0.5 = 50% mix
weight: 1.0 = fully Pose B
```

**Practice** (3-4 hours):
1. Implement cross-fade
2. Transition from walk to run
3. Adjust transition duration

**Exercises**:
1. Smooth transition: Walk → Run (0.3s)
2. Instant transition: Idle → Jump
3. Long transition: Combat → Idle (1.0s)
4. Visualize blend weights over time

### Step 2: Blend Trees

**Theory** (3 hours):
1. Continue [Blending Guide: Blend Trees](../guides/animation/blending.md#blend-trees)
   - 1D blend spaces (walk → run → sprint)
   - 2D blend spaces (directional movement)
   - Blend parameters

**Conceptual Example**:
```
Speed Parameter (0.0 to 1.0):
  0.0 → Idle (100%)
  0.3 → Walk (70%) + Run (30%)
  0.6 → Walk (30%) + Run (70%)
  1.0 → Sprint (100%)
```

**Practice** (5-6 hours):
1. Build 1D blend tree (locomotion)
2. Implement parameter control
3. Test different parameter values

**Exercises**:
1. Create walk-run-sprint blend
2. Smoothly adjust speed parameter
3. Add crouch variations
4. Implement directional blending

### Step 3: Layered Animation

**Theory** (2-3 hours):
1. Continue [Blending Guide: Layered Animation](../guides/animation/blending.md#layered-animation)
   - Upper body vs lower body
   - Masked blending
   - Layer priorities

**Use Cases**:
- Lower body: Walking/running
- Upper body: Aiming/shooting
- Full body override: Death animation

**Practice** (4-5 hours):
1. Implement layer system
2. Split skeleton into regions
3. Blend different animations per region

**Exercises**:
1. Lower body: Walk cycle
2. Upper body: Aim animation
3. Combine: Walking while aiming
4. Add reload animation to upper body

**Run Example**:
```bash
cargo run --example animation_blending_demo
```

### Step 4: Animation State Machines

**Theory** (3-4 hours):
1. Read [Skeletal Animation Complete](../guides/animation/skeletal-animation.md)
2. Study state machine patterns
   - States (idle, walk, run, jump)
   - Transitions (conditions)
   - Default state

**State Graph**:
```
    [Idle]
      ↓↑
   [Walk] ←→ [Run] ←→ [Sprint]
      ↓          ↓
   [Jump] ←→ [Fall]
      ↓
   [Land] → [Idle]
```

**Practice** (6-8 hours):
1. Design state graph
2. Implement state machine
3. Define transition conditions
4. Add transition blending

**Exercises**:
1. Basic FSM: Idle ↔ Walk ↔ Run
2. Add jumping (grounded check)
3. Add falling (velocity check)
4. Add landing recovery
5. Add combat states

### Step 5: Input Integration

**Practice** (4-5 hours):
1. Read [Input Guide](../guides/input.md)
2. Connect input to animation parameters
3. Implement responsive controls

**Integration Pattern**:
```rust
// Input → Parameter → Animation
let speed = input.movement_magnitude();
blend_tree.set_parameter("speed", speed);

if input.jump_pressed() && grounded {
    state_machine.trigger("jump");
}
```

**Exercises**:
1. WASD controls locomotion blend tree
2. Space bar triggers jump
3. Shift modifies speed parameter
4. Mouse controls aim direction

### Intermediate Checkpoint

**Self-Assessment**:
- [ ] Can implement smooth transitions
- [ ] Understand blend trees and parameters
- [ ] Can layer animations (upper/lower body)
- [ ] Built a working state machine
- [ ] Integrated animation with input
- [ ] Created responsive character movement

**Capstone Project**: Third-person character with:
- State machine (idle, walk, run, jump, fall, land)
- Locomotion blend tree
- Upper body aiming layer
- Input-driven animation
- Smooth transitions between all states

**Time to Complete**: 25-35 hours

---

## Advanced: IK, Retargeting, and Root Motion

**Goal**: Implement production-grade animation features and optimization.

### Prerequisites

- ✓ Completed Intermediate section
- ✓ Strong understanding of blending
- ✓ Built animation state machines

### Step 1: Inverse Kinematics (IK)

**Theory** (4-5 hours):
1. Read [Advanced Features: IK](../guides/animation/advanced-features.md#inverse-kinematics-ik)
2. Study IK algorithms:
   - Two-bone IK (arms, legs)
   - FABRIK (Full Body IK)
   - Constraints

**Mathematical Concept**:
```
Forward Kinematics (FK):
  Joint angles → End effector position

Inverse Kinematics (IK):
  Target position → Joint angles
```

**Practice** (8-10 hours):
1. Implement two-bone IK solver
2. Apply to character's arm
3. Apply to character's leg
4. Add pole targets (elbow/knee direction)

**Exercises**:
1. Foot IK for uneven terrain
2. Hand IK to reach objects
3. Look-at IK for head
4. Hand placement on weapons

**Run Example**:
```bash
cargo run --example animation_advanced_demo
```

### Step 2: Animation Retargeting

**Theory** (3-4 hours):
1. Continue [Advanced Features: Retargeting](../guides/animation/advanced-features.md#retargeting)
2. Study skeleton differences
3. Understand bone mapping

**Challenges**:
- Different bone proportions
- Different joint hierarchies
- Different bind poses

**Practice** (6-8 hours):
1. Create bone mapping
2. Implement retargeting algorithm
3. Test on different skeletons

**Exercises**:
1. Retarget walk cycle: Human → Robot
2. Adjust for proportion differences
3. Maintain animation quality
4. Handle missing bones

### Step 3: Additive Blending

**Theory** (2-3 hours):
1. Continue [Advanced Features: Additive](../guides/animation/advanced-features.md#additive-blending)
2. Understand difference vs absolute blending
3. Study use cases

**Additive Formula**:
```
Result = Base Animation + (Additive Animation - Reference Pose)
```

**Use Cases**:
- Breathing motion on top of any animation
- Recoil on top of aiming
- Leaning on top of locomotion

**Practice** (4-5 hours):
1. Create additive animations
2. Apply to base animations
3. Control additive intensity

**Exercises**:
1. Breathing additive layer
2. Weapon recoil additive
3. Lean additive (left/right)
4. Hit reaction additive

### Step 4: Root Motion

**Theory** (3 hours):
1. Continue [Advanced Features: Root Motion](../guides/animation/advanced-features.md#root-motion)
2. Understand root vs child motion
3. Study extraction methods

**Root Motion Concept**:
- Animation moves character in place
- Extract root bone movement
- Apply to entity transform
- Character moves in world space

**Practice** (5-6 hours):
1. Extract root motion from animation
2. Apply to character controller
3. Handle rotation and translation
4. Blend root motion

**Exercises**:
1. Walk animation with root motion
2. Turn-in-place animation
3. Jump with forward motion
4. Dodge roll

**Integration**: Combine with [Physics Path](physics.md) for character controller

### Step 5: Performance Optimization

**Theory** (2-3 hours):
1. Read [Performance Path](performance.md) for profiling basics
2. Study animation performance characteristics
3. Understand animation-specific bottlenecks

**Optimization Techniques**:
- Animation LOD (simpler for distant characters)
- Update rate reduction
- Bone culling
- Animation compression

**Practice** (6-8 hours):
1. Profile animation systems
2. Implement animation LOD
3. Optimize blend tree evaluation
4. Reduce memory usage

**Exercises**:
1. Profile 100 animated characters
2. Implement distance-based LOD
3. Reduce update rate for distant characters
4. Measure performance gains

**Target**: Support 100+ animated characters at 60 FPS

### Advanced Checkpoint

**Self-Assessment**:
- [ ] Can implement IK solvers
- [ ] Understand animation retargeting
- [ ] Know how to use additive blending
- [ ] Can extract and apply root motion
- [ ] Optimized animation systems for performance
- [ ] Built production-ready animation pipeline

**Capstone Project**: Choose one:

1. **IK System**: Full-body IK with foot placement, hand reaching, and look-at
2. **Retargeting Tool**: Tool to retarget animations between different characters
3. **MMO Animation**: Optimize system to support 200+ animated characters

**Time to Complete**: 35-50 hours

---

## Cross-References

### Related Systems
- [Physics Path](physics.md) - Character controllers, ragdolls
- [Scripting Path](scripting.md) - Script-driven animation logic
- [Rendering Path](rendering.md) - Skinned mesh rendering

### Performance
- [Performance Path](performance.md) - Profiling and optimization
- [Spatial Optimization](../guides/spatial-optimization.md) - Animation LOD strategies

### Assets
- [Assets Guide](../guides/assets.md) - Load animations from GLTF
- [Assets Path](assets.md) - Complete asset pipeline mastery

---

## Practice Resources

### Examples to Study
```bash
# Beginner
cargo run --example skeletal_animation_demo
cargo run --example animation_demo

# Intermediate
cargo run --example animation_blending_demo
cargo run --example gltf_animation_loader_demo

# Advanced
cargo run --example animation_advanced_demo
```

### External Tools
- **Blender**: Create and export animations
- **Mixamo**: Free character animations
- **Cascadeur**: AI-assisted animation

### Reference Animations
Look for these animation types to practice:
- Locomotion: idle, walk, run, sprint
- Actions: jump, fall, land, crouch
- Combat: attack, block, dodge, death
- Interactions: open door, climb ladder, pick up

---

## Next Steps

After completing this path:

1. **Specialize**: Deep dive into specific areas (IK, retargeting)
2. **Integrate**: Combine with [Physics Path](physics.md) for ragdolls
3. **Optimize**: Focus on large-scale animation (MMO scenarios)
4. **Create**: Build a complete animated game character

## Getting Help

- Study examples in `examples/` directory
- Review `praxis_scene` crate for animation components
- Check animation system source code
- Test with various GLTF models

---

[← Back to Learning Paths](README.md) | [Next: Physics Path →](physics.md)
