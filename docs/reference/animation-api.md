# Animation API Reference

API reference for skeletal animation system in Praxis.

## Components

### Skeleton

Defines bone hierarchy for skeletal animation.

```rust
#[derive(Component)]
pub struct Skeleton { /* ... */ }
```

**Methods:**
- `new(bones: Vec<Bone>) -> Self`
- `bone(index: usize) -> Option<&Bone>`
- `bone_mut(index: usize) -> Option<&mut Bone>`
- `bone_count() -> usize`
- `find_bone(name: &str) -> Option<usize>`
- `inverse_bind_matrix(index: usize) -> Option<&Mat4>`

### Bone

Individual bone in skeleton hierarchy.

```rust
pub struct Bone {
    pub name: String,
    pub parent_index: Option<usize>,
    pub bind_pose_translation: Vec3,
    pub bind_pose_rotation: Quat,
    pub bind_pose_scale: Vec3,
}
```

**Methods:**
- `new(name: String, parent: Option<usize>) -> Self`
- `with_bind_pose(name, parent, translation, rotation, scale) -> Self`
- `bind_pose_matrix() -> Mat4`

### AnimationPlayer

Controls animation playback.

```rust
#[derive(Component)]
pub struct AnimationPlayer { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `add_clip(name: String, clip: AnimationClip)`
- `remove_clip(name: &str)`
- `play(name: &str)` - Start playing animation
- `play_once(name: &str)` - Play without looping
- `stop(name: &str)` - Stop playing
- `pause(name: &str)` - Pause playback
- `resume(name: &str)` - Resume paused animation
- `set_speed(name: &str, speed: f32)` - Adjust playback speed
- `set_weight(name: &str, weight: f32)` - Set blend weight (0.0-1.0)
- `is_playing(name: &str) -> bool`
- `clips() -> &HashMap<String, AnimationClip>`
- `update(delta_time: f32)` - Advance playback time
- `evaluate(skeleton: &Skeleton) -> AnimatedPose`

### AnimationClip

Keyframe animation data.

```rust
pub struct AnimationClip { /* ... */ }
```

**Methods:**
- `new(name: String, duration: f32) -> Self`
- `add_translation_keyframe(bone_idx: usize, time: f32, value: Vec3)`
- `add_rotation_keyframe(bone_idx: usize, time: f32, value: Quat)`
- `add_scale_keyframe(bone_idx: usize, time: f32, value: Vec3)`
- `duration() -> f32`
- `name() -> &str`
- `track_count() -> usize`

### AnimatedPose

Computed bone transforms for rendering.

```rust
#[derive(Component)]
pub struct AnimatedPose { /* ... */ }
```

**Methods:**
- `new(bone_count: usize) -> Self`
- `from_skeleton(skeleton: &Skeleton) -> Self`
- `local_transform(bone_idx: usize) -> Option<&Mat4>`
- `world_transform(bone_idx: usize) -> Option<&Mat4>`
- `skinning_matrix(bone_idx: usize) -> Option<&Mat4>`
- `set_local_transform(bone_idx: usize, transform: Mat4)`
- `update_world_transforms(skeleton: &Skeleton)`
- `update_skinning_matrices(skeleton: &Skeleton)`

## Advanced Animation

### AnimationBlender

Advanced blending with cross-fades and blend trees.

```rust
#[derive(Component)]
pub struct AnimationBlender { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `add_clip(name: String, clip: AnimationClip)`
- `play(name: &str)`
- `cross_fade(from: &str, to: &str, duration: f32)` - Smooth transition
- `add_blend_tree(name: String, tree: BlendNode)` - 1D or 2D blend tree
- `activate_blend_tree(name: &str)`
- `set_blend_parameter(name: &str, value: f32)` - For 1D blend trees
- `set_blend_parameters_2d(name: &str, x: f32, y: f32)` - For 2D blend trees
- `add_layer(layer: AnimationLayer)` - Add animation layer
- `play_on_layer(layer_idx: usize, clip: &str)`
- `update(delta_time: f32)`
- `evaluate(skeleton: &Skeleton) -> AnimatedPose`

### BlendNode

Node-based animation blending.

```rust
pub enum BlendNode {
    BlendNode1D(BlendNode1D),
    BlendNode2D(BlendNode2D),
    Additive(AdditiveBlendNode),
}
```

### BlendNode1D

1D blend tree (e.g., walk-run transition based on speed).

```rust
pub struct BlendNode1D { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `add_clip(name: &str, position: f32)` - Add clip at parameter position
- `remove_clip(name: &str)`
- `evaluate(parameter: f32) -> BlendResult`

### BlendNode2D

2D blend tree (e.g., directional movement).

```rust
pub struct BlendNode2D { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `add_clip(name: &str, x: f32, y: f32)` - Add clip at 2D position
- `remove_clip(name: &str)`
- `evaluate(x: f32, y: f32) -> BlendResult`

### AdditiveBlendNode

Additive animation blending.

```rust
pub struct AdditiveBlendNode { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `set_base(clip_name: &str)` - Base animation
- `set_additive(clip_name: &str)` - Additive layer
- `set_weight(weight: f32)` - Blend weight

### AnimationLayer

Layer for multi-layer animation.

```rust
pub struct AnimationLayer {
    pub weight: f32,
    pub blend_mode: LayerBlendMode,
    pub mask: Option<BoneMask>,
}
```

**Methods:**
- `new(weight: f32) -> Self`
- `set_mask(mask: BoneMask)` - Bone masking for partial body animation
- `set_blend_mode(mode: LayerBlendMode)`

### LayerBlendMode

How layers blend together.

```rust
pub enum LayerBlendMode {
    Override,  // Replace base animation
    Additive,  // Add to base animation
}
```

### BoneMask

Selects which bones are affected by a layer.

```rust
pub struct BoneMask { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `with_bone_count(count: usize) -> Self`
- `enable_bone(bone_idx: usize)`
- `disable_bone(bone_idx: usize)`
- `enable_bone_and_children(bone_idx: usize, skeleton: &Skeleton)`
- `is_bone_enabled(bone_idx: usize) -> bool`
- `clear()`

## GLTF Integration

### Loading Animated Models

```rust
use praxis_assets::GltfLoader;

let loader = GltfLoader::new();
let asset = loader.load_gltf("assets/models/character.gltf")?;

// Access skeleton
for skin in &asset.skins {
    let skeleton = skin.skeleton.clone();
}

// Access animations
for animation in &asset.animations {
    let clip = animation.clip.clone();
}
```

## Systems

### update_animations

Updates animation players and computes poses.

```rust
pub fn update_animations(
    delta_time: f32,
    query: Query<(&Skeleton, &mut AnimationPlayer, &mut AnimatedPose)>,
)
```

**Schedule:** Run every frame during update phase.

### update_animation_blenders

Updates advanced animation blenders.

```rust
pub fn update_animation_blenders(
    delta_time: f32,
    query: Query<(&Skeleton, &mut AnimationBlender, &mut AnimatedPose)>,
)
```

**Schedule:** Run every frame during update phase.

## Common Patterns

### Basic Animation

```rust
use praxis_scene::{Skeleton, AnimationPlayer, AnimationClip, AnimatedPose, Bone};

// Create skeleton
let skeleton = Skeleton::new(vec![
    Bone::with_bind_pose("Root", None, Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
    Bone::with_bind_pose("Arm", Some(0), Vec3::new(1.0, 0.0, 0.0), Quat::IDENTITY, Vec3::ONE),
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
let pose = AnimatedPose::new(skeleton.bone_count());
world.spawn((skeleton, player, pose));
```

### Cross-Fade Transition

```rust
use praxis_scene::AnimationBlender;

let mut blender = AnimationBlender::new();
blender.add_clip("Idle", idle_clip);
blender.add_clip("Walk", walk_clip);

// Start with idle
blender.play("Idle");

// Smoothly transition to walk over 0.3 seconds
blender.cross_fade("Idle", "Walk", 0.3);
```

### 1D Blend Tree (Walk-Run)

```rust
use praxis_scene::BlendNode1D;

let mut blend_tree = BlendNode1D::new();
blend_tree.add_clip("Idle", 0.0);
blend_tree.add_clip("Walk", 0.5);
blend_tree.add_clip("Run", 1.0);

blender.add_blend_tree("Movement", blend_tree.into());
blender.activate_blend_tree("Movement");

// Control speed dynamically
blender.set_blend_parameter("Movement", current_speed / max_speed);
```

### 2D Blend Tree (Directional Movement)

```rust
use praxis_scene::BlendNode2D;

let mut blend_tree = BlendNode2D::new();
blend_tree.add_clip("Forward", 0.0, 1.0);
blend_tree.add_clip("Back", 0.0, -1.0);
blend_tree.add_clip("Left", -1.0, 0.0);
blend_tree.add_clip("Right", 1.0, 0.0);

blender.add_blend_tree("Locomotion", blend_tree.into());
blender.activate_blend_tree("Locomotion");

// Set direction
blender.set_blend_parameters_2d("Locomotion", move_x, move_y);
```

### Layered Animation (Upper Body Override)

```rust
use praxis_scene::{AnimationLayer, BoneMask, LayerBlendMode};

// Play walk on base layer
blender.play("Walk");

// Create upper body mask
let mut mask = BoneMask::with_bone_count(skeleton.bone_count());
mask.enable_bone_and_children(spine_bone_index, &skeleton);

// Add layer for upper body
let mut layer = AnimationLayer::new(1.0);
layer.set_mask(mask);
layer.set_blend_mode(LayerBlendMode::Override);

blender.add_layer(layer);
blender.play_on_layer(0, "Aim");  // Character walks while aiming
```

### Additive Animation (Recoil)

```rust
use praxis_scene::AdditiveBlendNode;

let mut additive = AdditiveBlendNode::new();
additive.set_base("Walk");
additive.set_additive("Recoil");
additive.set_weight(1.0);

blender.add_blend_tree("CombatMovement", additive.into());
blender.activate_blend_tree("CombatMovement");
```

## Performance Considerations

### Optimization Tips

1. **Weight Threshold**: Animations with weight < 0.001 are skipped
2. **Early Exit**: Stopped animations don't process
3. **Bone Masking**: Only evaluate enabled bones in layers
4. **LOD**: Reduce animation update frequency for distant characters
5. **Caching**: Reuse AnimatedPose allocations

### Scalability Guidelines

- **<50 bones**: Excellent (typical humanoid)
- **50-100 bones**: Good (detailed character)
- **100-200 bones**: Acceptable (facial animation)
- **>200 bones**: Consider LOD or culling

### Update Frequency

```rust
// Full rate for important characters
schedule.add_systems(update_animations);

// Reduced rate for background characters
// Run every 2-3 frames with interpolation
```

## See Also

- [Animation Guide](../guides/animation.md) - Quick start guide
- [Animation Guides](../guides/animation/README.md) - Comprehensive animation documentation
- [Animation Concepts](../concepts/animation.md) - Theory and architecture
- [Animation Learning Path](../learning-paths/animation.md) - Structured learning progression
- [praxis_scene Crate](../../crates/praxis_scene/README.md) - Crate documentation
