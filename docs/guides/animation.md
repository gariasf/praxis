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

- [Animation Concepts](../concepts/animation.md) - Theory and architecture
- [Animation System Deep Dive](../animation_system.md) - Detailed implementation
- [Advanced Animation Features](../animation_advanced_features.md) - IK, retargeting
- [praxis_scene README](../../crates/praxis_scene/README.md) - API documentation
