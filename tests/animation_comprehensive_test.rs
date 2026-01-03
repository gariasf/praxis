//! Comprehensive integration tests for animation system functionality.

use praxis_scene::*;
use praxis_math::{Mat4, Quat, Vec3};
use std::f32::consts::PI;

#[test]
fn test_skeleton_hierarchy_setup() {
    let bones = vec![
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
            "Head".to_string(),
            Some(1),
            Vec3::new(0.0, 0.5, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
    ];

    let skeleton = Skeleton::new(bones);
    
    assert_eq!(skeleton.bone_count(), 3);
    assert!(skeleton.find_bone("Root").is_some());
    assert!(skeleton.find_bone("Spine").is_some());
    assert!(skeleton.find_bone("Head").is_some());
    
    // Verify hierarchy
    assert_eq!(skeleton.bone(0).unwrap().parent_index, None);
    assert_eq!(skeleton.bone(1).unwrap().parent_index, Some(0));
    assert_eq!(skeleton.bone(2).unwrap().parent_index, Some(1));
}

#[test]
fn test_animation_clip_creation_with_keyframes() {
    let mut clip = AnimationClip::new("Walk".to_string(), 2.0);
    
    // Add translation track
    clip.add_translation_keyframe(0, 0.0, Vec3::ZERO);
    clip.add_translation_keyframe(0, 1.0, Vec3::new(1.0, 0.0, 0.0));
    clip.add_translation_keyframe(0, 2.0, Vec3::new(2.0, 0.0, 0.0));
    
    // Add rotation track
    clip.add_rotation_keyframe(1, 0.0, Quat::IDENTITY);
    clip.add_rotation_keyframe(1, 1.0, Quat::from_rotation_y(PI / 2.0));
    
    // Add scale track
    clip.add_scale_keyframe(2, 0.0, Vec3::ONE);
    clip.add_scale_keyframe(2, 2.0, Vec3::new(2.0, 2.0, 2.0));
    
    assert_eq!(clip.duration(), 2.0);
    assert_eq!(clip.track_count(), 3);
    assert!(clip.bone_track(0).is_some());
    assert!(clip.bone_track(1).is_some());
    assert!(clip.bone_track(2).is_some());
}

#[test]
fn test_animation_player_playback_control() {
    let mut player = AnimationPlayer::new();
    
    let clip1 = AnimationClip::new("Idle".to_string(), 1.0);
    let clip2 = AnimationClip::new("Walk".to_string(), 2.0);
    
    player.add_clip("Idle".to_string(), clip1);
    player.add_clip("Walk".to_string(), clip2);
    
    // Test playback
    player.play("Idle");
    assert!(player.is_playing("Idle"));
    
    player.pause("Idle");
    assert!(!player.is_playing("Idle"));
    
    player.resume("Idle");
    assert!(player.is_playing("Idle"));
    
    player.stop("Idle");
    assert!(!player.is_playing("Idle"));
}

#[test]
fn test_animation_player_time_progression() {
    let mut player = AnimationPlayer::new();
    let clip = AnimationClip::new("Test".to_string(), 1.0);
    player.add_clip("Test".to_string(), clip);
    
    player.play("Test");
    assert_eq!(player.current_time("Test"), Some(0.0));
    
    player.update(0.25);
    let time = player.current_time("Test").unwrap();
    assert!((time - 0.25).abs() < 0.001);
    
    player.update(0.25);
    let time = player.current_time("Test").unwrap();
    assert!((time - 0.5).abs() < 0.001);
}

#[test]
fn test_animation_player_looping_behavior() {
    let mut player = AnimationPlayer::new();
    let clip = AnimationClip::new("Loop".to_string(), 1.0);
    player.add_clip("Loop".to_string(), clip);
    
    player.play("Loop");
    player.set_looping("Loop", true);
    
    // Update past duration
    player.update(1.5);
    
    // Should have looped back
    let time = player.current_time("Loop").unwrap();
    assert!((time - 0.5).abs() < 0.001);
    assert!(player.is_playing("Loop"));
}

#[test]
fn test_animation_player_non_looping_behavior() {
    let mut player = AnimationPlayer::new();
    let clip = AnimationClip::new("Once".to_string(), 1.0);
    player.add_clip("Once".to_string(), clip);
    
    player.play("Once");
    player.set_looping("Once", false);
    
    // Update past duration
    player.update(1.5);
    
    // Should have stopped
    assert!(!player.is_playing("Once"));
}

#[test]
fn test_animation_player_speed_control() {
    let mut player = AnimationPlayer::new();
    let clip = AnimationClip::new("Fast".to_string(), 2.0);
    player.add_clip("Fast".to_string(), clip);
    
    player.play("Fast");
    player.set_speed("Fast", 2.0);
    
    player.update(0.5);
    
    // At 2x speed, 0.5 seconds should advance by 1.0 second
    let time = player.current_time("Fast").unwrap();
    assert!((time - 1.0).abs() < 0.001);
}

#[test]
fn test_animation_blender_simple_playback() {
    let mut blender = AnimationBlender::new();
    
    let clip = AnimationClip::new("Idle".to_string(), 1.0);
    blender.add_clip("Idle", clip);
    
    blender.play("Idle");
    assert_eq!(blender.current_clip(), Some("Idle"));
    assert_eq!(blender.current_time(), 0.0);
}

#[test]
fn test_animation_blender_cross_fade() {
    let mut blender = AnimationBlender::new();
    
    let clip1 = AnimationClip::new("Idle".to_string(), 1.0);
    let clip2 = AnimationClip::new("Walk".to_string(), 2.0);
    
    blender.add_clip("Idle", clip1);
    blender.add_clip("Walk", clip2);
    
    blender.play("Idle");
    blender.cross_fade("Idle", "Walk", 0.5);
    
    assert!(blender.is_cross_fading());
    
    // Update to complete fade
    blender.update(0.6);
    
    assert!(!blender.is_cross_fading());
    assert_eq!(blender.current_clip(), Some("Walk"));
}

#[test]
fn test_blend_node_1d_setup() {
    let mut node = BlendNode1D::new();
    
    node.add_clip("Idle", 0.0);
    node.add_clip("Walk", 0.5);
    node.add_clip("Run", 1.0);
    
    node.set_parameter(0.5);
    assert_eq!(node.parameter(), 0.5);
    
    let weights = node.compute_weights();
    assert!(!weights.is_empty());
}

#[test]
fn test_blend_node_2d_setup() {
    let mut node = BlendNode2D::new();
    
    node.add_clip("Forward", 0.0, 1.0);
    node.add_clip("Back", 0.0, -1.0);
    node.add_clip("Left", -1.0, 0.0);
    node.add_clip("Right", 1.0, 0.0);
    
    node.set_parameters(0.5, 0.5);
    let (x, y) = node.parameters();
    assert_eq!(x, 0.5);
    assert_eq!(y, 0.5);
    
    let weights = node.compute_weights();
    assert!(!weights.is_empty());
}

#[test]
fn test_bone_mask_operations() {
    let mut mask = BoneMask::with_bone_count(10);
    
    // Initially all disabled
    assert!(!mask.is_bone_enabled(0));
    assert!(!mask.is_bone_enabled(5));
    
    // Enable specific bones
    mask.enable_bone(0);
    mask.enable_bone(5);
    
    assert!(mask.is_bone_enabled(0));
    assert!(mask.is_bone_enabled(5));
    assert!(!mask.is_bone_enabled(3));
    
    // Disable bone
    mask.disable_bone(0);
    assert!(!mask.is_bone_enabled(0));
}

#[test]
fn test_animation_layer_setup() {
    let layer = AnimationLayer::new(0.8);
    
    assert_eq!(layer.weight(), 0.8);
    assert!(layer.current_clip().is_none());
}

#[test]
fn test_animation_layer_playback() {
    let mut layer = AnimationLayer::new(1.0);
    
    layer.play("UpperBodyAim");
    assert_eq!(layer.current_clip(), Some("UpperBodyAim"));
    assert_eq!(layer.time(), 0.0);
    
    layer.stop();
    assert!(layer.current_clip().is_none());
}

#[test]
fn test_animated_pose_creation() {
    let pose = AnimatedPose::new(5);
    
    assert_eq!(pose.local_transforms().len(), 5);
    assert_eq!(pose.world_transforms().len(), 5);
    assert_eq!(pose.skinning_matrices().len(), 5);
}

#[test]
fn test_complete_animation_pipeline() {
    // Create skeleton
    let bones = vec![
        Bone::with_bind_pose(
            "Root".to_string(),
            None,
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        Bone::with_bind_pose(
            "Arm".to_string(),
            Some(0),
            Vec3::new(1.0, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
    ];
    let skeleton = Skeleton::new(bones);
    
    // Create animation
    let mut clip = AnimationClip::new("Wave".to_string(), 1.0);
    clip.add_rotation_keyframe(1, 0.0, Quat::IDENTITY);
    clip.add_rotation_keyframe(1, 0.5, Quat::from_rotation_z(PI / 2.0));
    clip.add_rotation_keyframe(1, 1.0, Quat::IDENTITY);
    
    // Create player
    let mut player = AnimationPlayer::new();
    player.add_clip("Wave".to_string(), clip);
    player.play("Wave");
    
    // Update
    player.update(0.25);
    
    // Evaluate
    let pose = player.evaluate(&skeleton);
    
    assert_eq!(pose.local_transforms().len(), skeleton.bone_count());
    assert_eq!(pose.world_transforms().len(), skeleton.bone_count());
}

#[test]
fn test_cross_fade_transition_progress() {
    let mut transition = CrossFadeTransition::new(
        "Idle".to_string(),
        "Walk".to_string(),
        1.0,
    );
    
    assert_eq!(transition.blend_weight(), 0.0);
    assert!(!transition.is_complete());
    
    transition.update(0.5);
    assert!((transition.blend_weight() - 0.5).abs() < 0.001);
    assert!(!transition.is_complete());
    
    transition.update(0.5);
    assert_eq!(transition.blend_weight(), 1.0);
    assert!(transition.is_complete());
}

#[test]
fn test_additive_blend_node() {
    let mut node = AdditiveBlendNode::new();
    
    node.set_base("Walk");
    node.set_additive("Recoil");
    node.set_weight(0.7);
    
    let (base, additive, weight) = node.get_clips();
    
    assert_eq!(base, Some("Walk".to_string()));
    assert_eq!(additive, Some("Recoil".to_string()));
    assert_eq!(weight, 0.7);
}

#[test]
fn test_blend_weight_normalization_1d() {
    let mut node = BlendNode1D::new();
    node.add_clip("A", 0.0);
    node.add_clip("B", 0.5);
    node.add_clip("C", 1.0);
    
    node.set_parameter(0.25);
    let weights = node.compute_weights();
    
    // Sum of weights should be 1.0
    let sum: f32 = weights.iter().map(|(_, w)| w).sum();
    assert!((sum - 1.0).abs() < 0.01);
}

#[test]
fn test_blend_weight_normalization_2d() {
    let mut node = BlendNode2D::new();
    node.add_clip("A", 0.0, 1.0);
    node.add_clip("B", 1.0, 0.0);
    node.add_clip("C", 0.0, -1.0);
    node.add_clip("D", -1.0, 0.0);
    
    node.set_parameters(0.3, 0.4);
    let weights = node.compute_weights();
    
    // Sum of weights should be approximately 1.0
    let sum: f32 = weights.iter().map(|(_, w)| w).sum();
    assert!((sum - 1.0).abs() < 0.01);
}

#[test]
fn test_skeleton_recompute_inverse_bind_matrices() {
    let bones = vec![
        Bone::with_bind_pose(
            "Root".to_string(),
            None,
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::ONE,
        ),
    ];
    
    let mut skeleton = Skeleton::new(bones);
    let original = skeleton.inverse_bind_matrix(0).unwrap();
    
    // Modify bone and recompute
    skeleton.bone_mut(0).unwrap().bind_pose_translation = Vec3::new(5.0, 0.0, 0.0);
    skeleton.recompute_inverse_bind_matrices();
    
    let updated = skeleton.inverse_bind_matrix(0).unwrap();
    
    // Matrices should be different after recomputation
    assert!((original - updated).abs_diff_eq(Mat4::IDENTITY, 0.001));
}
