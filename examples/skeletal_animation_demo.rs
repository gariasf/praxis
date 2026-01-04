//! Skeletal animation system demonstration.
//!
//! This example demonstrates the skeletal animation system with:
//! - Creating a skeleton with multiple bones
//! - Defining animation clips with keyframes
//! - Playing and controlling animations
//! - Keyframe interpolation
//! - Animation looping and blending

use praxis_ecs::World;
use praxis_math::{Quat, Vec3};
use praxis_scene::{AnimatedPose, AnimationClip, AnimationPlayer, Bone, Skeleton};
use praxis_utils::timing::FrameTimer;

/// Creates a simple skeleton with 3 bones: root, spine, and head.
fn create_skeleton() -> Skeleton {
    Skeleton::new(vec![
        Bone::with_bind_pose(
            "Root".to_string(),
            None,
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        Bone::with_bind_pose(
            "Spine".to_string(),
            Some(0), // Parent is Root
            Vec3::new(0.0, 1.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        Bone::with_bind_pose(
            "Head".to_string(),
            Some(1), // Parent is Spine
            Vec3::new(0.0, 1.5, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
    ])
}

/// Creates a "walk" animation that moves the root bone forward.
fn create_walk_animation() -> AnimationClip {
    let mut clip = AnimationClip::new("Walk".to_string(), 2.0);

    // Animate the root bone moving forward
    clip.add_translation_keyframe(0, 0.0, Vec3::ZERO);
    clip.add_translation_keyframe(0, 1.0, Vec3::new(2.0, 0.0, 0.0));
    clip.add_translation_keyframe(0, 2.0, Vec3::new(4.0, 0.0, 0.0));

    // Animate the spine rotating
    clip.add_rotation_keyframe(1, 0.0, Quat::IDENTITY);
    clip.add_rotation_keyframe(1, 0.5, Quat::from_rotation_z(std::f32::consts::PI / 16.0));
    clip.add_rotation_keyframe(1, 1.0, Quat::IDENTITY);
    clip.add_rotation_keyframe(1, 1.5, Quat::from_rotation_z(-std::f32::consts::PI / 16.0));
    clip.add_rotation_keyframe(1, 2.0, Quat::IDENTITY);

    clip
}

/// Creates an "idle" animation with subtle head bobbing.
fn create_idle_animation() -> AnimationClip {
    let mut clip = AnimationClip::new("Idle".to_string(), 2.0);

    // Subtle head bobbing
    clip.add_translation_keyframe(2, 0.0, Vec3::new(0.0, 1.5, 0.0));
    clip.add_translation_keyframe(2, 1.0, Vec3::new(0.0, 1.55, 0.0));
    clip.add_translation_keyframe(2, 2.0, Vec3::new(0.0, 1.5, 0.0));

    // Slight head rotation
    clip.add_rotation_keyframe(2, 0.0, Quat::IDENTITY);
    clip.add_rotation_keyframe(2, 1.0, Quat::from_rotation_y(std::f32::consts::PI / 32.0));
    clip.add_rotation_keyframe(2, 2.0, Quat::IDENTITY);

    clip
}

/// Updates animations for all entities with animation components.
///
/// This function iterates over all entities with Skeleton, AnimationPlayer, and AnimatedPose
/// components and advances their animation playback.
fn update_animations_manual(world: &mut World, delta_time: f32) {
    let mut query = world.query::<(&Skeleton, &mut AnimationPlayer, &mut AnimatedPose)>();
    for (skeleton, mut player, mut pose) in query.iter_mut(world.inner_mut()) {
        // Update animation playback times
        player.update(delta_time);

        // Evaluate the current animation state and update the pose
        *pose = player.evaluate(skeleton);
    }
}

/// Prints animation state for debugging.
fn debug_animation_state(world: &mut World) {
    let mut query = world.query::<(&AnimationPlayer, &AnimatedPose)>();
    for (player, pose) in query.iter(world.inner()) {
        let playing_clips = player.playing_clips();
        if !playing_clips.is_empty() {
            println!("Playing animations: {playing_clips:?}");

            // Print first bone's local transform
            if let Some(transform) = pose.local_transform(0) {
                let translation = transform.col(3).truncate();
                println!("  Root bone position: {translation:?}");
            }
        }
    }
}

fn main() {
    println!("=== Skeletal Animation Demo ===\n");

    // Initialize ECS world
    let mut world = World::new();

    // Create skeleton
    let skeleton = create_skeleton();
    println!("Created skeleton with {} bones:", skeleton.bone_count());
    for i in 0..skeleton.bone_count() {
        if let Some(bone) = skeleton.bone(i) {
            println!(
                "  - {} (parent: {:?})",
                bone.name,
                bone.parent_index
                    .map(|p| skeleton.bone(p).unwrap().name.as_str())
            );
        }
    }
    println!();

    // Create animation player with clips
    let mut player = AnimationPlayer::new();
    player.add_clip("Walk".to_string(), create_walk_animation());
    player.add_clip("Idle".to_string(), create_idle_animation());
    println!("Created animation clips:");
    for (name, clip) in player.clips() {
        println!(
            "  - {} (duration: {:.2}s, {} tracks)",
            name,
            clip.duration(),
            clip.track_count()
        );
    }
    println!();

    // Create initial pose
    let pose = AnimatedPose::new(skeleton.bone_count());

    // Spawn animated entity
    let entity = world.spawn((skeleton.clone(), player, pose));
    println!("Spawned animated entity: {entity:?}\n");

    // Simulate animation for a few frames
    let mut timer = FrameTimer::new();
    let mut frame = 0;

    println!("=== Starting Animation Playback ===\n");

    // Play the walk animation
    {
        let mut query = world.query::<&mut AnimationPlayer>();
        for mut player in query.iter_mut(world.inner_mut()) {
            player.play("Walk");
            player.set_looping("Walk", true);
            println!("Started playing 'Walk' animation (looping)\n");
        }
    }

    // Run simulation for 5 seconds (approximately 300 frames at 60 FPS)
    let simulation_duration = 5.0;
    let target_fps = 60.0;
    let frame_time = 1.0 / target_fps;
    let total_frames = (simulation_duration * target_fps) as usize;

    println!("Running simulation for {simulation_duration:.1}s...\n");

    for _ in 0..total_frames {
        frame += 1;
        timer.tick();

        // Update animations
        update_animations_manual(&mut world, frame_time);

        // Print state every 60 frames (once per second)
        if frame % 60 == 0 {
            println!(
                "--- Frame {} (t={:.2}s) ---",
                frame,
                frame as f32 * frame_time
            );
            debug_animation_state(&mut world);
            println!();
        }
    }

    println!("=== Testing Animation Control ===\n");

    // Test pause/resume
    {
        let mut query = world.query::<&mut AnimationPlayer>();
        for mut player in query.iter_mut(world.inner_mut()) {
            println!("Pausing animation...");
            player.pause("Walk");

            if let Some(time) = player.current_time("Walk") {
                println!("  Paused at time: {time:.2}s");
            }
        }
    }

    // Run a few frames while paused
    for _ in 0..5 {
        update_animations_manual(&mut world, frame_time);
    }

    {
        let mut query = world.query::<&AnimationPlayer>();
        for player in query.iter(world.inner()) {
            if let Some(time) = player.current_time("Walk") {
                println!("  Time after 5 frames: {time:.2}s (should be unchanged)");
            }
        }
    }

    // Resume animation
    {
        let mut query = world.query::<&mut AnimationPlayer>();
        for mut player in query.iter_mut(world.inner_mut()) {
            println!("\nResuming animation...");
            player.resume("Walk");
        }
    }

    println!();

    // Test animation speed
    {
        let mut query = world.query::<&mut AnimationPlayer>();
        for mut player in query.iter_mut(world.inner_mut()) {
            println!("Setting animation speed to 2x...");
            player.set_speed("Walk", 2.0);
        }
    }

    // Run a few frames with 2x speed
    for _ in 0..10 {
        update_animations_manual(&mut world, frame_time);
    }

    {
        let mut query = world.query::<&AnimationPlayer>();
        for player in query.iter(world.inner()) {
            if let Some(time) = player.current_time("Walk") {
                println!("  Time after 10 frames at 2x speed: {time:.2}s");
            }
        }
    }

    println!();

    // Test stopping animation
    {
        let mut query = world.query::<&mut AnimationPlayer>();
        for mut player in query.iter_mut(world.inner_mut()) {
            println!("Stopping animation...");
            player.stop("Walk");
            println!("  Is playing: {}", player.is_playing("Walk"));
        }
    }

    println!();

    // Test animation blending with multiple clips
    {
        let mut query = world.query::<&mut AnimationPlayer>();
        for mut player in query.iter_mut(world.inner_mut()) {
            println!("Playing both 'Walk' and 'Idle' animations...");
            player.play("Walk");
            player.set_weight("Walk", 0.7);
            player.play("Idle");
            player.set_weight("Idle", 0.3);
            println!("  Walk weight: 0.7");
            println!("  Idle weight: 0.3");
        }
    }

    // Run blended animation for a bit
    for _ in 0..60 {
        update_animations_manual(&mut world, frame_time);
    }

    {
        let mut query = world.query::<(&AnimationPlayer, &AnimatedPose)>();
        for (player, pose) in query.iter(world.inner()) {
            println!("\nAfter 1 second of blended animation:");
            println!("  Playing clips: {:?}", player.playing_clips());
            if let Some(transform) = pose.local_transform(0) {
                let translation = transform.col(3).truncate();
                println!("  Root bone position: {translation:?}");
            }
        }
    }

    println!("\n=== Animation Demo Complete ===");
    println!("\nKey Features Demonstrated:");
    println!("  ✓ Skeleton with hierarchical bones");
    println!("  ✓ Animation clips with keyframe interpolation");
    println!("  ✓ Translation, rotation, and scale animation");
    println!("  ✓ Animation playback control (play, pause, resume, stop)");
    println!("  ✓ Animation speed control");
    println!("  ✓ Looping animations");
    println!("  ✓ Multiple animations and blending");
}
