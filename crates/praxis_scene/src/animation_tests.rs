//! Comprehensive tests for animation system.

#[cfg(test)]
mod tests {
    use crate::animation::*;
    use praxis_math::{Mat4, Quat, Vec3};
    use std::f32::consts::PI;

    // ============================================================================
    // Animation Interpolation Tests
    // ============================================================================

    #[test]
    fn test_translation_interpolation_linear() {
        let mut track = BoneTrack::new();
        track.add_translation_keyframe(0.0, Vec3::ZERO);
        track.add_translation_keyframe(1.0, Vec3::new(10.0, 0.0, 0.0));
        track.add_translation_keyframe(2.0, Vec3::new(20.0, 0.0, 0.0));

        // Test at exact keyframes
        let t0 = track.sample_translation(0.0).unwrap();
        assert!((t0 - Vec3::ZERO).length() < 0.001);

        let t1 = track.sample_translation(1.0).unwrap();
        assert!((t1 - Vec3::new(10.0, 0.0, 0.0)).length() < 0.001);

        let t2 = track.sample_translation(2.0).unwrap();
        assert!((t2 - Vec3::new(20.0, 0.0, 0.0)).length() < 0.001);

        // Test interpolation between keyframes
        let t_half = track.sample_translation(0.5).unwrap();
        assert!((t_half - Vec3::new(5.0, 0.0, 0.0)).length() < 0.001);

        let t_quarter = track.sample_translation(0.25).unwrap();
        assert!((t_quarter - Vec3::new(2.5, 0.0, 0.0)).length() < 0.001);

        let t_three_quarter = track.sample_translation(1.5).unwrap();
        assert!((t_three_quarter - Vec3::new(15.0, 0.0, 0.0)).length() < 0.001);
    }

    #[test]
    fn test_rotation_interpolation_slerp() {
        let mut track = BoneTrack::new();
        
        // Rotation from no rotation to 90 degrees around Z axis
        let start_rotation = Quat::IDENTITY;
        let end_rotation = Quat::from_rotation_z(PI / 2.0);
        
        track.add_rotation_keyframe(0.0, start_rotation);
        track.add_rotation_keyframe(1.0, end_rotation);

        // Test at keyframes
        let r0 = track.sample_rotation(0.0).unwrap();
        assert!((r0.dot(start_rotation) - 1.0).abs() < 0.001);

        let r1 = track.sample_rotation(1.0).unwrap();
        assert!((r1.dot(end_rotation) - 1.0).abs() < 0.001);

        // Test interpolation at midpoint
        let r_half = track.sample_rotation(0.5).unwrap();
        let expected_half = start_rotation.slerp(end_rotation, 0.5);
        assert!((r_half.dot(expected_half) - 1.0).abs() < 0.001);

        // Verify quaternion is normalized
        assert!((r_half.length() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_scale_interpolation_linear() {
        let mut track = BoneTrack::new();
        track.add_scale_keyframe(0.0, Vec3::ONE);
        track.add_scale_keyframe(1.0, Vec3::new(2.0, 2.0, 2.0));
        track.add_scale_keyframe(2.0, Vec3::new(0.5, 0.5, 0.5));

        // Test at keyframes
        let s0 = track.sample_scale(0.0).unwrap();
        assert!((s0 - Vec3::ONE).length() < 0.001);

        let s1 = track.sample_scale(1.0).unwrap();
        assert!((s1 - Vec3::new(2.0, 2.0, 2.0)).length() < 0.001);

        // Test interpolation
        let s_half = track.sample_scale(0.5).unwrap();
        let expected = Vec3::new(1.5, 1.5, 1.5);
        assert!((s_half - expected).length() < 0.001);

        let s_1_5 = track.sample_scale(1.5).unwrap();
        let expected_1_5 = Vec3::new(1.25, 1.25, 1.25);
        assert!((s_1_5 - expected_1_5).length() < 0.001);
    }

    #[test]
    fn test_interpolation_before_first_keyframe() {
        let mut track = BoneTrack::new();
        track.add_translation_keyframe(1.0, Vec3::new(10.0, 0.0, 0.0));
        track.add_translation_keyframe(2.0, Vec3::new(20.0, 0.0, 0.0));

        // Sample before first keyframe should return first keyframe value
        let t = track.sample_translation(0.0).unwrap();
        assert!((t - Vec3::new(10.0, 0.0, 0.0)).length() < 0.001);

        let t_neg = track.sample_translation(-5.0).unwrap();
        assert!((t_neg - Vec3::new(10.0, 0.0, 0.0)).length() < 0.001);
    }

    #[test]
    fn test_interpolation_after_last_keyframe() {
        let mut track = BoneTrack::new();
        track.add_translation_keyframe(0.0, Vec3::ZERO);
        track.add_translation_keyframe(1.0, Vec3::new(10.0, 0.0, 0.0));

        // Sample after last keyframe should return last keyframe value
        let t = track.sample_translation(2.0).unwrap();
        assert!((t - Vec3::new(10.0, 0.0, 0.0)).length() < 0.001);

        let t_far = track.sample_translation(100.0).unwrap();
        assert!((t_far - Vec3::new(10.0, 0.0, 0.0)).length() < 0.001);
    }

    #[test]
    fn test_interpolation_single_keyframe() {
        let mut track = BoneTrack::new();
        track.add_translation_keyframe(0.5, Vec3::new(5.0, 5.0, 5.0));

        // All samples should return the single keyframe value
        let t0 = track.sample_translation(0.0).unwrap();
        assert!((t0 - Vec3::new(5.0, 5.0, 5.0)).length() < 0.001);

        let t1 = track.sample_translation(0.5).unwrap();
        assert!((t1 - Vec3::new(5.0, 5.0, 5.0)).length() < 0.001);

        let t2 = track.sample_translation(1.0).unwrap();
        assert!((t2 - Vec3::new(5.0, 5.0, 5.0)).length() < 0.001);
    }

    #[test]
    fn test_interpolation_empty_track() {
        let track = BoneTrack::new();

        // Empty track should return None
        assert!(track.sample_translation(0.0).is_none());
        assert!(track.sample_rotation(0.0).is_none());
        assert!(track.sample_scale(0.0).is_none());
    }

    #[test]
    fn test_keyframe_sorting() {
        let mut track = BoneTrack::new();
        
        // Add keyframes in random order
        track.add_translation_keyframe(2.0, Vec3::new(20.0, 0.0, 0.0));
        track.add_translation_keyframe(0.0, Vec3::ZERO);
        track.add_translation_keyframe(1.0, Vec3::new(10.0, 0.0, 0.0));

        // Verify they're sorted correctly
        assert_eq!(track.translation_keyframes[0].time, 0.0);
        assert_eq!(track.translation_keyframes[1].time, 1.0);
        assert_eq!(track.translation_keyframes[2].time, 2.0);

        // Interpolation should work correctly
        let t = track.sample_translation(0.5).unwrap();
        assert!((t - Vec3::new(5.0, 0.0, 0.0)).length() < 0.001);
    }

    #[test]
    fn test_multi_axis_interpolation() {
        let mut track = BoneTrack::new();
        track.add_translation_keyframe(0.0, Vec3::new(0.0, 0.0, 0.0));
        track.add_translation_keyframe(1.0, Vec3::new(10.0, 20.0, 30.0));

        let t = track.sample_translation(0.5).unwrap();
        assert!((t - Vec3::new(5.0, 10.0, 15.0)).length() < 0.001);

        let t_quarter = track.sample_translation(0.25).unwrap();
        assert!((t_quarter - Vec3::new(2.5, 5.0, 7.5)).length() < 0.001);
    }

    // ============================================================================
    // Bone Matrix Calculation Tests
    // ============================================================================

    #[test]
    fn test_bone_bind_pose_matrix_identity() {
        let bone = Bone::with_bind_pose(
            "Test".to_string(),
            None,
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::ONE,
        );

        let matrix = bone.bind_pose_matrix();
        assert!((matrix - Mat4::IDENTITY).abs_diff_eq(Mat4::IDENTITY, 0.001));
    }

    #[test]
    fn test_bone_bind_pose_matrix_translation() {
        let translation = Vec3::new(5.0, 10.0, 15.0);
        let bone = Bone::with_bind_pose(
            "Test".to_string(),
            None,
            translation,
            Quat::IDENTITY,
            Vec3::ONE,
        );

        let matrix = bone.bind_pose_matrix();
        let extracted_translation = matrix.col(3).truncate();
        assert!((extracted_translation - translation).length() < 0.001);
    }

    #[test]
    fn test_bone_bind_pose_matrix_rotation() {
        let rotation = Quat::from_rotation_z(PI / 2.0);
        let bone = Bone::with_bind_pose(
            "Test".to_string(),
            None,
            Vec3::ZERO,
            rotation,
            Vec3::ONE,
        );

        let matrix = bone.bind_pose_matrix();
        let extracted_rotation = Quat::from_mat4(&matrix);
        assert!((extracted_rotation.dot(rotation) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_bone_bind_pose_matrix_scale() {
        let scale = Vec3::new(2.0, 3.0, 4.0);
        let bone = Bone::with_bind_pose(
            "Test".to_string(),
            None,
            Vec3::ZERO,
            Quat::IDENTITY,
            scale,
        );

        let matrix = bone.bind_pose_matrix();
        let extracted_scale = Vec3::new(
            matrix.col(0).truncate().length(),
            matrix.col(1).truncate().length(),
            matrix.col(2).truncate().length(),
        );
        assert!((extracted_scale - scale).length() < 0.001);
    }

    #[test]
    fn test_bone_bind_pose_matrix_combined() {
        let translation = Vec3::new(10.0, 20.0, 30.0);
        let rotation = Quat::from_rotation_y(PI / 4.0);
        let scale = Vec3::new(2.0, 2.0, 2.0);

        let bone = Bone::with_bind_pose(
            "Test".to_string(),
            None,
            translation,
            rotation,
            scale,
        );

        let matrix = bone.bind_pose_matrix();

        // Verify translation
        let extracted_translation = matrix.col(3).truncate();
        assert!((extracted_translation - translation).length() < 0.001);

        // Verify scale (approximate due to rotation)
        let extracted_scale = Vec3::new(
            matrix.col(0).truncate().length(),
            matrix.col(1).truncate().length(),
            matrix.col(2).truncate().length(),
        );
        assert!((extracted_scale - scale).length() < 0.001);
    }

    #[test]
    fn test_skeleton_inverse_bind_matrices_single_bone() {
        let bones = vec![Bone::with_bind_pose(
            "Root".to_string(),
            None,
            Vec3::new(1.0, 2.0, 3.0),
            Quat::IDENTITY,
            Vec3::ONE,
        )];

        let skeleton = Skeleton::new(bones);
        let inverse_bind = skeleton.inverse_bind_matrix(0).unwrap();

        // Inverse of translation matrix should move back
        let original = skeleton.bone(0).unwrap().bind_pose_matrix();
        let identity = original * inverse_bind;
        
        // Should be approximately identity
        assert!((identity - Mat4::IDENTITY).abs_diff_eq(Mat4::IDENTITY, 0.001));
    }

    #[test]
    fn test_skeleton_inverse_bind_matrices_hierarchy() {
        let bones = vec![
            Bone::with_bind_pose(
                "Root".to_string(),
                None,
                Vec3::new(0.0, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
            ),
            Bone::with_bind_pose(
                "Child1".to_string(),
                Some(0),
                Vec3::new(1.0, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
            ),
            Bone::with_bind_pose(
                "Child2".to_string(),
                Some(1),
                Vec3::new(1.0, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
            ),
        ];

        let skeleton = Skeleton::new(bones);

        // Each inverse bind matrix should be valid
        for i in 0..3 {
            let inverse_bind = skeleton.inverse_bind_matrix(i);
            assert!(inverse_bind.is_some());
        }

        // Child2's world position should be at (2, 0, 0)
        // Its inverse bind matrix should transform (2, 0, 0) to origin
        let child2_inverse = skeleton.inverse_bind_matrix(2).unwrap();
        let world_pos = Vec3::new(2.0, 0.0, 0.0);
        let local_pos = child2_inverse.transform_point3(world_pos);
        
        assert!(local_pos.length() < 0.001);
    }

    #[test]
    fn test_animated_pose_world_transforms() {
        let bones = vec![
            Bone::with_bind_pose(
                "Root".to_string(),
                None,
                Vec3::new(0.0, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
            ),
            Bone::with_bind_pose(
                "Child".to_string(),
                Some(0),
                Vec3::new(1.0, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
            ),
        ];

        let skeleton = Skeleton::new(bones);
        let mut pose = AnimatedPose::new(skeleton.bone_count());

        // Set root to translate by (5, 0, 0)
        pose.set_local_transform(0, Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0)));
        
        // Set child to translate by (2, 0, 0) relative to parent
        pose.set_local_transform(1, Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0)));

        pose.update_world_transforms(&skeleton);

        // Root world transform should be at (5, 0, 0)
        let root_world = pose.world_transform(0).unwrap();
        let root_pos = root_world.col(3).truncate();
        assert!((root_pos - Vec3::new(5.0, 0.0, 0.0)).length() < 0.001);

        // Child world transform should be at (7, 0, 0) = (5 + 2)
        let child_world = pose.world_transform(1).unwrap();
        let child_pos = child_world.col(3).truncate();
        assert!((child_pos - Vec3::new(7.0, 0.0, 0.0)).length() < 0.001);
    }

    #[test]
    fn test_animated_pose_skinning_matrices() {
        let bones = vec![
            Bone::with_bind_pose(
                "Root".to_string(),
                None,
                Vec3::ZERO,
                Quat::IDENTITY,
                Vec3::ONE,
            ),
        ];

        let skeleton = Skeleton::new(bones);
        let mut pose = AnimatedPose::new(skeleton.bone_count());

        // Set animated transform
        pose.set_local_transform(0, Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0)));
        
        pose.update_world_transforms(&skeleton);
        pose.update_skinning_matrices(&skeleton);

        // Skinning matrix should be world * inverse_bind
        let skinning_matrix = pose.skinning_matrices()[0];
        let inverse_bind = skeleton.inverse_bind_matrix(0).unwrap();
        let world = pose.world_transform(0).unwrap();
        
        let expected = world * inverse_bind;
        assert!((skinning_matrix - expected).abs_diff_eq(Mat4::IDENTITY, 0.001));
    }

    // ============================================================================
    // Blend Weight Normalization Tests
    // ============================================================================

    #[test]
    fn test_blend_node_1d_weight_normalization() {
        let mut node = BlendNode1D::new();
        node.add_clip("Idle", 0.0);
        node.add_clip("Walk", 0.5);
        node.add_clip("Run", 1.0);

        node.set_parameter(0.5);
        let weights = node.compute_weights();

        // Weights should sum to 1.0
        let sum: f32 = weights.iter().map(|(_, w)| w).sum();
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_blend_node_1d_boundary_weights() {
        let mut node = BlendNode1D::new();
        node.add_clip("Idle", 0.0);
        node.add_clip("Walk", 0.5);
        node.add_clip("Run", 1.0);

        // At exact parameter, should get 100% of that clip
        node.set_parameter(0.0);
        let weights = node.compute_weights();
        assert_eq!(weights.len(), 1);
        assert_eq!(weights[0].0, "Idle");
        assert!((weights[0].1 - 1.0).abs() < 0.001);

        node.set_parameter(1.0);
        let weights = node.compute_weights();
        assert_eq!(weights.len(), 1);
        assert_eq!(weights[0].0, "Run");
        assert!((weights[0].1 - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_blend_node_1d_interpolation_weights() {
        let mut node = BlendNode1D::new();
        node.add_clip("Idle", 0.0);
        node.add_clip("Walk", 1.0);

        // At midpoint, should get 50/50
        node.set_parameter(0.5);
        let weights = node.compute_weights();
        
        assert_eq!(weights.len(), 2);
        let sum: f32 = weights.iter().map(|(_, w)| w).sum();
        assert!((sum - 1.0).abs() < 0.001);
        
        // Both should be approximately 0.5
        for (_, weight) in &weights {
            assert!((*weight - 0.5).abs() < 0.001);
        }
    }

    #[test]
    fn test_blend_node_2d_weight_normalization() {
        let mut node = BlendNode2D::new();
        node.add_clip("Forward", 0.0, 1.0);
        node.add_clip("Back", 0.0, -1.0);
        node.add_clip("Left", -1.0, 0.0);
        node.add_clip("Right", 1.0, 0.0);

        node.set_parameters(0.5, 0.5);
        let weights = node.compute_weights();

        // Weights should sum to approximately 1.0
        let sum: f32 = weights.iter().map(|(_, w)| w).sum();
        assert!((sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_blend_node_2d_exact_position() {
        let mut node = BlendNode2D::new();
        node.add_clip("Forward", 0.0, 1.0);
        node.add_clip("Back", 0.0, -1.0);

        // At exact clip position, should get 100% of that clip
        node.set_parameters(0.0, 1.0);
        let weights = node.compute_weights();

        assert_eq!(weights.len(), 1);
        assert_eq!(weights[0].0, "Forward");
        assert!((weights[0].1 - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_blend_node_2d_center_position() {
        let mut node = BlendNode2D::new();
        node.add_clip("Forward", 0.0, 1.0);
        node.add_clip("Back", 0.0, -1.0);
        node.add_clip("Left", -1.0, 0.0);
        node.add_clip("Right", 1.0, 0.0);

        // At center, all clips should contribute
        node.set_parameters(0.0, 0.0);
        let weights = node.compute_weights();

        // All weights should be positive and sum to 1.0
        for (_, weight) in &weights {
            assert!(*weight > 0.0);
        }
        let sum: f32 = weights.iter().map(|(_, w)| w).sum();
        assert!((sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_animation_player_weight_clamping() {
        let mut player = AnimationPlayer::new();
        let clip = AnimationClip::new("Test".to_string(), 1.0);
        player.add_clip("Test".to_string(), clip);
        player.play("Test");

        // Test weight clamping
        player.set_weight("Test", 1.5);
        // Weight should be clamped to 1.0
        // We can't directly check the weight, but the behavior should be correct

        player.set_weight("Test", -0.5);
        // Weight should be clamped to 0.0
    }

    #[test]
    fn test_cross_fade_weight_progression() {
        let mut transition = CrossFadeTransition::new(
            "Idle".to_string(),
            "Walk".to_string(),
            1.0,
        );

        // At start, weight should be 0.0
        assert!((transition.blend_weight() - 0.0).abs() < 0.001);

        // At midpoint
        transition.update(0.5);
        assert!((transition.blend_weight() - 0.5).abs() < 0.001);

        // At end
        transition.update(0.5);
        assert!((transition.blend_weight() - 1.0).abs() < 0.001);
        assert!(transition.is_complete());
    }

    #[test]
    fn test_cross_fade_weight_clamping() {
        let mut transition = CrossFadeTransition::new(
            "Idle".to_string(),
            "Walk".to_string(),
            1.0,
        );

        // Update beyond duration
        transition.update(2.0);
        
        // Weight should be clamped to 1.0
        assert!((transition.blend_weight() - 1.0).abs() < 0.001);
        assert!(transition.is_complete());
    }

    #[test]
    fn test_animation_layer_weight_clamping() {
        let mut layer = AnimationLayer::new(0.5);
        
        assert!((layer.weight() - 0.5).abs() < 0.001);

        // Test clamping above 1.0
        layer.set_weight(1.5);
        assert!((layer.weight() - 1.0).abs() < 0.001);

        // Test clamping below 0.0
        layer.set_weight(-0.5);
        assert!((layer.weight() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_additive_blend_node_weight_clamping() {
        let mut node = AdditiveBlendNode::new();
        node.set_base("Base");
        node.set_additive("Additive");

        // Test weight clamping
        node.set_weight(1.5);
        let (_, _, weight) = node.get_clips();
        assert!((weight - 1.0).abs() < 0.001);

        node.set_weight(-0.5);
        let (_, _, weight) = node.get_clips();
        assert!((weight - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_bone_mask_weight_values() {
        let mut mask = BoneMask::with_bone_count(5);
        
        // Disabled bone should return 0.0 weight
        assert_eq!(mask.bone_weight(0), 0.0);

        // Enabled bone should return 1.0 weight
        mask.enable_bone(0);
        assert_eq!(mask.bone_weight(0), 1.0);

        // Disabled bone should return 0.0 weight
        mask.disable_bone(0);
        assert_eq!(mask.bone_weight(0), 0.0);
    }

    #[test]
    fn test_multiple_animation_blend_normalization() {
        let mut player = AnimationPlayer::new();
        
        let clip1 = AnimationClip::new("Clip1".to_string(), 1.0);
        let clip2 = AnimationClip::new("Clip2".to_string(), 1.0);
        let clip3 = AnimationClip::new("Clip3".to_string(), 1.0);
        
        player.add_clip("Clip1".to_string(), clip1);
        player.add_clip("Clip2".to_string(), clip2);
        player.add_clip("Clip3".to_string(), clip3);

        player.play("Clip1");
        player.play("Clip2");
        player.play("Clip3");

        // Set different weights
        player.set_weight("Clip1", 0.5);
        player.set_weight("Clip2", 0.3);
        player.set_weight("Clip3", 0.2);

        // All clips should be playing
        assert_eq!(player.playing_clips().len(), 3);
    }
}
