# Animation System

Skeletal animation and blending in Praxis, providing keyframe-based animation for bone hierarchies.

## Core Components

### Skeleton
Defines the bone hierarchy and bind poses:

```rust
#[derive(Component)]
pub struct Skeleton {
    bones: Vec<Bone>,
    inverse_bind_matrices: Vec<Mat4>,
}

pub struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    pub bind_translation: Vec3,
    pub bind_rotation: Quat,
    pub bind_scale: Vec3,
}
```

### AnimationClip
Stores keyframe data for bone transforms:

```rust
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub bone_tracks: Vec<BoneTrack>,
}

pub struct BoneTrack {
    pub bone_index: usize,
    pub translation_keys: Vec<(f32, Vec3)>,
    pub rotation_keys: Vec<(f32, Quat)>,
    pub scale_keys: Vec<(f32, Vec3)>,
}
```

### AnimationPlayer
Controls playback state:

```rust
#[derive(Component)]
pub struct AnimationPlayer {
    clips: HashMap<String, AnimationClip>,
    playing: HashMap<String, PlaybackState>,
}
```

### AnimatedPose
Final computed bone transforms for GPU skinning:

```rust
#[derive(Component)]
pub struct AnimatedPose {
    local_transforms: Vec<Mat4>,
    world_transforms: Vec<Mat4>,
    skinning_matrices: Vec<Mat4>,
}
```

## Keyframe Interpolation

Transforms are interpolated between keyframes:

| Channel | Method |
|---------|--------|
| Translation | Linear interpolation (lerp) |
| Rotation | Spherical linear interpolation (slerp) |
| Scale | Linear interpolation (lerp) |

## Animation Blending

Multiple animations can play simultaneously with weights:

```rust
player.play("Walk");
player.set_weight("Walk", 0.7);
player.play("Run");
player.set_weight("Run", 0.3);
// Result: 70% walk + 30% run
```

## Advanced Blending (AnimationBlender)

### Cross-Fade Transitions
Smooth transitions between animations:

```rust
blender.cross_fade("Idle", "Walk", 0.3); // 0.3 second transition
```

### 1D Blend Trees
Parameter-driven blending (e.g., speed):

```rust
let mut tree = BlendNode1D::new();
tree.add_clip("Idle", 0.0);
tree.add_clip("Walk", 0.5);
tree.add_clip("Run", 1.0);
blender.set_blend_parameter("Movement", 0.75); // Between walk and run
```

### 2D Blend Trees
Two-axis blending (e.g., directional movement):

```rust
let mut tree = BlendNode2D::new();
tree.add_clip("Forward", 0.0, 1.0);
tree.add_clip("Back", 0.0, -1.0);
tree.add_clip("Left", -1.0, 0.0);
tree.add_clip("Right", 1.0, 0.0);
```

### Animation Layers
Partial skeleton animation with bone masks:

```rust
let mut layer = AnimationLayer::new(1.0);
layer.set_mask(upper_body_mask);
layer.set_blend_mode(LayerBlendMode::Override);
blender.play_on_layer(0, "Aim");
// Result: Lower body walks, upper body aims
```

## Usage Example

```rust
use praxis_scene::{Skeleton, AnimationPlayer, AnimatedPose};

// Setup entity with animation
let skeleton = Skeleton::new(bones);
let mut player = AnimationPlayer::new();
player.add_clip("Walk", walk_clip);
player.play("Walk");

let pose = AnimatedPose::new(skeleton.bone_count());
world.spawn((skeleton, player, pose));

// Update system
fn animation_system(mut query: Query<(&Skeleton, &mut AnimationPlayer, &mut AnimatedPose)>) {
    praxis_scene::update_animations(delta_time, &mut query);
}
```

## GLTF Animation Loading

```rust
let asset = GltfLoader::new().load_gltf("character.gltf")?;

for animation in &asset.animations {
    player.add_clip(animation.name.clone(), animation.clip.clone());
}
```

## See Also

- [Beginner's Guide: Animation System](../beginners-guide.md#animation-system) - Deep dive explanation
- [Animation Learning Path](../learning-paths/animation.md) - Structured learning progression
- [praxis_scene crate](../../crates/praxis_scene/README.md) - API documentation
- [animation_blending_demo](../../examples/animation_blending_demo.rs) - Working example
