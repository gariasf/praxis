//! Demonstrates advanced animation blending features.
//!
//! This example showcases:
//! - Cross-fade transitions between animations
//! - 1D blend trees for speed-based blending
//! - 2D blend trees for directional movement
//! - Layered animation with bone masking
//! - Additive animation blending

use praxis_ecs::World;
use praxis_math::{Quat, Vec3};
use praxis_scene::{
    AnimatedPose, AnimationBlender, AnimationClip, AnimationLayer, 
    BlendNode1D, BlendNode2D, AdditiveBlendNode, BoneMask, Bone, Skeleton,
    LayerBlendMode, update_animation_blenders,
};

fn main() {
    println!("=== Animation Blending Demo ===\n");
    
    // Create skeleton
    let skeleton = create_simple_skeleton();
    
    // Create animation clips
    let idle_clip = create_idle_animation();
    let walk_clip = create_walk_animation();
    let run_clip = create_run_animation();
    let wave_clip = create_wave_animation();
    
    println!("Created skeleton with {} bones", skeleton.bone_count());
    println!("Created 4 animation clips: Idle, Walk, Run, Wave\n");
    
    // Demo 1: Cross-fade transitions
    demo_cross_fade(
        &skeleton,
        idle_clip.clone(),
        walk_clip.clone(),
    );
    
    // Demo 2: 1D Blend Tree
    demo_blend_tree_1d(
        &skeleton,
        idle_clip.clone(),
        walk_clip.clone(),
        run_clip.clone(),
    );
    
    // Demo 3: 2D Blend Tree
    demo_blend_tree_2d(&skeleton);
    
    // Demo 4: Layered Animation
    demo_layered_animation(
        &skeleton,
        walk_clip.clone(),
        wave_clip,
    );
    
    // Demo 5: Additive Blending
    demo_additive_blending(&skeleton, walk_clip, run_clip);
    
    println!("\n=== Demo Complete ===");
}

fn create_simple_skeleton() -> Skeleton {
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
            Some(0),
            Vec3::new(0.0, 1.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        Bone::with_bind_pose(
            "LeftArm".to_string(),
            Some(1),
            Vec3::new(-0.5, 1.5, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        Bone::with_bind_pose(
            "RightArm".to_string(),
            Some(1),
            Vec3::new(0.5, 1.5, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
    ])
}

fn create_idle_animation() -> AnimationClip {
    let mut clip = AnimationClip::new("Idle".to_string(), 2.0);
    
    // Subtle breathing motion
    clip.add_translation_keyframe(1, 0.0, Vec3::new(0.0, 1.0, 0.0));
    clip.add_translation_keyframe(1, 1.0, Vec3::new(0.0, 1.05, 0.0));
    clip.add_translation_keyframe(1, 2.0, Vec3::new(0.0, 1.0, 0.0));
    
    clip
}

fn create_walk_animation() -> AnimationClip {
    let mut clip = AnimationClip::new("Walk".to_string(), 1.0);
    
    // Walking motion
    clip.add_translation_keyframe(0, 0.0, Vec3::ZERO);
    clip.add_translation_keyframe(0, 0.5, Vec3::new(0.5, 0.0, 0.0));
    clip.add_translation_keyframe(0, 1.0, Vec3::new(1.0, 0.0, 0.0));
    
    // Arm swing
    clip.add_rotation_keyframe(2, 0.0, Quat::IDENTITY);
    clip.add_rotation_keyframe(2, 0.5, Quat::from_rotation_z(0.5));
    clip.add_rotation_keyframe(2, 1.0, Quat::IDENTITY);
    
    clip
}

fn create_run_animation() -> AnimationClip {
    let mut clip = AnimationClip::new("Run".to_string(), 0.6);
    
    // Running motion (faster)
    clip.add_translation_keyframe(0, 0.0, Vec3::ZERO);
    clip.add_translation_keyframe(0, 0.3, Vec3::new(0.8, 0.0, 0.0));
    clip.add_translation_keyframe(0, 0.6, Vec3::new(1.6, 0.0, 0.0));
    
    // More aggressive arm swing
    clip.add_rotation_keyframe(2, 0.0, Quat::IDENTITY);
    clip.add_rotation_keyframe(2, 0.3, Quat::from_rotation_z(1.0));
    clip.add_rotation_keyframe(2, 0.6, Quat::IDENTITY);
    
    clip
}

fn create_wave_animation() -> AnimationClip {
    let mut clip = AnimationClip::new("Wave".to_string(), 1.5);
    
    // Waving motion for right arm
    clip.add_rotation_keyframe(3, 0.0, Quat::IDENTITY);
    clip.add_rotation_keyframe(3, 0.5, Quat::from_rotation_z(1.57)); // 90 degrees
    clip.add_rotation_keyframe(3, 1.0, Quat::from_rotation_z(0.78)); // 45 degrees
    clip.add_rotation_keyframe(3, 1.5, Quat::IDENTITY);
    
    clip
}

fn demo_cross_fade(skeleton: &Skeleton, idle_clip: AnimationClip, walk_clip: AnimationClip) {
    println!("--- Demo 1: Cross-Fade Transitions ---");
    
    let mut world = World::new();
    let mut blender = AnimationBlender::new();
    
    // Add clips to blender
    blender.add_clip("Idle", idle_clip);
    blender.add_clip("Walk", walk_clip);
    
    // Start with idle
    blender.play("Idle");
    println!("Starting with Idle animation");
    
    // Simulate some time
    blender.update(1.0);
    
    // Cross-fade to walk over 0.3 seconds
    println!("Cross-fading from Idle to Walk over 0.3 seconds");
    blender.cross_fade("Idle", "Walk", 0.3);
    
    // Simulate cross-fade
    for i in 0..4 {
        blender.update(0.1);
        let pose = blender.evaluate(skeleton);
        println!(
            "  Frame {}: Pose evaluated with {} bones",
            i,
            pose.local_transforms().len()
        );
    }
    
    println!("Cross-fade complete!\n");
}

fn demo_blend_tree_1d(
    skeleton: &Skeleton,
    idle_clip: AnimationClip,
    walk_clip: AnimationClip,
    run_clip: AnimationClip,
) {
    println!("--- Demo 2: 1D Blend Tree (Speed-Based) ---");
    
    let mut blender = AnimationBlender::new();
    
    // Add clips
    blender.add_clip("Idle", idle_clip);
    blender.add_clip("Walk", walk_clip);
    blender.add_clip("Run", run_clip);
    
    // Create 1D blend tree for speed
    let mut blend_tree = BlendNode1D::new();
    blend_tree.add_clip("Idle", 0.0);
    blend_tree.add_clip("Walk", 0.5);
    blend_tree.add_clip("Run", 1.0);
    
    blender.add_blend_tree("Movement", blend_tree.into());
    blender.activate_blend_tree("Movement");
    
    println!("Created 1D blend tree: Idle (0.0) -> Walk (0.5) -> Run (1.0)");
    
    // Test different speed values
    let speed_values = [0.0, 0.25, 0.5, 0.75, 1.0];
    
    for speed in speed_values {
        blender.set_blend_parameter("Movement", speed);
        blender.update(0.016);
        let pose = blender.evaluate(skeleton);
        
        println!(
            "  Speed {:.2}: Evaluated pose with {} bones",
            speed,
            pose.local_transforms().len()
        );
    }
    
    println!();
}

fn demo_blend_tree_2d(skeleton: &Skeleton) {
    println!("--- Demo 3: 2D Blend Tree (Directional Movement) ---");
    
    let mut blender = AnimationBlender::new();
    
    // Add directional movement clips
    blender.add_clip("Forward", create_walk_animation());
    blender.add_clip("Back", create_walk_animation());
    blender.add_clip("Left", create_walk_animation());
    blender.add_clip("Right", create_walk_animation());
    blender.add_clip("Idle", create_idle_animation());
    
    // Create 2D blend tree
    let mut blend_tree = BlendNode2D::new();
    blend_tree.add_clip("Idle", 0.0, 0.0);
    blend_tree.add_clip("Forward", 0.0, 1.0);
    blend_tree.add_clip("Back", 0.0, -1.0);
    blend_tree.add_clip("Left", -1.0, 0.0);
    blend_tree.add_clip("Right", 1.0, 0.0);
    
    blender.add_blend_tree("Locomotion", blend_tree.into());
    blender.activate_blend_tree("Locomotion");
    
    println!("Created 2D blend tree for 8-directional movement");
    
    // Test different directions
    let directions = [
        (0.0, 0.0, "Idle"),
        (0.0, 1.0, "Forward"),
        (0.7, 0.7, "Forward-Right"),
        (1.0, 0.0, "Right"),
        (0.0, -1.0, "Back"),
    ];
    
    for (x, y, name) in directions {
        blender.set_blend_parameters_2d("Locomotion", x, y);
        blender.update(0.016);
        let pose = blender.evaluate(skeleton);
        
        println!(
            "  Direction {} ({:.1}, {:.1}): {} bones animated",
            name,
            x,
            y,
            pose.local_transforms().len()
        );
    }
    
    println!();
}

fn demo_layered_animation(skeleton: &Skeleton, walk_clip: AnimationClip, wave_clip: AnimationClip) {
    println!("--- Demo 4: Layered Animation with Bone Masking ---");
    
    let mut blender = AnimationBlender::new();
    
    // Add clips
    blender.add_clip("Walk", walk_clip);
    blender.add_clip("Wave", wave_clip);
    
    // Play walk on base layer
    blender.play("Walk");
    println!("Base layer: Walking animation (full body)");
    
    // Create upper body mask for waving
    let mut upper_body_mask = BoneMask::with_bone_count(4);
    upper_body_mask.enable_bone(3); // Right arm
    
    // Create layer for upper body
    let mut upper_layer = AnimationLayer::new(1.0);
    upper_layer.set_mask(upper_body_mask);
    upper_layer.set_blend_mode(LayerBlendMode::Override);
    
    blender.add_layer(upper_layer);
    blender.play_on_layer(0, "Wave");
    
    println!("Layer 1: Waving animation (right arm only, weight: 1.0)");
    println!("Result: Character walks while waving");
    
    // Simulate animation
    for i in 0..5 {
        blender.update(0.1);
        let pose = blender.evaluate(skeleton);
        println!(
            "  Frame {}: Combined pose with {} bones",
            i,
            pose.local_transforms().len()
        );
    }
    
    println!();
}

fn demo_additive_blending(skeleton: &Skeleton, walk_clip: AnimationClip, run_clip: AnimationClip) {
    println!("--- Demo 5: Additive Blending ---");
    
    let mut blender = AnimationBlender::new();
    
    // Add clips
    blender.add_clip("Walk", walk_clip);
    blender.add_clip("Run", run_clip);
    
    // Create additive blend node
    let mut additive_node = AdditiveBlendNode::new();
    additive_node.set_base("Walk");
    additive_node.set_additive("Run");
    additive_node.set_weight(0.5);
    
    blender.add_blend_tree("AdditiveMovement", additive_node.into());
    blender.activate_blend_tree("AdditiveMovement");
    
    println!("Created additive blend: Walk (base) + Run (50% additive)");
    println!("Result: Walk with added intensity from run");
    
    // Simulate with different additive weights
    let weights = [0.0, 0.25, 0.5, 0.75, 1.0];
    
    for weight in weights {
        // Recreate node with new weight
        let mut additive_node = AdditiveBlendNode::new();
        additive_node.set_base("Walk");
        additive_node.set_additive("Run");
        additive_node.set_weight(weight);
        blender.add_blend_tree("AdditiveMovement", additive_node.into());
        
        blender.update(0.016);
        let pose = blender.evaluate(skeleton);
        
        println!(
            "  Additive weight {:.2}: {} bones affected",
            weight,
            pose.local_transforms().len()
        );
    }
    
    println!();
}
