//! Interactive animation demonstration with blend transitions.
//!
//! This example showcases:
//! - Animated character with skeletal animation
//! - Smooth cross-fade transitions between animations
//! - 1D blend trees for speed-based blending
//! - Interactive controls to switch between animations
//! - Real-time animation state updates
//!
//! Controls:
//! - 1 - Switch to Idle animation (with cross-fade)
//! - 2 - Switch to Walk animation (with cross-fade)
//! - 3 - Switch to Run animation (with cross-fade)
//! - 4 - Activate speed blend tree
//! - Arrow Up/Down - Adjust speed parameter (in blend tree mode)
//! - ESC - Exit demo

use praxis_ecs::{Component, Query, World};
use praxis_input::{InputState, KeyCode};
use praxis_math::{Quat, Vec3};
use praxis_scene::{
    AnimatedPose, AnimationBlender, AnimationClip, BlendNode1D, Bone, Skeleton,
    update_animation_blenders,
};
use praxis_utils::{info, Result};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
enum AnimationMode {
    Idle,
    Walk,
    Run,
    BlendTree,
}

struct DemoState {
    current_mode: AnimationMode,
    speed_parameter: f32,
    last_mode_change: Instant,
    elapsed_time: f32,
}

impl DemoState {
    fn new() -> Self {
        Self {
            current_mode: AnimationMode::Idle,
            speed_parameter: 0.0,
            last_mode_change: Instant::now(),
            elapsed_time: 0.0,
        }
    }
}

/// Marker component for the animated character
#[derive(Component)]
struct AnimatedCharacter;

fn create_character_skeleton() -> Skeleton {
    // Create a simple humanoid-like skeleton
    Skeleton::new(vec![
        // 0: Root/Hips
        Bone::with_bind_pose(
            "Hips".to_string(),
            None,
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        // 1: Spine
        Bone::with_bind_pose(
            "Spine".to_string(),
            Some(0),
            Vec3::new(0.0, 0.8, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        // 2: Chest
        Bone::with_bind_pose(
            "Chest".to_string(),
            Some(1),
            Vec3::new(0.0, 0.5, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        // 3: Head
        Bone::with_bind_pose(
            "Head".to_string(),
            Some(2),
            Vec3::new(0.0, 0.8, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        // 4: Left Shoulder
        Bone::with_bind_pose(
            "LeftShoulder".to_string(),
            Some(2),
            Vec3::new(-0.4, 0.6, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        // 5: Left Arm
        Bone::with_bind_pose(
            "LeftArm".to_string(),
            Some(4),
            Vec3::new(-0.6, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        // 6: Right Shoulder
        Bone::with_bind_pose(
            "RightShoulder".to_string(),
            Some(2),
            Vec3::new(0.4, 0.6, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        // 7: Right Arm
        Bone::with_bind_pose(
            "RightArm".to_string(),
            Some(6),
            Vec3::new(0.6, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        // 8: Left Leg
        Bone::with_bind_pose(
            "LeftLeg".to_string(),
            Some(0),
            Vec3::new(-0.2, -0.8, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        // 9: Right Leg
        Bone::with_bind_pose(
            "RightLeg".to_string(),
            Some(0),
            Vec3::new(0.2, -0.8, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
    ])
}

fn create_idle_animation() -> AnimationClip {
    let mut clip = AnimationClip::new("Idle".to_string(), 2.0);
    
    // Subtle breathing motion on chest
    clip.add_translation_keyframe(2, 0.0, Vec3::new(0.0, 0.5, 0.0));
    clip.add_translation_keyframe(2, 1.0, Vec3::new(0.0, 0.55, 0.0));
    clip.add_translation_keyframe(2, 2.0, Vec3::new(0.0, 0.5, 0.0));
    
    // Slight head bob
    clip.add_translation_keyframe(3, 0.0, Vec3::new(0.0, 0.8, 0.0));
    clip.add_translation_keyframe(3, 1.0, Vec3::new(0.0, 0.82, 0.0));
    clip.add_translation_keyframe(3, 2.0, Vec3::new(0.0, 0.8, 0.0));
    
    clip
}

fn create_walk_animation() -> AnimationClip {
    let mut clip = AnimationClip::new("Walk".to_string(), 1.2);
    
    // Hip movement
    clip.add_translation_keyframe(0, 0.0, Vec3::ZERO);
    clip.add_translation_keyframe(0, 0.6, Vec3::new(0.0, 0.05, 0.0));
    clip.add_translation_keyframe(0, 1.2, Vec3::ZERO);
    
    // Spine rotation
    clip.add_rotation_keyframe(1, 0.0, Quat::IDENTITY);
    clip.add_rotation_keyframe(1, 0.6, Quat::from_rotation_y(0.1));
    clip.add_rotation_keyframe(1, 1.2, Quat::IDENTITY);
    
    // Left arm swing
    clip.add_rotation_keyframe(5, 0.0, Quat::from_rotation_z(0.3));
    clip.add_rotation_keyframe(5, 0.6, Quat::from_rotation_z(-0.3));
    clip.add_rotation_keyframe(5, 1.2, Quat::from_rotation_z(0.3));
    
    // Right arm swing (opposite)
    clip.add_rotation_keyframe(7, 0.0, Quat::from_rotation_z(-0.3));
    clip.add_rotation_keyframe(7, 0.6, Quat::from_rotation_z(0.3));
    clip.add_rotation_keyframe(7, 1.2, Quat::from_rotation_z(-0.3));
    
    // Left leg
    clip.add_rotation_keyframe(8, 0.0, Quat::from_rotation_x(-0.5));
    clip.add_rotation_keyframe(8, 0.6, Quat::from_rotation_x(0.5));
    clip.add_rotation_keyframe(8, 1.2, Quat::from_rotation_x(-0.5));
    
    // Right leg (opposite)
    clip.add_rotation_keyframe(9, 0.0, Quat::from_rotation_x(0.5));
    clip.add_rotation_keyframe(9, 0.6, Quat::from_rotation_x(-0.5));
    clip.add_rotation_keyframe(9, 1.2, Quat::from_rotation_x(0.5));
    
    clip
}

fn create_run_animation() -> AnimationClip {
    let mut clip = AnimationClip::new("Run".to_string(), 0.8);
    
    // Hip movement (more pronounced)
    clip.add_translation_keyframe(0, 0.0, Vec3::ZERO);
    clip.add_translation_keyframe(0, 0.4, Vec3::new(0.0, 0.1, 0.0));
    clip.add_translation_keyframe(0, 0.8, Vec3::ZERO);
    
    // Spine rotation (more pronounced)
    clip.add_rotation_keyframe(1, 0.0, Quat::IDENTITY);
    clip.add_rotation_keyframe(1, 0.4, Quat::from_rotation_y(0.2));
    clip.add_rotation_keyframe(1, 0.8, Quat::IDENTITY);
    
    // Left arm swing (aggressive)
    clip.add_rotation_keyframe(5, 0.0, Quat::from_rotation_z(0.8));
    clip.add_rotation_keyframe(5, 0.4, Quat::from_rotation_z(-0.8));
    clip.add_rotation_keyframe(5, 0.8, Quat::from_rotation_z(0.8));
    
    // Right arm swing (aggressive, opposite)
    clip.add_rotation_keyframe(7, 0.0, Quat::from_rotation_z(-0.8));
    clip.add_rotation_keyframe(7, 0.4, Quat::from_rotation_z(0.8));
    clip.add_rotation_keyframe(7, 0.8, Quat::from_rotation_z(-0.8));
    
    // Left leg (aggressive)
    clip.add_rotation_keyframe(8, 0.0, Quat::from_rotation_x(-1.0));
    clip.add_rotation_keyframe(8, 0.4, Quat::from_rotation_x(1.0));
    clip.add_rotation_keyframe(8, 0.8, Quat::from_rotation_x(-1.0));
    
    // Right leg (aggressive, opposite)
    clip.add_rotation_keyframe(9, 0.0, Quat::from_rotation_x(1.0));
    clip.add_rotation_keyframe(9, 0.4, Quat::from_rotation_x(-1.0));
    clip.add_rotation_keyframe(9, 0.8, Quat::from_rotation_x(1.0));
    
    clip
}

fn create_animation_blender() -> AnimationBlender {
    let mut blender = AnimationBlender::new();
    
    // Add all animation clips
    blender.add_clip("Idle", create_idle_animation());
    blender.add_clip("Walk", create_walk_animation());
    blender.add_clip("Run", create_run_animation());
    
    // Create 1D blend tree for speed-based blending
    let mut blend_tree = BlendNode1D::new();
    blend_tree.add_clip("Idle", 0.0);
    blend_tree.add_clip("Walk", 0.5);
    blend_tree.add_clip("Run", 1.0);
    
    blender.add_blend_tree("SpeedBlend", blend_tree.into());
    
    blender
}

fn handle_input(
    input: &InputState,
    demo_state: &mut DemoState,
    query: &mut Query<(&mut AnimationBlender,), praxis_ecs::With<AnimatedCharacter>>,
) {
    let now = Instant::now();
    let time_since_change = now.duration_since(demo_state.last_mode_change).as_secs_f32();
    
    // Prevent rapid mode switching
    if time_since_change < 0.3 {
        return;
    }
    
    let mut mode_changed = false;
    let old_mode = demo_state.current_mode;
    
    // Mode switching
    if input.is_key_just_pressed(KeyCode::Digit1) {
        demo_state.current_mode = AnimationMode::Idle;
        mode_changed = true;
    } else if input.is_key_just_pressed(KeyCode::Digit2) {
        demo_state.current_mode = AnimationMode::Walk;
        mode_changed = true;
    } else if input.is_key_just_pressed(KeyCode::Digit3) {
        demo_state.current_mode = AnimationMode::Run;
        mode_changed = true;
    } else if input.is_key_just_pressed(KeyCode::Digit4) {
        demo_state.current_mode = AnimationMode::BlendTree;
        mode_changed = true;
    }
    
    // Speed parameter adjustment (for blend tree mode)
    if demo_state.current_mode == AnimationMode::BlendTree {
        if input.is_key_pressed(KeyCode::ArrowUp) {
            demo_state.speed_parameter = (demo_state.speed_parameter + 0.02).min(1.0);
        }
        if input.is_key_pressed(KeyCode::ArrowDown) {
            demo_state.speed_parameter = (demo_state.speed_parameter - 0.02).max(0.0);
        }
    }
    
    // Apply mode change to blender
    if mode_changed {
        for (mut blender,) in query.iter_mut() {
            match demo_state.current_mode {
                AnimationMode::Idle => {
                    if old_mode != AnimationMode::Idle {
                        let from = match old_mode {
                            AnimationMode::Walk => "Walk",
                            AnimationMode::Run => "Run",
                            _ => "Idle",
                        };
                        blender.cross_fade(from, "Idle", 0.3);
                        info!("Cross-fading to Idle animation");
                    }
                }
                AnimationMode::Walk => {
                    if old_mode != AnimationMode::Walk {
                        let from = match old_mode {
                            AnimationMode::Idle => "Idle",
                            AnimationMode::Run => "Run",
                            _ => "Walk",
                        };
                        blender.cross_fade(from, "Walk", 0.3);
                        info!("Cross-fading to Walk animation");
                    }
                }
                AnimationMode::Run => {
                    if old_mode != AnimationMode::Run {
                        let from = match old_mode {
                            AnimationMode::Idle => "Idle",
                            AnimationMode::Walk => "Walk",
                            _ => "Run",
                        };
                        blender.cross_fade(from, "Run", 0.3);
                        info!("Cross-fading to Run animation");
                    }
                }
                AnimationMode::BlendTree => {
                    blender.activate_blend_tree("SpeedBlend");
                    info!("Activated speed blend tree (use arrow keys to adjust)");
                }
            }
        }
        demo_state.last_mode_change = now;
    }
    
    // Update blend tree parameter
    if demo_state.current_mode == AnimationMode::BlendTree {
        for (mut blender,) in query.iter_mut() {
            blender.set_blend_parameter("SpeedBlend", demo_state.speed_parameter);
        }
    }
}

fn update_animations(
    delta_time: f32,
    query: &mut Query<(&Skeleton, &mut AnimationBlender, &mut AnimatedPose)>,
) {
    update_animation_blenders(delta_time, query);
}

fn print_status(
    query: &Query<(&AnimatedPose,), praxis_ecs::With<AnimatedCharacter>>,
    demo_state: &DemoState,
    frame: usize,
) {
    // Print status every 60 frames (1 second at 60 FPS)
    if frame % 60 == 0 {
        for (pose,) in query.iter() {
            info!("=== Status Update (t={:.1}s) ===", demo_state.elapsed_time);
            info!("Current mode: {:?}", demo_state.current_mode);
            
            if demo_state.current_mode == AnimationMode::BlendTree {
                info!("Speed parameter: {:.2}", demo_state.speed_parameter);
            }
            
            // Print root bone position
            if let Some(root_transform) = pose.local_transform(0) {
                let pos = root_transform.col(3).truncate();
                info!("Root bone position: ({:.2}, {:.2}, {:.2})", pos.x, pos.y, pos.z);
            }
            
            info!("Animated bones: {}", pose.local_transforms().len());
        }
    }
}

fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;

    info!("=== Animation Demo ===");
    info!("Character Animation with Blend Transitions");
    info!("");
    info!("This demo showcases skeletal animation with smooth transitions:");
    info!("  - Idle, Walk, and Run animations");
    info!("  - Cross-fade blending between animations");
    info!("  - 1D blend tree for speed-based blending");
    info!("");
    info!("Controls:");
    info!("  1 - Switch to Idle animation");
    info!("  2 - Switch to Walk animation");
    info!("  3 - Switch to Run animation");
    info!("  4 - Activate speed blend tree");
    info!("  Arrow Up/Down - Adjust speed (blend tree mode)");
    info!("  ESC - Exit demo");
    info!("");

    let mut world = World::new();
    let mut input_state = InputState::new();
    let mut demo_state = DemoState::new();

    // Create animated character
    let skeleton = create_character_skeleton();
    let mut blender = create_animation_blender();
    
    // Start with idle animation
    blender.play("Idle");
    info!("Started with Idle animation");
    info!("");
    
    let pose = AnimatedPose::new(skeleton.bone_count());
    
    world.spawn((
        skeleton.clone(),
        blender,
        pose,
        AnimatedCharacter,
    ));

    info!("Spawned animated character with {} bones", skeleton.bone_count());
    info!("Running demo for 60 seconds (press ESC to exit)...");
    info!("");

    let mut frame = 0;
    let start_time = Instant::now();
    let frame_duration = Duration::from_millis(16); // 60 FPS

    // Run for 60 seconds or until ESC is pressed
    while demo_state.elapsed_time < 60.0 {
        frame += 1;
        let frame_start = Instant::now();
        
        // Update input
        input_state.update();
        
        // Check for exit
        if input_state.is_key_pressed(KeyCode::Escape) {
            info!("");
            info!("ESC pressed, exiting demo");
            break;
        }
        
        // Handle input
        {
            let mut query = world.query_filtered::<(&mut AnimationBlender,), praxis_ecs::With<AnimatedCharacter>>();
            handle_input(&input_state, &mut demo_state, &mut query);
        }
        
        // Update animations
        let delta_time = 1.0 / 60.0;
        {
            let mut query = world.query::<(&Skeleton, &mut AnimationBlender, &mut AnimatedPose)>();
            update_animations(delta_time, &mut query);
        }
        
        // Print status
        {
            let query = world.query_filtered::<(&AnimatedPose,), praxis_ecs::With<AnimatedCharacter>>();
            print_status(&query, &demo_state, frame);
        }
        
        demo_state.elapsed_time = start_time.elapsed().as_secs_f32();
        
        // Frame rate limiting
        let frame_time = frame_start.elapsed();
        if frame_time < frame_duration {
            std::thread::sleep(frame_duration - frame_time);
        }
    }

    info!("");
    info!("=== Animation Demo Complete ===");
    info!("Total frames: {}", frame);
    info!("Total time: {:.1}s", demo_state.elapsed_time);
    info!("");
    info!("Features demonstrated:");
    info!("  ✓ Skeletal animation with 10-bone humanoid character");
    info!("  ✓ Three distinct animation clips (Idle, Walk, Run)");
    info!("  ✓ Smooth cross-fade transitions between animations");
    info!("  ✓ 1D blend tree for speed-based animation blending");
    info!("  ✓ Real-time parameter adjustment");
    info!("  ✓ Keyframe interpolation (translation, rotation)");

    Ok(())
}
