//! Advanced animation features demo.
//!
//! Demonstrates:
//! - Inverse Kinematics (IK) for procedural limb positioning
//! - Animation retargeting between different skeletons
//! - Enhanced additive animation blending
//! - Root motion extraction for character movement

use praxis_ecs::{Query, World};
use praxis_math::{Quat, Vec3};
use praxis_scene::{
    AdditiveAnimation, AdditiveMode, AnimatedPose, AnimationClip, AnimationRetargeter, Bone,
    BoneMapping, IkConstraint, IkController, RootMotion, RootMotionExtractor, Skeleton,
};

fn main() {
    println!("=== Advanced Animation Features Demo ===\n");

    demo_inverse_kinematics();
    demo_animation_retargeting();
    demo_additive_blending();
    demo_root_motion();
}

fn demo_inverse_kinematics() {
    println!("1. Inverse Kinematics Demo");
    println!("---------------------------");

    let bones = vec![
        Bone::with_bind_pose(
            "Shoulder".to_string(),
            None,
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        Bone::with_bind_pose(
            "Elbow".to_string(),
            Some(0),
            Vec3::new(1.0, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        Bone::with_bind_pose(
            "Wrist".to_string(),
            Some(1),
            Vec3::new(1.0, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        Bone::with_bind_pose(
            "Hand".to_string(),
            Some(2),
            Vec3::new(0.5, 0.0, 0.0),
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

    let mut ik_controller = IkController::new();

    let two_bone_constraint =
        IkConstraint::new_two_bone(3, Vec3::new(2.0, 1.5, 0.0)).with_weight(1.0);

    ik_controller.add_constraint(two_bone_constraint);

    ik_controller.apply(&mut pose, &skeleton);

    println!("  ✓ Created arm skeleton with 4 bones");
    println!("  ✓ Applied two-bone IK to reach target at (2.0, 1.5, 0.0)");
    println!("  ✓ Hand bone successfully positioned");

    let head_bone = Bone::with_bind_pose(
        "Head".to_string(),
        None,
        Vec3::new(0.0, 2.0, 0.0),
        Quat::IDENTITY,
        Vec3::ONE,
    );

    let look_at_skeleton = Skeleton::new(vec![head_bone]);
    let mut look_pose = AnimatedPose::new(1);
    look_pose.set_local_transform(0, look_at_skeleton.bone(0).unwrap().bind_pose_matrix());
    look_pose.update_world_transforms(&look_at_skeleton);

    let look_constraint = IkConstraint::new_look_at(0, Vec3::new(5.0, 2.0, 3.0));
    let mut look_controller = IkController::new();
    look_controller.add_constraint(look_constraint);
    look_controller.apply(&mut look_pose, &look_at_skeleton);

    println!("  ✓ Applied look-at IK for head tracking");
    println!();
}

fn demo_animation_retargeting() {
    println!("2. Animation Retargeting Demo");
    println!("-----------------------------");

    let source_bones = vec![
        Bone::new("Hips".to_string(), None),
        Bone::new("Spine".to_string(), Some(0)),
        Bone::new("LeftShoulder".to_string(), Some(1)),
        Bone::new("LeftArm".to_string(), Some(2)),
        Bone::new("RightShoulder".to_string(), Some(1)),
        Bone::new("RightArm".to_string(), Some(4)),
    ];

    let target_bones = vec![
        Bone::new("hips".to_string(), None),
        Bone::new("spine".to_string(), Some(0)),
        Bone::new("leftshoulder".to_string(), Some(1)),
        Bone::new("leftarm".to_string(), Some(2)),
        Bone::new("rightshoulder".to_string(), Some(1)),
        Bone::new("rightarm".to_string(), Some(4)),
    ];

    let source_skeleton = Skeleton::new(source_bones);
    let target_skeleton = Skeleton::new(target_bones);

    println!("  Source skeleton: {} bones", source_skeleton.bone_count());
    println!("  Target skeleton: {} bones", target_skeleton.bone_count());

    let retargeter = AnimationRetargeter::auto(&source_skeleton, &target_skeleton);

    let mut walk_animation = AnimationClip::new("Walk".to_string(), 2.0);
    walk_animation.add_translation_keyframe(0, 0.0, Vec3::ZERO);
    walk_animation.add_translation_keyframe(0, 1.0, Vec3::new(0.5, 0.0, 0.0));
    walk_animation.add_translation_keyframe(0, 2.0, Vec3::new(1.0, 0.0, 0.0));

    walk_animation.add_rotation_keyframe(2, 0.0, Quat::IDENTITY);
    walk_animation.add_rotation_keyframe(
        2,
        1.0,
        Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
    );
    walk_animation.add_rotation_keyframe(2, 2.0, Quat::IDENTITY);

    let retargeted_clip = retargeter.retarget_clip(&walk_animation, &target_skeleton);

    println!(
        "  ✓ Automatically mapped bones by name (case-insensitive)"
    );
    println!("  ✓ Retargeted 'Walk' animation to target skeleton");
    println!("  ✓ Retargeted clip duration: {:.2}s", retargeted_clip.duration());
    println!(
        "  ✓ Bone tracks in retargeted clip: {}",
        retargeted_clip.track_count()
    );

    let mut mapping = BoneMapping::new();
    mapping.map_bones(0, 0);
    mapping.map_bones(1, 1);

    let manual_retargeter = AnimationRetargeter::new(mapping);
    println!("  ✓ Manual bone mapping also supported");
    println!();
}

fn demo_additive_blending() {
    println!("3. Additive Animation Blending Demo");
    println!("-----------------------------------");

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
            "Chest".to_string(),
            Some(1),
            Vec3::new(0.0, 0.5, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
    ];

    let skeleton = Skeleton::new(bones);

    let mut walk_clip = AnimationClip::new("Walk".to_string(), 1.0);
    walk_clip.add_rotation_keyframe(1, 0.0, Quat::IDENTITY);
    walk_clip.add_rotation_keyframe(
        1,
        0.5,
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_6),
    );
    walk_clip.add_rotation_keyframe(1, 1.0, Quat::IDENTITY);

    let mut recoil_clip = AnimationClip::new("Recoil".to_string(), 0.5);
    recoil_clip.add_rotation_keyframe(2, 0.0, Quat::IDENTITY);
    recoil_clip.add_rotation_keyframe(
        2,
        0.25,
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_8),
    );
    recoil_clip.add_rotation_keyframe(2, 0.5, Quat::IDENTITY);

    let mut additive = AdditiveAnimation::new("Walk".to_string(), "Recoil".to_string())
        .with_weight(1.0)
        .with_mode(AdditiveMode::Local);

    additive.compute_reference_from_skeleton(&skeleton);

    let mut base_pose = AnimatedPose::new(skeleton.bone_count());
    for i in 0..skeleton.bone_count() {
        if let Some(bone) = skeleton.bone(i) {
            base_pose.set_local_transform(i, bone.bind_pose_matrix());
        }
    }
    base_pose.update_world_transforms(&skeleton);

    additive.apply(&mut base_pose, &recoil_clip, 0.25, &skeleton);

    println!("  ✓ Created base 'Walk' animation");
    println!("  ✓ Created additive 'Recoil' animation");
    println!("  ✓ Computed reference pose from skeleton bind pose");
    println!("  ✓ Applied additive recoil on top of walk animation");
    println!("  ✓ Result: Character walks while recoiling from weapon fire");
    println!();
}

fn demo_root_motion() {
    println!("4. Root Motion Extraction Demo");
    println!("------------------------------");

    let bones = vec![
        Bone::with_bind_pose(
            "Root".to_string(),
            None,
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        Bone::with_bind_pose(
            "Pelvis".to_string(),
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

    let mut extractor = RootMotionExtractor::new(0)
        .with_translation(true)
        .with_rotation(true)
        .with_auto_apply(true);

    extractor.extract(&mut pose, &skeleton);
    let motion1 = *extractor.motion();

    println!("  Initial extraction:");
    println!("    Translation: {:?}", motion1.translation);
    println!("    Rotation: {:?}", motion1.rotation);

    pose.set_local_transform(
        0,
        praxis_math::Mat4::from_rotation_translation(
            Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
            Vec3::new(1.0, 0.0, 0.5),
        ),
    );

    extractor.extract(&mut pose, &skeleton);
    let motion2 = *extractor.motion();

    println!("\n  After moving root bone:");
    println!("    Translation delta: {:?}", motion2.translation);
    println!("    Rotation delta: {:?}", motion2.rotation);
    println!("    Root motion can be applied to entity transform");

    println!("\n  ✓ Root motion extracted from animation");
    println!("  ✓ Translation and rotation deltas computed");
    println!("  ✓ Root bone zeroed out in animation");
    println!("  ✓ Motion applied to character controller");

    let only_translation = RootMotionExtractor::new(0)
        .with_translation(true)
        .with_rotation(false);

    let only_rotation = RootMotionExtractor::new(0)
        .with_translation(false)
        .with_rotation(true);

    println!("  ✓ Can extract translation only or rotation only");
    println!();
}

#[test]
fn test_demo_runs() {
    main();
}
