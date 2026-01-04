# Advanced Animation Integration Guide

This guide demonstrates how to integrate all advanced animation features (IK, retargeting, additive blending, and root motion) into a complete character animation system.

## Overview

We'll build a complete animated character system with:
- Base locomotion animations
- IK for foot placement and hand reaching
- Additive animations for breathing and reactions
- Root motion for character movement
- Animation retargeting for different character models

## Project Structure

```rust
// Character controller
struct CharacterController {
    // Animation state
    skeleton: Skeleton,
    animation_player: AnimationPlayer,
    animation_clips: HashMap<String, AnimationClip>,
    
    // Advanced features
    ik_controller: IkController,
    root_motion_extractor: RootMotionExtractor,
    
    // Additive animations
    breathing_additive: AdditiveAnimation,
    recoil_additive: AdditiveAnimation,
    
    // State
    current_pose: AnimatedPose,
    is_grounded: bool,
    target_object: Option<Vec3>,
}
```

## Step 1: Initialize the System

```rust
use praxis_scene::*;
use praxis_math::{Vec3, Quat};
use std::collections::HashMap;

impl CharacterController {
    fn new(skeleton: Skeleton) -> Self {
        let bone_count = skeleton.bone_count();
        
        // Create animation player
        let mut animation_player = AnimationPlayer::new();
        
        // Load animation clips (from files or procedurally)
        let mut animation_clips = HashMap::new();
        
        // Create IK controller
        let ik_controller = IkController::new();
        
        // Create root motion extractor (assuming root bone is index 0)
        let root_motion_extractor = RootMotionExtractor::new(0)
            .with_translation(true)
            .with_rotation(true);
        
        // Setup additive animations
        let mut breathing_additive = AdditiveAnimation::new(
            "Idle".to_string(),
            "Breathe".to_string()
        )
        .with_weight(0.3)
        .with_mode(AdditiveMode::Local);
        breathing_additive.compute_reference_from_skeleton(&skeleton);
        
        let mut recoil_additive = AdditiveAnimation::new(
            "Aim".to_string(),
            "Recoil".to_string()
        )
        .with_weight(1.0)
        .with_mode(AdditiveMode::Local);
        recoil_additive.compute_reference_from_skeleton(&skeleton);
        
        Self {
            skeleton,
            animation_player,
            animation_clips,
            ik_controller,
            root_motion_extractor,
            breathing_additive,
            recoil_additive,
            current_pose: AnimatedPose::new(bone_count),
            is_grounded: true,
            target_object: None,
        }
    }
}
```

## Step 2: Load and Retarget Animations

```rust
impl CharacterController {
    fn load_animations(&mut self, source_skeleton: &Skeleton, source_clips: &[AnimationClip]) {
        // Create retargeter
        let retargeter = AnimationRetargeter::auto(source_skeleton, &self.skeleton);
        
        // Retarget all animations
        for source_clip in source_clips {
            let retargeted = retargeter.retarget_clip(source_clip, &self.skeleton);
            let name = retargeted.name().to_string();
            
            // Add to player
            self.animation_player.add_clip(name.clone(), retargeted.clone());
            
            // Cache for other uses
            self.animation_clips.insert(name, retargeted);
        }
        
        println!("Loaded {} animations", self.animation_clips.len());
    }
    
    fn load_from_library(
        &mut self,
        library_skeleton: &Skeleton,
        library: &AnimationLibrary
    ) {
        // Load standard animations
        let clips = vec![
            library.get_clip("Idle").unwrap(),
            library.get_clip("Walk").unwrap(),
            library.get_clip("Run").unwrap(),
            library.get_clip("Jump").unwrap(),
        ];
        
        self.load_animations(library_skeleton, &clips);
        
        // Start with idle
        self.animation_player.play("Idle");
    }
}
```

## Step 3: Update Animation State

```rust
impl CharacterController {
    fn update(&mut self, delta_time: f32) {
        // 1. Update animation playback
        self.animation_player.update(delta_time);
        
        // 2. Evaluate base animation
        self.current_pose = self.animation_player.evaluate(&self.skeleton);
        
        // 3. Apply additive animations
        self.apply_additive_animations();
        
        // 4. Apply IK constraints
        self.apply_ik();
        
        // 5. Extract root motion
        self.extract_root_motion();
        
        // 6. Finalize pose
        self.current_pose.update_skinning_matrices(&self.skeleton);
    }
}
```

## Step 4: Implement Additive Animation

```rust
impl CharacterController {
    fn apply_additive_animations(&mut self) {
        // Apply breathing (always)
        if let Some(breathe_clip) = self.animation_clips.get("Breathe") {
            // Use sine wave for continuous breathing
            let breathe_time = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f32() * 0.5)
                .sin()
                .abs();
            
            self.breathing_additive.apply(
                &mut self.current_pose,
                breathe_clip,
                breathe_time,
                &self.skeleton,
            );
        }
        
        // Apply recoil (conditional)
        if self.is_firing() {
            if let Some(recoil_clip) = self.animation_clips.get("Recoil") {
                let recoil_time = self.get_recoil_time();
                
                self.recoil_additive.apply(
                    &mut self.current_pose,
                    recoil_clip,
                    recoil_time,
                    &self.skeleton,
                );
            }
        }
    }
    
    fn is_firing(&self) -> bool {
        // Check if weapon is firing
        // Implementation depends on your game logic
        false
    }
    
    fn get_recoil_time(&self) -> f32 {
        // Get time within recoil animation
        // Implementation depends on your game logic
        0.0
    }
}
```

## Step 5: Implement IK System

```rust
impl CharacterController {
    fn apply_ik(&mut self) {
        // Clear previous constraints
        self.ik_controller.clear_constraints();
        
        // Add foot IK if grounded
        if self.is_grounded {
            self.add_foot_ik();
        }
        
        // Add hand IK if reaching for object
        if let Some(target) = self.target_object {
            self.add_hand_ik(target);
        }
        
        // Apply all constraints
        self.ik_controller.apply(&mut self.current_pose, &self.skeleton);
    }
    
    fn add_foot_ik(&mut self) {
        // Find bone indices (cache these in production)
        let left_foot_idx = self.skeleton.find_bone("LeftFoot").unwrap();
        let right_foot_idx = self.skeleton.find_bone("RightFoot").unwrap();
        
        // Get current foot positions
        let left_foot_pos = self.current_pose
            .world_transform(left_foot_idx)
            .map(|m| m.col(3).truncate())
            .unwrap_or(Vec3::ZERO);
        
        let right_foot_pos = self.current_pose
            .world_transform(right_foot_idx)
            .map(|m| m.col(3).truncate())
            .unwrap_or(Vec3::ZERO);
        
        // Raycast to find ground positions
        let left_ground = self.raycast_ground(left_foot_pos);
        let right_ground = self.raycast_ground(right_foot_pos);
        
        // Create IK constraints with pole targets
        let left_knee_hint = left_foot_pos + Vec3::new(0.0, 0.5, 0.5);
        let right_knee_hint = right_foot_pos + Vec3::new(0.0, 0.5, 0.5);
        
        let left_ik = IkConstraint::new_two_bone(left_foot_idx, left_ground)
            .with_pole_target(left_knee_hint)
            .with_weight(1.0);
        
        let right_ik = IkConstraint::new_two_bone(right_foot_idx, right_ground)
            .with_pole_target(right_knee_hint)
            .with_weight(1.0);
        
        self.ik_controller.add_constraint(left_ik);
        self.ik_controller.add_constraint(right_ik);
    }
    
    fn add_hand_ik(&mut self, target: Vec3) {
        // Find hand bone
        let right_hand_idx = self.skeleton.find_bone("RightHand").unwrap();
        
        // Create IK constraint
        let hand_ik = IkConstraint::new_two_bone(right_hand_idx, target)
            .with_weight(1.0);
        
        self.ik_controller.add_constraint(hand_ik);
    }
    
    fn raycast_ground(&self, position: Vec3) -> Vec3 {
        // Perform raycast to find ground
        // This is a placeholder - implement with your physics system
        Vec3::new(position.x, 0.0, position.z)
    }
}
```

## Step 6: Extract and Apply Root Motion

```rust
impl CharacterController {
    fn extract_root_motion(&mut self) {
        self.root_motion_extractor.extract(&mut self.current_pose, &self.skeleton);
    }
    
    fn apply_root_motion_to_transform(&mut self, transform: &mut Transform) {
        let motion = self.root_motion_extractor.motion();
        
        if !motion.consumed {
            // Apply translation
            transform.translation += motion.translation;
            
            // Apply rotation
            transform.rotation *= motion.rotation;
            
            // Mark as consumed
            self.root_motion_extractor.motion_mut().consume();
        }
    }
    
    fn get_motion_delta(&self) -> (Vec3, Quat) {
        let motion = self.root_motion_extractor.motion();
        (motion.translation, motion.rotation)
    }
}
```

## Step 7: Character State Machine Integration

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharacterState {
    Idle,
    Walking,
    Running,
    Jumping,
    Falling,
}

impl CharacterController {
    fn update_state(&mut self, state: CharacterState, delta_time: f32) {
        match state {
            CharacterState::Idle => {
                self.animation_player.play("Idle");
                self.root_motion_extractor.with_translation(false);
            }
            CharacterState::Walking => {
                self.animation_player.play("Walk");
                self.root_motion_extractor.with_translation(true);
            }
            CharacterState::Running => {
                self.animation_player.play("Run");
                self.root_motion_extractor.with_translation(true);
            }
            CharacterState::Jumping => {
                self.animation_player.play("Jump");
                self.animation_player.set_looping("Jump", false);
                self.is_grounded = false;
            }
            CharacterState::Falling => {
                self.animation_player.play("Fall");
                self.is_grounded = false;
            }
        }
        
        // Update animation
        self.update(delta_time);
    }
}
```

## Step 8: ECS Integration

```rust
use praxis_ecs::{World, Query, Component};

#[derive(Component)]
struct CharacterAnimationComponent {
    controller: CharacterController,
}

fn animation_update_system(
    delta_time: f32,
    mut query: Query<(&mut CharacterAnimationComponent, &mut Transform)>
) {
    for (mut anim, mut transform) in query.iter_mut() {
        // Update animation
        anim.controller.update(delta_time);
        
        // Apply root motion
        anim.controller.apply_root_motion_to_transform(&mut transform);
    }
}

fn ik_target_update_system(
    mut query: Query<(&mut CharacterAnimationComponent, &Transform)>,
    target_query: Query<(&InteractableObject, &Transform)>,
) {
    for (mut anim, character_transform) in query.iter_mut() {
        // Find nearest interactable object
        let mut nearest_target = None;
        let mut nearest_distance = f32::MAX;
        
        for (_, target_transform) in target_query.iter() {
            let distance = character_transform
                .translation
                .distance(target_transform.translation);
            
            if distance < nearest_distance && distance < 2.0 {
                nearest_distance = distance;
                nearest_target = Some(target_transform.translation);
            }
        }
        
        anim.controller.target_object = nearest_target;
    }
}

#[derive(Component)]
struct InteractableObject;
```

## Step 9: Blend Tree Integration

For more complex movement, integrate with blend trees:

```rust
impl CharacterController {
    fn setup_locomotion_blend_tree(&mut self) {
        use praxis_scene::BlendNode1D;
        
        let mut blend_tree = BlendNode1D::new();
        blend_tree.add_clip("Idle", 0.0);
        blend_tree.add_clip("Walk", 0.5);
        blend_tree.add_clip("Run", 1.0);
        
        // This would integrate with AnimationBlender
        // Implementation depends on whether you're using AnimationPlayer
        // or AnimationBlender for your base system
    }
    
    fn update_locomotion_blend(&mut self, speed: f32) {
        // Normalize speed to 0-1 range
        let max_speed = 10.0;
        let normalized_speed = (speed / max_speed).clamp(0.0, 1.0);
        
        // Update blend parameter
        // blend_tree.set_parameter(normalized_speed);
    }
}
```

## Complete Usage Example

```rust
fn main() {
    // Setup
    let mut world = World::new();
    
    // Load character skeleton and animations
    let skeleton = load_skeleton("character.skel");
    let library_skeleton = load_skeleton("animation_library.skel");
    let animation_library = load_animation_library("animations.lib");
    
    // Create character controller
    let mut controller = CharacterController::new(skeleton);
    controller.load_from_library(&library_skeleton, &animation_library);
    
    // Spawn character entity
    let character = world.spawn((
        CharacterAnimationComponent { controller },
        Transform::default(),
    ));
    
    // Game loop
    let mut state = CharacterState::Idle;
    let mut time = 0.0;
    
    loop {
        let delta_time = 0.016; // 60 FPS
        time += delta_time;
        
        // Update character state based on input
        state = get_character_state_from_input();
        
        // Update animation system
        animation_update_system(delta_time, &mut world.query());
        
        // Update IK targets
        ik_target_update_system(&mut world.query(), &world.query());
        
        // Render
        render_system(&world);
    }
}

fn get_character_state_from_input() -> CharacterState {
    // Read input and determine state
    // Placeholder implementation
    CharacterState::Walking
}

fn render_system(world: &World) {
    // Render all entities
    // Implementation depends on your rendering system
}

fn load_skeleton(path: &str) -> Skeleton {
    // Load skeleton from file
    // Placeholder implementation
    Skeleton::new(vec![])
}

fn load_animation_library(path: &str) -> AnimationLibrary {
    // Load animation library
    // Placeholder implementation
    AnimationLibrary::new()
}

struct AnimationLibrary {
    clips: HashMap<String, AnimationClip>,
}

impl AnimationLibrary {
    fn new() -> Self {
        Self {
            clips: HashMap::new(),
        }
    }
    
    fn get_clip(&self, name: &str) -> Option<&AnimationClip> {
        self.clips.get(name)
    }
}

#[derive(Component)]
struct Transform {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}
```

## Best Practices

### 1. Performance Optimization
```rust
// Cache bone indices
struct BoneIndices {
    root: usize,
    spine: usize,
    head: usize,
    left_hand: usize,
    right_hand: usize,
    left_foot: usize,
    right_foot: usize,
}

impl BoneIndices {
    fn from_skeleton(skeleton: &Skeleton) -> Self {
        Self {
            root: skeleton.find_bone("Root").unwrap(),
            spine: skeleton.find_bone("Spine").unwrap(),
            head: skeleton.find_bone("Head").unwrap(),
            left_hand: skeleton.find_bone("LeftHand").unwrap(),
            right_hand: skeleton.find_bone("RightHand").unwrap(),
            left_foot: skeleton.find_bone("LeftFoot").unwrap(),
            right_foot: skeleton.find_bone("RightFoot").unwrap(),
        }
    }
}
```

### 2. Error Handling
```rust
impl CharacterController {
    fn safe_find_bone(&self, name: &str) -> Result<usize, String> {
        self.skeleton
            .find_bone(name)
            .ok_or_else(|| format!("Bone '{}' not found in skeleton", name))
    }
}
```

### 3. Debug Visualization
```rust
#[cfg(debug_assertions)]
impl CharacterController {
    fn debug_draw(&self, debug_renderer: &mut DebugRenderer) {
        // Draw IK targets
        for constraint in self.ik_controller.constraints() {
            debug_renderer.draw_sphere(constraint.target(), 0.1, Color::RED);
            
            if let Some(pole) = constraint.pole_target {
                debug_renderer.draw_sphere(pole, 0.05, Color::BLUE);
            }
        }
        
        // Draw root motion
        let motion = self.root_motion_extractor.motion();
        debug_renderer.draw_arrow(
            Vec3::ZERO,
            motion.translation,
            Color::GREEN,
        );
    }
}
```

## Troubleshooting

### Issue: Feet sliding on ground
**Solution**: Increase foot IK weight and ensure ground detection is accurate

### Issue: Hand not reaching target
**Solution**: Check if target is within arm reach distance

### Issue: Choppy movement
**Solution**: Ensure root motion is being applied every frame

### Issue: Animation looks wrong after retargeting
**Solution**: Verify bone mapping is correct, may need manual mapping

## Conclusion

This integration guide provides a complete foundation for advanced character animation. The system is modular and can be extended with additional features like:

- Facial animation
- Finger IK
- Dynamic ragdoll transitions
- Animation events
- Motion matching

See the [Advanced Animation Features](../animation_advanced_features.md) documentation for more details on individual systems.
