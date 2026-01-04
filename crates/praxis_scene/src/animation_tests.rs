use super::*;
use praxis_math::{Quat, Vec3};

#[test]
fn test_ik_constraint_creation() {
    let constraint = IkConstraint::new_two_bone(2, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(constraint.end_effector_bone, 2);
    assert_eq!(constraint.target(), Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(constraint.weight(), 1.0);
}

#[test]
fn test_ik_constraint_with_pole() {
    let constraint = IkConstraint::new_two_bone(2, Vec3::ZERO)
        .with_pole_target(Vec3::Y)
        .with_weight(0.5);
    
    assert!(constraint.pole_target.is_some());
    assert_eq!(constraint.weight(), 0.5);
}

#[test]
fn test_ik_controller() {
    let mut controller = IkController::new();
    assert_eq!(controller.constraints().len(), 0);
    
    let constraint = IkConstraint::new_two_bone(1, Vec3::ZERO);
    controller.add_constraint(constraint);
    
    assert_eq!(controller.constraints().len(), 1);
    
    controller.clear_constraints();
    assert_eq!(controller.constraints().len(), 0);
}

#[test]
fn test_ik_two_bone_solver() {
    let bones = vec![
        Bone::with_bind_pose("Root".to_string(), None, Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
        Bone::with_bind_pose(
            "Upper".to_string(),
            Some(0),
            Vec3::new(0.0, 1.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        Bone::with_bind_pose(
            "Lower".to_string(),
            Some(1),
            Vec3::new(0.0, 1.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        Bone::with_bind_pose(
            "Hand".to_string(),
            Some(2),
            Vec3::new(0.0, 1.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
    ];
    
    let skeleton = Skeleton::new(bones);
    let mut pose = AnimatedPose::new(skeleton.bone_count());
    
    for i in 0..skeleton.bone_count() {
        if let Some(bone) = skeleton.bone(i) {
            pose.set_local_transform(i, bone.bind_pose_matrix());
        }
    }
    pose.update_world_transforms(&skeleton);
    
    let constraint = IkConstraint::new_two_bone(3, Vec3::new(1.5, 2.0, 0.0));
    IkSolver::solve_two_bone(&constraint, &mut pose, &skeleton);
    
    assert!(pose.world_transform(3).is_some());
}

#[test]
fn test_bone_mapping_creation() {
    let mut mapping = BoneMapping::new();
    mapping.map_bones(0, 1);
    mapping.map_bones(1, 2);
    
    assert_eq!(mapping.get_target_bone(0), Some(1));
    assert_eq!(mapping.get_target_bone(1), Some(2));
    assert_eq!(mapping.get_target_bone(2), None);
}

#[test]
fn test_bone_mapping_auto() {
    let source_bones = vec![
        Bone::new("Root".to_string(), None),
        Bone::new("Spine".to_string(), Some(0)),
        Bone::new("LeftArm".to_string(), Some(1)),
    ];
    
    let target_bones = vec![
        Bone::new("root".to_string(), None),
        Bone::new("spine".to_string(), Some(0)),
        Bone::new("leftarm".to_string(), Some(1)),
    ];
    
    let source_skeleton = Skeleton::new(source_bones);
    let target_skeleton = Skeleton::new(target_bones);
    
    let mapping = BoneMapping::auto_map(&source_skeleton, &target_skeleton);
    
    assert_eq!(mapping.get_target_bone(0), Some(0));
    assert_eq!(mapping.get_target_bone(1), Some(1));
    assert_eq!(mapping.get_target_bone(2), Some(2));
}

#[test]
fn test_animation_retargeter() {
    let source_bones = vec![
        Bone::new("Root".to_string(), None),
        Bone::new("Bone1".to_string(), Some(0)),
    ];
    
    let target_bones = vec![
        Bone::new("Root".to_string(), None),
        Bone::new("Bone1".to_string(), Some(0)),
    ];
    
    let source_skeleton = Skeleton::new(source_bones);
    let target_skeleton = Skeleton::new(target_bones);
    
    let retargeter = AnimationRetargeter::auto(&source_skeleton, &target_skeleton);
    
    let mut source_clip = AnimationClip::new("Test".to_string(), 1.0);
    source_clip.add_translation_keyframe(0, 0.0, Vec3::ZERO);
    source_clip.add_translation_keyframe(0, 1.0, Vec3::new(1.0, 0.0, 0.0));
    
    let target_clip = retargeter.retarget_clip(&source_clip, &target_skeleton);
    
    assert_eq!(target_clip.duration(), 1.0);
    assert!(target_clip.bone_track(0).is_some());
}

#[test]
fn test_retarget_pose() {
    let source_bones = vec![
        Bone::new("Root".to_string(), None),
        Bone::new("Child".to_string(), Some(0)),
    ];
    
    let target_bones = vec![
        Bone::new("Root".to_string(), None),
        Bone::new("Child".to_string(), Some(0)),
    ];
    
    let source_skeleton = Skeleton::new(source_bones);
    let target_skeleton = Skeleton::new(target_bones);
    
    let mut source_pose = AnimatedPose::new(source_skeleton.bone_count());
    source_pose.set_local_transform(0, Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0)));
    source_pose.update_world_transforms(&source_skeleton);
    
    let retargeter = AnimationRetargeter::auto(&source_skeleton, &target_skeleton);
    let target_pose = retargeter.retarget_pose(&source_pose, &target_skeleton);
    
    assert!(target_pose.local_transform(0).is_some());
}

#[test]
fn test_additive_animation_creation() {
    let additive = AdditiveAnimation::new("Base".to_string(), "Additive".to_string());
    
    assert_eq!(additive.base_clip_name, "Base");
    assert_eq!(additive.additive_clip_name, "Additive");
    assert_eq!(additive.weight, 1.0);
}

#[test]
fn test_additive_animation_with_reference() {
    let bones = vec![Bone::new("Root".to_string(), None)];
    let skeleton = Skeleton::new(bones);
    
    let mut additive = AdditiveAnimation::new("Base".to_string(), "Additive".to_string());
    additive.compute_reference_from_skeleton(&skeleton);
    
    assert!(additive.reference_pose.is_some());
}

#[test]
fn test_root_motion_creation() {
    let motion = RootMotion::new(Vec3::new(1.0, 0.0, 0.0), Quat::IDENTITY);
    
    assert_eq!(motion.translation, Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(motion.rotation, Quat::IDENTITY);
    assert!(!motion.consumed);
}

#[test]
fn test_root_motion_consume() {
    let mut motion = RootMotion::identity();
    assert!(!motion.consumed);
    
    motion.consume();
    assert!(motion.consumed);
    
    motion.reset();
    assert!(!motion.consumed);
}

#[test]
fn test_root_motion_extractor_creation() {
    let extractor = RootMotionExtractor::new(0);
    
    assert_eq!(extractor.root_bone_index, 0);
    assert!(extractor.extract_translation);
    assert!(extractor.extract_rotation);
    assert!(extractor.apply_to_transform);
}

#[test]
fn test_root_motion_extractor_configuration() {
    let extractor = RootMotionExtractor::new(0)
        .with_translation(false)
        .with_rotation(true)
        .with_auto_apply(false);
    
    assert!(!extractor.extract_translation);
    assert!(extractor.extract_rotation);
    assert!(!extractor.apply_to_transform);
}

#[test]
fn test_root_motion_extraction() {
    let bones = vec![
        Bone::with_bind_pose("Root".to_string(), None, Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
        Bone::with_bind_pose(
            "Child".to_string(),
            Some(0),
            Vec3::new(0.0, 1.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
    ];
    
    let skeleton = Skeleton::new(bones);
    let mut pose = AnimatedPose::new(skeleton.bone_count());
    
    for i in 0..skeleton.bone_count() {
        if let Some(bone) = skeleton.bone(i) {
            pose.set_local_transform(i, bone.bind_pose_matrix());
        }
    }
    
    let mut extractor = RootMotionExtractor::new(0);
    
    extractor.extract(&mut pose, &skeleton);
    let motion = extractor.motion();
    assert_eq!(motion.translation, Vec3::ZERO);
    
    pose.set_local_transform(
        0,
        Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)),
    );
    
    extractor.extract(&mut pose, &skeleton);
    let motion = extractor.motion();
    assert_eq!(motion.translation, Vec3::new(1.0, 0.0, 0.0));
}

#[test]
fn test_ik_look_at_constraint() {
    let constraint = IkConstraint::new_look_at(1, Vec3::new(0.0, 0.0, 5.0));
    
    assert_eq!(constraint.constraint_type, IkConstraintType::LookAt);
    assert_eq!(constraint.end_effector_bone, 1);
    assert_eq!(constraint.target(), Vec3::new(0.0, 0.0, 5.0));
}

#[test]
fn test_ik_chain_constraint() {
    let constraint = IkConstraint::new_chain(3, Vec3::new(1.0, 2.0, 3.0), 20);
    
    assert_eq!(constraint.constraint_type, IkConstraintType::Chain);
    assert_eq!(constraint.max_iterations, 20);
}

#[test]
fn test_additive_mode() {
    let additive = AdditiveAnimation::new("Base".to_string(), "Add".to_string())
        .with_mode(AdditiveMode::World);
    
    assert_eq!(additive.mode, AdditiveMode::World);
}

#[test]
fn test_bone_mapping_by_name() {
    let mut mapping = BoneMapping::new();
    mapping.map_bone_names("SourceBone".to_string(), "TargetBone".to_string());
    
    assert!(mapping.name_mapping.contains_key("SourceBone"));
}
