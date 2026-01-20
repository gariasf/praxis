# Exercise 48: Skeletal Animation System

**Difficulty**: 🔴 Advanced | **Estimated Time**: 6-8h | **Subsystem**: Animation

## Overview

Implement skeletal animation with bone hierarchies, animation clips, and GPU skinning. Essential for character animation in games.

## Learning Objectives

- Understand skeletal animation principles
- Implement bone hierarchy and transforms
- Learn keyframe interpolation
- Implement GPU skinning with vertex shaders

## Requirements

### Functional Requirements

1. **Skeleton Structure**
   - Bone hierarchy (parent-child relationships)
   - Bind pose (rest position)
   - Inverse bind pose matrices

2. **Animation Clips**
   - Keyframe storage (time, position, rotation, scale)
   - Interpolation between keyframes (linear, cubic)
   - Looping and one-shot playback

3. **Animation Player**
   - Play/pause/stop controls
   - Playback speed
   - Time tracking
   - Multiple animation instances

4. **GPU Skinning**
   - Compute final bone matrices
   - Upload to shader as uniform buffer
   - Vertex shader applies up to 4 bone influences per vertex

### Non-Functional Requirements

- **Performance**: 100 animated characters at 60 FPS
- **Memory**: Bone palette < 256 bones (typical GPU limit)
- **Quality**: Smooth interpolation, no popping

## API Design

```rust
pub struct Skeleton {
    pub bones: Vec<Bone>,
    pub root_bone: usize,
}

pub struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    pub inverse_bind_pose: Mat4,
}

pub struct AnimationClip {
    pub duration: f32,
    pub tracks: Vec<AnimationTrack>,
}

pub struct AnimationTrack {
    pub bone_index: usize,
    pub keyframes: Vec<Keyframe>,
}

pub struct Keyframe {
    pub time: f32,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

pub struct AnimationPlayer {
    pub current_time: f32,
    pub playback_speed: f32,
    pub looping: bool,
}

impl AnimationPlayer {
    pub fn play(&mut self);
    pub fn pause(&mut self);
    pub fn stop(&mut self);
    pub fn update(&mut self, delta_time: f32);
    pub fn sample_animation(&self, clip: &AnimationClip, skeleton: &Skeleton) -> Vec<Mat4>;
}
```

## Validation Criteria

### Correctness
- [ ] Bone transforms propagate correctly through hierarchy
- [ ] Keyframe interpolation smooth
- [ ] Multiple animations don't interfere
- [ ] Inverse bind pose correctly applied

### Performance
- [ ] 100 characters @ 60 FPS
- [ ] Bone matrix computation < 1ms per character
- [ ] GPU skinning outperforms CPU skinning

## Test Cases

```rust
#[test]
fn test_skeleton_hierarchy() {
    let mut skeleton = Skeleton::new();
    
    let root = skeleton.add_bone(Bone {
        name: "root".to_string(),
        parent: None,
        inverse_bind_pose: Mat4::IDENTITY,
    });
    
    let child = skeleton.add_bone(Bone {
        name: "child".to_string(),
        parent: Some(root),
        inverse_bind_pose: Mat4::IDENTITY,
    });
    
    assert_eq!(skeleton.bones[child].parent, Some(root));
}

#[test]
fn test_keyframe_interpolation() {
    let kf1 = Keyframe {
        time: 0.0,
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    
    let kf2 = Keyframe {
        time: 1.0,
        position: Vec3::new(10.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    
    let result = interpolate_keyframes(&kf1, &kf2, 0.5);
    assert!((result.position.x - 5.0).abs() < 0.001);
}

#[test]
fn test_animation_looping() {
    let clip = AnimationClip {
        duration: 1.0,
        tracks: vec![],
    };
    
    let mut player = AnimationPlayer::new();
    player.looping = true;
    player.current_time = 0.9;
    
    player.update(0.2); // Should wrap to 0.1
    assert!((player.current_time - 0.1).abs() < 0.001);
}
```

## Performance Targets

| Scenario | Target |
|----------|--------|
| 10 characters, 30 bones | 60 FPS |
| 100 characters, 30 bones | 60 FPS |
| 1 character, 200 bones | 60 FPS |
| Bone matrix update | < 1ms |

## Hints & Guidance

### Bone Matrix Calculation
```
For each bone:
1. Get local transform from keyframes
2. Compute world transform: parent_world * local
3. Apply inverse bind pose: world * inverse_bind
4. Upload to GPU
```

### Keyframe Search
Use binary search to find surrounding keyframes:
```rust
fn find_keyframes(track: &AnimationTrack, time: f32) -> (usize, usize) {
    // Binary search for time
    // Return indices of prev and next keyframes
}
```

### GPU Skinning Shader
```glsl
// Vertex shader
layout(location = 0) in vec3 position;
layout(location = 1) in vec4 bone_indices;  // Up to 4 bones
layout(location = 2) in vec4 bone_weights;  // Must sum to 1.0

uniform mat4 bone_matrices[256];

void main() {
    mat4 skin_matrix = 
        bone_matrices[int(bone_indices.x)] * bone_weights.x +
        bone_matrices[int(bone_indices.y)] * bone_weights.y +
        bone_matrices[int(bone_indices.z)] * bone_weights.z +
        bone_matrices[int(bone_indices.w)] * bone_weights.w;
    
    vec4 skinned_pos = skin_matrix * vec4(position, 1.0);
    gl_Position = projection * view * skinned_pos;
}
```

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use glam::{Mat4, Quat, Vec3};

#[derive(Clone)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
}

#[derive(Clone)]
pub struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    pub inverse_bind_pose: Mat4,
}

#[derive(Clone)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub tracks: Vec<AnimationTrack>,
}

#[derive(Clone)]
pub struct AnimationTrack {
    pub bone_index: usize,
    pub keyframes: Vec<Keyframe>,
}

#[derive(Clone, Copy)]
pub struct Keyframe {
    pub time: f32,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

pub struct AnimationPlayer {
    pub current_time: f32,
    pub playback_speed: f32,
    pub looping: bool,
    pub playing: bool,
}

impl AnimationPlayer {
    pub fn new() -> Self {
        Self {
            current_time: 0.0,
            playback_speed: 1.0,
            looping: true,
            playing: false,
        }
    }
    
    pub fn play(&mut self) {
        self.playing = true;
    }
    
    pub fn pause(&mut self) {
        self.playing = false;
    }
    
    pub fn stop(&mut self) {
        self.playing = false;
        self.current_time = 0.0;
    }
    
    pub fn update(&mut self, delta_time: f32, clip: &AnimationClip) {
        if !self.playing {
            return;
        }
        
        self.current_time += delta_time * self.playback_speed;
        
        if self.current_time >= clip.duration {
            if self.looping {
                self.current_time = self.current_time % clip.duration;
            } else {
                self.current_time = clip.duration;
                self.playing = false;
            }
        }
    }
    
    pub fn sample_animation(&self, clip: &AnimationClip, skeleton: &Skeleton) -> Vec<Mat4> {
        let mut local_transforms = vec![Mat4::IDENTITY; skeleton.bones.len()];
        
        // Sample each track
        for track in &clip.tracks {
            if let Some(transform) = self.sample_track(track) {
                local_transforms[track.bone_index] = transform;
            }
        }
        
        // Compute world transforms
        let mut world_transforms = vec![Mat4::IDENTITY; skeleton.bones.len()];
        self.compute_world_transforms(skeleton, &local_transforms, &mut world_transforms);
        
        // Apply inverse bind pose
        let mut final_matrices = vec![Mat4::IDENTITY; skeleton.bones.len()];
        for i in 0..skeleton.bones.len() {
            final_matrices[i] = world_transforms[i] * skeleton.bones[i].inverse_bind_pose;
        }
        
        final_matrices
    }
    
    fn sample_track(&self, track: &AnimationTrack) -> Option<Mat4> {
        if track.keyframes.is_empty() {
            return None;
        }
        
        // Find surrounding keyframes
        let (prev_idx, next_idx) = find_keyframe_indices(&track.keyframes, self.current_time);
        
        if prev_idx == next_idx {
            // Exact keyframe or only one keyframe
            let kf = &track.keyframes[prev_idx];
            return Some(Mat4::from_scale_rotation_translation(
                kf.scale,
                kf.rotation,
                kf.position,
            ));
        }
        
        // Interpolate between keyframes
        let prev_kf = &track.keyframes[prev_idx];
        let next_kf = &track.keyframes[next_idx];
        
        let time_diff = next_kf.time - prev_kf.time;
        let t = if time_diff > 0.0 {
            (self.current_time - prev_kf.time) / time_diff
        } else {
            0.0
        };
        
        let position = prev_kf.position.lerp(next_kf.position, t);
        let rotation = prev_kf.rotation.slerp(next_kf.rotation, t);
        let scale = prev_kf.scale.lerp(next_kf.scale, t);
        
        Some(Mat4::from_scale_rotation_translation(scale, rotation, position))
    }
    
    fn compute_world_transforms(
        &self,
        skeleton: &Skeleton,
        local_transforms: &[Mat4],
        world_transforms: &mut [Mat4],
    ) {
        for (i, bone) in skeleton.bones.iter().enumerate() {
            if let Some(parent_idx) = bone.parent {
                world_transforms[i] = world_transforms[parent_idx] * local_transforms[i];
            } else {
                world_transforms[i] = local_transforms[i];
            }
        }
    }
}

fn find_keyframe_indices(keyframes: &[Keyframe], time: f32) -> (usize, usize) {
    if keyframes.is_empty() {
        return (0, 0);
    }
    
    if time <= keyframes[0].time {
        return (0, 0);
    }
    
    if time >= keyframes[keyframes.len() - 1].time {
        let last = keyframes.len() - 1;
        return (last, last);
    }
    
    // Binary search
    let mut left = 0;
    let mut right = keyframes.len() - 1;
    
    while left < right {
        let mid = (left + right) / 2;
        if keyframes[mid].time < time {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    
    if left > 0 && keyframes[left].time > time {
        (left - 1, left)
    } else {
        (left, left)
    }
}

// Example usage
fn example() {
    // Create skeleton
    let skeleton = Skeleton {
        bones: vec![
            Bone {
                name: "root".to_string(),
                parent: None,
                inverse_bind_pose: Mat4::IDENTITY,
            },
            Bone {
                name: "spine".to_string(),
                parent: Some(0),
                inverse_bind_pose: Mat4::IDENTITY,
            },
        ],
    };
    
    // Create animation clip
    let clip = AnimationClip {
        name: "walk".to_string(),
        duration: 1.0,
        tracks: vec![
            AnimationTrack {
                bone_index: 1,
                keyframes: vec![
                    Keyframe {
                        time: 0.0,
                        position: Vec3::ZERO,
                        rotation: Quat::IDENTITY,
                        scale: Vec3::ONE,
                    },
                    Keyframe {
                        time: 0.5,
                        position: Vec3::new(0.0, 1.0, 0.0),
                        rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
                        scale: Vec3::ONE,
                    },
                    Keyframe {
                        time: 1.0,
                        position: Vec3::ZERO,
                        rotation: Quat::IDENTITY,
                        scale: Vec3::ONE,
                    },
                ],
            },
        ],
    };
    
    // Play animation
    let mut player = AnimationPlayer::new();
    player.play();
    
    // Update loop
    let delta_time = 0.016; // 60 FPS
    player.update(delta_time, &clip);
    
    // Get bone matrices for rendering
    let bone_matrices = player.sample_animation(&clip, &skeleton);
    
    // Upload to GPU...
}
```

</details>

## Related Resources

- [Praxis Animation Guide](../../guides/animation/skeletal-basics.md)
- [Skeletal Animation on Wikipedia](https://en.wikipedia.org/wiki/Skeletal_animation)
- [GPU Gems - Skinning](https://developer.nvidia.com/gpugems/gpugems/part-iii-materials/chapter-20-efficient-shadow-volume-rendering)

## Next Steps

- Implement animation blending (Exercise 49)
- Add IK solver (Exercise 50)
- Study GLTF animation loading
