# Animation Guide

Practical guide to using the skeletal animation system in Praxis for character animation, blending, and GLTF workflows.

## Quick Start

### Basic Animation Setup

```rust
use praxis_scene::{Skeleton, AnimationPlayer, AnimatedPose, Bone};
use praxis_math::{Vec3, Quat};
use praxis_ecs::{World, Transform, GlobalTransform};

// Create skeleton
let bones = vec![
    Bone::with_bind_pose("Root", None, Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
    Bone::with_bind_pose("Spine", Some(0), Vec3::Y, Quat::IDENTITY, Vec3::ONE),
    Bone::with_bind_pose("Head", Some(1), Vec3::Y, Quat::IDENTITY, Vec3::ONE),
];
let skeleton = Skeleton::new(bones);

// Create animation player
let mut player = AnimationPlayer::new();

// Create animated pose
let pose = AnimatedPose::new(skeleton.bone_count());

// Spawn animated entity
let mut world = World::new();
world.spawn((
    Transform::default(),
    GlobalTransform::default(),
    skeleton,
    player,
    pose,
));
```

## Loading Animations from GLTF

### Load GLTF with Animations

```rust
use praxis_assets::GltfLoader;

let loader = GltfLoader::new();
let asset = loader.load_gltf("assets/models/character.gltf")?;

// Extract skeleton
let skeleton = asset.skins[0].skeleton.clone();

// Create player and add all animations
let mut player = AnimationPlayer::new();
for animation in &asset.animations {
    let name = animation.name.clone().unwrap_or_else(|| 
        format!("Animation_{}", player.clip_count())
    );
    player.add_clip(name, animation.clip.clone());
}

// Spawn entity
let pose = AnimatedPose::new(skeleton.bone_count());
world.spawn((
    Transform::default(),
    GlobalTransform::default(),
    skeleton,
    player,
    pose,
));
```

### Play an Animation

```rust
fn start_animation(mut query: Query<&mut AnimationPlayer>) {
    for mut player in query.iter_mut() {
        player.play("Walk");
        player.set_looping(true);
        player.set_speed(1.0);
    }
}
```

## Animation System Integration

Add animation update system to your schedule:

```rust
use praxis_scene::update_animations;
use praxis_ecs::Schedule;

schedule.add_systems(update_animations);
```

## Animation Control

### Playback Controls

```rust
// Play animation
player.play("Run");

// Pause/resume
player.pause();
player.resume();

// Stop animation
player.stop();

// Set playback speed
player.set_speed(2.0);  // 2x speed
player.set_speed(0.5);  // Slow motion

// Loop control
player.set_looping(true);

// Check state
if player.is_playing() {
    println!("Playing: {}", player.current_clip_name().unwrap());
}
```

### Time Control

```rust
// Jump to specific time
player.set_time(2.5);  // Jump to 2.5 seconds

// Get current time
let time = player.current_time();

// Get animation duration
let duration = player.current_duration();

// Get normalized time (0.0 to 1.0)
let normalized = time / duration;
```

## Animation Blending

### Simple Weighted Blending

Play multiple animations with different weights:

```rust
// Play two animations simultaneously
player.play("Walk");
player.set_weight("Walk", 0.7);

player.play("Run");
player.set_weight("Run", 0.3);

// Result: 70% walk + 30% run
```

### Cross-Fade Transitions

Smooth transitions between animations:

```rust
use praxis_scene::AnimationBlender;

let mut blender = AnimationBlender::new();
blender.add_clip("Idle", idle_clip);
blender.add_clip("Walk", walk_clip);

// Start with idle
blender.play("Idle");

// Later, smoothly transition to walk over 0.3 seconds
blender.cross_fade("Idle", "Walk", 0.3);
```

## Blend Trees

### 1D Blend Tree (Speed-Based)

Blend animations based on a single parameter:

```rust
use praxis_scene::BlendNode1D;

let mut blend_tree = BlendNode1D::new();
blend_tree.add_clip("Idle", 0.0);   // At speed 0
blend_tree.add_clip("Walk", 0.5);   // At speed 0.5
blend_tree.add_clip("Run", 1.0);    // At speed 1.0

blender.add_blend_tree("Movement", blend_tree.into());
blender.activate_blend_tree("Movement");

// Update based on player speed
let speed = calculate_player_speed();
blender.set_blend_parameter("Movement", speed.clamp(0.0, 1.0));
```

### 2D Blend Tree (Directional Movement)

Blend animations in 2D space:

```rust
use praxis_scene::BlendNode2D;

let mut blend_tree = BlendNode2D::new();
blend_tree.add_clip("Forward", 0.0, 1.0);
blend_tree.add_clip("Back", 0.0, -1.0);
blend_tree.add_clip("Left", -1.0, 0.0);
blend_tree.add_clip("Right", 1.0, 0.0);

blender.add_blend_tree("Locomotion", blend_tree.into());
blender.activate_blend_tree("Locomotion");

// Update based on input direction
let direction = get_movement_direction();  // Returns (x, y)
blender.set_blend_parameters_2d("Locomotion", direction.x, direction.y);
```

## Layered Animation

Play different animations on different parts of the skeleton:

```rust
use praxis_scene::{AnimationLayer, BoneMask, LayerBlendMode};

// Base layer: full body walk
blender.play("Walk");

// Create upper body mask
let mut upper_body_mask = BoneMask::with_bone_count(skeleton.bone_count());

// Find spine bone and enable it and all children
if let Some(spine_idx) = skeleton.find_bone("Spine") {
    upper_body_mask.enable_bone_and_children_with_skeleton(spine_idx, &skeleton);
}

// Add layer for upper body animation
let mut upper_layer = AnimationLayer::new(1.0);
upper_layer.set_mask(upper_body_mask);
upper_layer.set_blend_mode(LayerBlendMode::Override);

blender.add_layer(upper_layer);
blender.play_on_layer(0, "Aim");

// Result: Character walks with lower body, aims with upper body
```

## Creating Animations Programmatically

### Manual Animation Creation

```rust
use praxis_scene::{AnimationClip, Keyframe};
use std::f32::consts::PI;

let mut clip = AnimationClip::new("CustomAnimation", 2.0);

// Add rotation keyframes for arm bone (index 1)
clip.add_rotation_keyframe(1, 0.0, Quat::IDENTITY);
clip.add_rotation_keyframe(1, 0.5, Quat::from_rotation_z(PI / 2.0));
clip.add_rotation_keyframe(1, 1.0, Quat::from_rotation_z(PI));
clip.add_rotation_keyframe(1, 1.5, Quat::from_rotation_z(PI / 2.0));
clip.add_rotation_keyframe(1, 2.0, Quat::IDENTITY);

// Add translation keyframes for another bone
clip.add_translation_keyframe(2, 0.0, Vec3::ZERO);
clip.add_translation_keyframe(2, 1.0, Vec3::new(0.0, 2.0, 0.0));
clip.add_translation_keyframe(2, 2.0, Vec3::ZERO);

player.add_clip("CustomAnimation", clip);
```

### Procedural Animation

```rust
fn generate_bounce_animation(height: f32, duration: f32) -> AnimationClip {
    let mut clip = AnimationClip::new("Bounce", duration);
    let steps = 10;
    
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let time = t * duration;
        
        // Sine wave for smooth bounce
        let y = (t * PI * 2.0).sin().abs() * height;
        
        clip.add_translation_keyframe(0, time, Vec3::new(0.0, y, 0.0));
    }
    
    clip
}

let bounce = generate_bounce_animation(2.0, 1.0);
player.add_clip("Bounce", bounce);
```

## Common Patterns

### Character Controller Animation

```rust
fn update_character_animation(
    mut query: Query<(&CharacterController, &mut AnimationBlender)>,
) {
    for (controller, mut blender) in query.iter_mut() {
        let speed = controller.velocity.length();
        
        // Idle when stationary
        if speed < 0.1 {
            if blender.current_clip() != Some("Idle") {
                blender.cross_fade_to("Idle", 0.2);
            }
        }
        // Walk/Run based on speed
        else {
            let normalized_speed = (speed / controller.max_speed).clamp(0.0, 1.0);
            blender.set_blend_parameter("Movement", normalized_speed);
        }
    }
}
```

### Jump Animation with State Machine

```rust
#[derive(Clone, Copy, PartialEq)]
enum AnimState {
    Idle,
    Walking,
    Jumping,
    Falling,
}

fn update_jump_animation(
    mut query: Query<(&mut AnimState, &mut AnimationPlayer, &Velocity)>,
) {
    for (mut state, mut player, velocity) in query.iter_mut() {
        let new_state = if velocity.y > 0.5 {
            AnimState::Jumping
        } else if velocity.y < -0.5 {
            AnimState::Falling
        } else if velocity.xz().length() > 0.1 {
            AnimState::Walking
        } else {
            AnimState::Idle
        };
        
        if new_state != *state {
            match new_state {
                AnimState::Idle => player.play("Idle"),
                AnimState::Walking => player.play("Walk"),
                AnimState::Jumping => player.play("Jump"),
                AnimState::Falling => player.play("Fall"),
            }
            *state = new_state;
        }
    }
}
```

### Additive Animations

Apply additional animations on top of base animations:

```rust
use praxis_scene::AdditiveBlendNode;

let mut additive = AdditiveBlendNode::new();
additive.set_base("Walk");
additive.set_additive("Recoil");
additive.set_weight(1.0);

blender.add_blend_tree("CombatMovement", additive.into());
blender.activate_blend_tree("CombatMovement");

// Trigger recoil
fn on_weapon_fire(mut query: Query<&mut AnimationBlender>) {
    for mut blender in query.iter_mut() {
        blender.play_on_layer(1, "Recoil");
        blender.set_layer_weight(1, 1.0);
    }
}
```

## Performance Tips

### Optimize Animation Updates

```rust
use praxis_ecs::{Query, Visibility, Transform, Changed};

// Only update visible characters
fn update_visible_animations(
    mut query: Query<(&mut AnimationPlayer, &Visibility), Changed<Transform>>,
) {
    for (mut player, visibility) in query.iter_mut() {
        if !visibility.is_visible() {
            continue;  // Skip hidden entities
        }
        
        // Animation update happens in update_animations system
        // This is just an example of visibility-based optimization
    }
}
```

### Distance-Based LOD

```rust
use praxis_ecs::{Query, Transform, Camera, With};

fn animation_lod(
    mut query: Query<(&GlobalTransform, &mut AnimationPlayer)>,
    camera: Query<&GlobalTransform, With<Camera>>,
) {
    let Ok(camera_transform) = camera.get_single() else {
        return;
    };
    let camera_pos = camera_transform.translation();
    
    for (transform, mut player) in query.iter_mut() {
        let distance = transform.translation().distance(camera_pos);
        
        // Reduce update rate for distant characters
        if distance > 50.0 {
            player.set_speed(0.5);  // Half speed
        } else if distance > 20.0 {
            player.set_speed(0.75);
        } else {
            player.set_speed(1.0);
        }
    }
}
```

### Bone Count Optimization

```rust
// Use simpler skeletons for background characters
fn spawn_background_character(world: &mut World) {
    let simple_skeleton = load_lod_skeleton("character_lod.gltf");
    // 20 bones instead of 50
    world.spawn((simple_skeleton, player, pose));
}
```

## Debugging

### Visualize Bone Hierarchy

```rust
fn debug_draw_skeleton(
    query: Query<(&Skeleton, &AnimatedPose)>,
    mut debug_lines: ResMut<DebugLines>,
) {
    for (skeleton, pose) in query.iter() {
        for (i, bone) in skeleton.bones().iter().enumerate() {
            if let Some(parent_idx) = bone.parent_index {
                let bone_pos = pose.world_transform(i).translation();
                let parent_pos = pose.world_transform(parent_idx).translation();
                
                debug_lines.line(parent_pos, bone_pos, Color::GREEN);
            }
        }
    }
}
```

### Log Animation State

```rust
fn log_animation_state(query: Query<(&Name, &AnimationPlayer)>) {
    for (name, player) in query.iter() {
        if let Some(clip_name) = player.current_clip_name() {
            tracing::debug!(
                "{}: playing '{}' at {:.2}s / {:.2}s (speed: {:.2}x)",
                name.as_str(),
                clip_name,
                player.current_time(),
                player.current_duration(),
                player.speed()
            );
        }
    }
}
```

## Design Rationale and Tradeoffs

### Why Skeletal Animation?

**Decision**: Use skeletal (skinned mesh) animation as the primary animation system

**Rationale**:
- **Memory efficient**: Single skeleton animates entire mesh; one animation works for all instances
- **Flexible**: Blending, layering, and procedural modifications work naturally
- **Industry standard**: Compatible with all 3D authoring tools (Blender, Maya, etc.)
- **Performance**: GPU skinning offloads bone transform calculations from CPU

**Alternatives Considered**:

| Technique | Pros | Cons | Why Not Primary |
|-----------|------|------|-----------------|
| **Vertex animation** | Simple, no bones | Huge memory cost, poor blending | Only for special effects |
| **Sprite-based 2D** | Very fast, simple | Not 3D, limited angles | Different use case |
| **Procedural only** | Infinite variety, adaptive | Hard to author, uncanny valley | Supplement, not replacement |
| **Motion capture retargeting** | Realistic motion | Expensive data, large files | Can layer on top |

**Key Insight**: Skeletal animation is the best balance of flexibility, performance, and tooling support for 3D games.

### Animation Data Structure

**Decision**: Keyframe-based animation clips with interpolated transforms

**Data Layout**:
```rust
AnimationClip {
    duration: f32,
    tracks: HashMap<BoneIndex, Track>,
}

Track {
    translation_keyframes: Vec<(time, Vec3)>,
    rotation_keyframes: Vec<(time, Quat)>,
    scale_keyframes: Vec<(time, Vec3)>,
}
```

**Why Separate Translation/Rotation/Scale Tracks?**
- Different interpolation methods (LERP vs SLERP vs NLERP)
- Independent keyframe timing (rotation may need more keys than translation)
- Memory efficiency (omit scale track if always 1.0)

**Why HashMap for Bone Tracks?**
- Sparse data: Not all bones animated in every clip (walk doesn't move fingers)
- Fast lookup: O(1) bone access during playback
- Memory: Only store animated bones

**Alternatives Considered**:
1. **Array of tracks**: Wasteful memory for sparse animations
2. **Compressed animation**: Better memory but slow decompression; defer until needed
3. **Curve-based**: More expressive but harder to author and slower to evaluate

### Interpolation Strategy

**Decision**: Linear interpolation (LERP) for translation/scale, Spherical Linear Interpolation (SLERP) for rotation

**Why SLERP for Rotations?**
- Quaternion LERP causes "swooping" artifacts
- SLERP produces constant angular velocity
- Normalized LERP (NLERP) is cheaper but has slight speed variation

**Performance Cost**: SLERP is ~3x slower than LERP
- Mitigation: Worth it for quality; rotation is most visible component
- Alternative: NLERP as opt-in for performance-critical animations

**Why LERP for Translation/Scale?**
- Linear motion is natural for position changes
- Scale interpolation rarely critical (most objects don't scale)
- LERP is fast (3 multiply-adds)

### Skeleton and Bone Hierarchy

**Decision**: Tree structure with parent-child relationships, bind pose, and inverse bind matrices

**Core Concepts**:
- **Bind Pose**: Default "T-pose" or "A-pose" where mesh was authored
- **Inverse Bind Matrix**: Transforms from bone space to mesh space
- **Hierarchy**: Children inherit parent transforms (shoulder → elbow → wrist)

**Why Inverse Bind Matrices?**
```
Final Bone Transform = GlobalTransform × InverseBindMatrix
```
- Allows arbitrary bind poses without re-authoring meshes
- Standard in all 3D engines and export formats (GLTF, FBX)

**Alternatives Considered**:
1. **Flat bone list**: Rejected - no automatic shoulder→elbow inheritance
2. **Graph structure**: Rejected - skeletons are always trees, not arbitrary graphs
3. **No bind pose**: Rejected - incompatible with artist workflows

### Animation Blending Architecture

**Decision**: Weighted blending system with blend trees and layers

**Three Blending Approaches**:

1. **Simple Weighted Blending**: Play multiple clips with weights
   - Use case: Quick transitions (walk 70% + run 30%)
   - Cost: O(n) for n clips
   
2. **Blend Trees**: Structured blending with 1D/2D parameters
   - Use case: Locomotion (idle-walk-run based on speed)
   - Cost: O(log n) with spatial acceleration
   
3. **Layered Animation**: Different animations on different bone groups
   - Use case: Walk with legs, aim with torso
   - Cost: O(layers × bones)

**Why Three Systems?**
- Different use cases have different optimal approaches
- Simple blending for quick needs, blend trees for complex locomotion, layers for body separation
- Users can choose complexity level based on needs

**Blend Tree Design: 1D vs 2D**

**1D Blend Tree** (single parameter):
```rust
// Speed: 0.0 → 1.0
0.0: Idle
0.5: Walk  
1.0: Run
// Parameter 0.25 → 50% Idle, 50% Walk
```

**2D Blend Tree** (two parameters):
```rust
// Direction: (x, y)
(0, 1): Forward
(0, -1): Back
(-1, 0): Left
(1, 0): Right
// Bilinear interpolation for diagonals
```

**Why Limited to 2D?**
- 3D blend spaces are exponentially complex and hard to visualize
- Most gameplay needs fit in 1D or 2D (speed, direction)
- Higher dimensions can be achieved by nesting blend trees

**Tradeoff**: 2D blend trees require more keyframe samples but provide smooth omnidirectional movement

### Layered Animation and Masking

**Decision**: Bone masks define which bones each layer affects

**Use Case**: Upper body aiming while lower body walks
```rust
Layer 0 (base): Walk animation, full body
Layer 1 (additive): Aim animation, upper body only (spine and above)
```

**Mask Design**:
- Bitmask: One bit per bone (64 bones = 8 bytes)
- Efficient: Bitwise operations for mask checks
- Hierarchical enable: "Enable spine and all children"

**Layer Blend Modes**:
1. **Override**: Replace base layer (upper body uses aim, ignores walk)
2. **Additive**: Add to base layer (lean animation added to run)

**Why Both Modes?**
- Override: When replacement is needed (aiming completely controls arms)
- Additive: When combination is needed (breathing added to idle)

**Alternatives Considered**:
1. **Per-bone weights**: More flexible but harder to manage (64 float values)
2. **Automatic masking**: Rejected - artists need control
3. **Blend shapes**: Different system (morph targets), complementary

### Animation State Machines vs Direct Control

**Decision**: Provide direct control API; users build state machines if needed

**Why Not Built-in State Machine?**
- Different games need different state machine logic (conditions, transitions)
- ECS-based state management is flexible and inspectable
- Overengineering for simple cases (some games only need play/stop)

**User Pattern**: Build state machine with ECS
```rust
#[derive(Component)]
enum AnimState { Idle, Walk, Run, Jump }

fn update_state(query: Query<(&mut AnimState, &mut AnimationPlayer, &Velocity)>) {
    // User defines state transitions
}
```

**Benefit**: Users see how it works, can customize, no magic

**Tradeoff**: More code for complex state machines vs built-in but inflexible system

### Cross-Fade Implementation

**Decision**: Time-based linear weight interpolation between clips

**Algorithm**:
```
Old clip weight: 1.0 → 0.0 over duration
New clip weight: 0.0 → 1.0 over duration
Blended pose = old_pose × old_weight + new_pose × new_weight
```

**Why Linear Interpolation?**
- Simple and predictable
- Computationally cheap
- Artists can control duration for desired feel

**Alternatives Considered**:
1. **Ease curves**: Smoother but more complex; can add later
2. **Instant switch**: Jarring, no smooth transition
3. **Synchronized cross-fade**: Requires matching keyframes; too restrictive

**Performance**: Cross-fade costs ~2x normal playback during transition
- Acceptable: Transitions are short (0.2-0.5s)
- Both clips evaluated and blended

### GLTF Animation Import

**Decision**: Support GLTF 2.0 animation format as primary import path

**Why GLTF?**
- Industry standard, open spec
- Supports all animation features (keyframes, interpolation, targets)
- All major 3D tools export GLTF (Blender, Maya, 3ds Max)
- JSON + binary = easy to parse and efficient

**Import Process**:
1. Parse GLTF animation samplers (keyframe data)
2. Extract bone targets (translation, rotation, scale)
3. Build AnimationClip with tracks
4. Store skeleton hierarchy from GLTF skin

**Alternatives Considered**:
1. **FBX**: Proprietary, complex binary format, licensing issues
2. **Collada (DAE)**: Verbose XML, falling out of favor
3. **Custom format**: Not worth losing tool compatibility

**Tradeoff**: GLTF import adds parsing complexity but enables standard workflows

### Update System Design

**Decision**: Single `update_animations` system runs all animation players

**System Responsibilities**:
1. Advance animation time based on delta_time and speed
2. Evaluate keyframes at current time
3. Interpolate between keyframes
4. Apply blending/layering
5. Update bone transforms in AnimatedPose
6. Handle looping and end-of-clip logic

**Why Single System?**
- Guarantees consistent update order
- Batch processing for cache efficiency
- Simplifies scheduling (no inter-dependencies)

**Performance Optimization**: Only update if playing
```rust
query.iter_mut()
    .filter(|player| player.is_playing())
    .for_each(|player| player.update(dt));
```

**Tradeoff**: All animations update in same system vs per-entity update flexibility

### GPU vs CPU Skinning

**Decision**: CPU skinning for now, GPU skinning as future optimization

**Current (CPU Skinning)**:
- Calculate final bone matrices on CPU
- Upload to GPU as uniform buffer
- Vertex shader applies matrices to vertices

**Future (GPU Skinning)**:
- Store bone matrices in texture or SSBO
- Vertex shader fetches and applies directly
- Saves CPU-GPU transfer bandwidth

**Why Start With CPU?**
- Simpler implementation, easier debugging
- Sufficient for indie game scales (10-100 animated characters)
- Can profile and optimize to GPU later if needed

**GPU Skinning Benefits** (when implemented):
- 2-3x reduction in CPU time for skinning
- Scales to 1000+ animated characters
- Required for large crowd scenes

**Tradeoff**: CPU skinning limits scale but simplifies initial implementation

### Procedural Animation Integration

**Decision**: Allow mixing keyframe and procedural animation

**Use Cases**:
- IK for foot placement on uneven terrain (procedural)
- Look-at target for head tracking (procedural)
- Breathing cycles (procedural sine wave)

**Integration Point**: After keyframe evaluation, before final output
```
Keyframe Animation → Procedural Modifications → Final Pose
```

**Why This Order?**
- Keyframes provide base motion (walk cycle)
- Procedural adds adaptation (foot adjusts to ground)
- Natural artist-to-programmer workflow

**Alternative (rejected)**: Procedural-first would require animators to work around code

### Performance Characteristics

**Typical Costs** (per animated character per frame):

| Operation | Time | Notes |
|-----------|------|-------|
| Keyframe lookup | 0.1-0.5μs | Binary search in keyframe array |
| Interpolation | 1-2μs | LERP/SLERP for all bones |
| Blending (2 clips) | 2-4μs | Weighted sum of poses |
| Hierarchy update | 2-5μs | Transform propagation |
| Skinning (CPU) | 10-50μs | Depends on bone count |
| **Total** | **15-60μs** | **~16k characters at 60fps budget** |

**Optimization Strategies**:
1. **LOD**: Reduce bone count for distant characters
2. **Update rate**: Update distant characters at 30fps or 15fps
3. **Culling**: Don't update off-screen characters
4. **GPU skinning**: Offload skinning to GPU (future)

**When to Profile**:
- >100 animated characters on screen
- >50 bones per skeleton
- >5 active animation clips per character
- Complex blend trees with many samples

### Additive Animation Theory

**Decision**: Support additive blending for layering subtle animations

**Additive Formula**:
```
Final = Base + (Additive - ReferencePose)
```

**Use Cases**:
- Breathing on top of idle
- Recoil on top of aiming
- Damage reactions on top of movement

**Why Additive vs Override?**
- Additive preserves base motion structure
- Override replaces entirely
- Additive can be scaled (weight 0.5 = half strength)

**Reference Pose**: Usually frame 0 of additive clip or bind pose
- Defines the "zero point" for the additive offset

**Tradeoff**: Requires carefully authored additive animations (subtle motions work best)

## Troubleshooting

### Animation Not Playing

**Problem**: Animation doesn't appear to move

**Solutions**:
- Verify `update_animations` system is in schedule
- Check that `AnimationPlayer` is in "Playing" state
- Ensure skeleton bone count matches animation tracks
- Confirm pose component is attached

### Jerky Animation

**Problem**: Animation looks choppy

**Solutions**:
- Increase keyframe count in animation
- Check frame rate isn't dropping
- Verify delta_time is being passed correctly
- Use SLERP for rotations (automatic in system)

### Wrong Pose After Loading

**Problem**: Character appears distorted after loading GLTF

**Solutions**:
- Check inverse bind matrices are computed
- Verify skeleton hierarchy is correct
- Ensure bind pose matches authoring tool
- Check for coordinate system differences (Y-up vs Z-up)

## Examples

See working examples:
- `examples/skeletal_animation_demo.rs` - Basic animation
- `examples/animation_blending_demo.rs` - Blending and transitions
- `examples/gltf_animation_loader_demo.rs` - GLTF workflow
- `examples/animation_advanced_demo.rs` - Advanced features

Run with:
```bash
cargo run --example skeletal_animation_demo
```

## See Also

- **Comprehensive Animation Documentation:**
  - [Skeletal Basics](animation/skeletal-basics.md) - Core architecture, data structures, and GLTF workflow
  - [Blending](animation/blending.md) - Cross-fades, blend trees, layered animation, and additive blending
  - [Advanced Features](animation/advanced-features.md) - IK, retargeting, additive blending, and root motion
- [Animation Concepts](../concepts/animation.md) - Theory and architecture
- [praxis_scene README](../../crates/praxis_scene/README.md) - API documentation
