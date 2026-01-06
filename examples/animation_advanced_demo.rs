//! Advanced animation features demo with visual rendering.
//!
//! Demonstrates:
//! - Inverse Kinematics (IK) for procedural limb positioning
//! - Animation retargeting between different skeletons
//! - Enhanced additive animation blending
//! - Root motion extraction for character movement
//! - **Visual rendering of all features with 3D meshes**
//! - **Camera controls for viewing**
//! - **Interactive controls for switching between demos**
//!
//! Controls:
//! - WASD - Move camera
//! - Space/Ctrl - Move up/down
//! - Shift - Sprint
//! - Mouse - Look around
//! - 1 - Show IK demo
//! - 2 - Show Retargeting demo
//! - 3 - Show Additive Blending demo
//! - ESC - Toggle cursor / Exit

#[path = "common.rs"]
mod common;

use common::CameraController;
use praxis_ecs::{Component, PerspectiveCameraBundle, Transform, World};
use praxis_graphics::{
    sphere_mesh, DrawCommand, LightingUniforms, RenderCommands, RenderContext,
};
use praxis_input::{Action, InputMap, InputState};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_scene::{
    AdditiveAnimation, AdditiveMode, AnimatedPose, AnimationClip, AnimationRetargeter, Bone,
    BoneMapping, IkConstraint, IkController, Skeleton,
};
use praxis_utils::{info, Result};
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey};
use winit::window::{CursorGrabMode, Window, WindowId};

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

#[derive(Debug, Clone, Copy, PartialEq)]
enum DemoMode {
    IK,
    Retargeting,
    Additive,
}

struct DemoState {
    current_mode: DemoMode,
    ik_time: f32,
    retarget_time: f32,
    additive_time: f32,
}

impl DemoState {
    fn new() -> Self {
        Self {
            current_mode: DemoMode::IK,
            ik_time: 0.0,
            retarget_time: 0.0,
            additive_time: 0.0,
        }
    }
}

#[derive(Component)]
struct IKCharacter;

#[derive(Component)]
struct RetargetCharacter;

#[derive(Component)]
struct AdditiveCharacter;

// ============================================================================
// IK Demo
// ============================================================================

fn create_ik_skeleton() -> Skeleton {
    Skeleton::new(vec![
        // 0: Shoulder
        Bone::with_bind_pose(
            "Shoulder".to_string(),
            None,
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        // 1: Elbow
        Bone::with_bind_pose(
            "Elbow".to_string(),
            Some(0),
            Vec3::new(1.0, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        // 2: Wrist
        Bone::with_bind_pose(
            "Wrist".to_string(),
            Some(1),
            Vec3::new(1.0, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        // 3: Hand
        Bone::with_bind_pose(
            "Hand".to_string(),
            Some(2),
            Vec3::new(0.5, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
    ])
}

fn create_ik_controller(target: Vec3) -> IkController {
    let mut controller = IkController::new();
    let two_bone_constraint = IkConstraint::new_two_bone(3, target).with_weight(1.0);
    controller.add_constraint(two_bone_constraint);
    controller
}

// ============================================================================
// Retargeting Demo
// ============================================================================

fn create_source_skeleton() -> Skeleton {
    Skeleton::new(vec![
        Bone::with_bind_pose("Hips".to_string(), None, Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
        Bone::with_bind_pose(
            "Spine".to_string(),
            Some(0),
            Vec3::new(0.0, 0.8, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        Bone::with_bind_pose(
            "LeftShoulder".to_string(),
            Some(1),
            Vec3::new(-0.3, 0.5, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        Bone::with_bind_pose(
            "LeftArm".to_string(),
            Some(2),
            Vec3::new(-0.5, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
    ])
}

fn create_target_skeleton() -> Skeleton {
    Skeleton::new(vec![
        Bone::with_bind_pose("hips".to_string(), None, Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
        Bone::with_bind_pose(
            "spine".to_string(),
            Some(0),
            Vec3::new(0.0, 1.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        Bone::with_bind_pose(
            "leftshoulder".to_string(),
            Some(1),
            Vec3::new(-0.4, 0.6, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
        Bone::with_bind_pose(
            "leftarm".to_string(),
            Some(2),
            Vec3::new(-0.6, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ),
    ])
}

fn create_source_animation() -> AnimationClip {
    let mut clip = AnimationClip::new("Walk".to_string(), 2.0);
    clip.add_translation_keyframe(0, 0.0, Vec3::ZERO);
    clip.add_translation_keyframe(0, 1.0, Vec3::new(0.5, 0.0, 0.0));
    clip.add_translation_keyframe(0, 2.0, Vec3::new(1.0, 0.0, 0.0));

    clip.add_rotation_keyframe(2, 0.0, Quat::IDENTITY);
    clip.add_rotation_keyframe(2, 1.0, Quat::from_rotation_z(std::f32::consts::FRAC_PI_4));
    clip.add_rotation_keyframe(2, 2.0, Quat::IDENTITY);

    clip
}

// ============================================================================
// Additive Demo
// ============================================================================

fn create_additive_skeleton() -> Skeleton {
    Skeleton::new(vec![
        Bone::with_bind_pose("Root".to_string(), None, Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
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
    ])
}

fn create_walk_clip() -> AnimationClip {
    let mut clip = AnimationClip::new("Walk".to_string(), 1.0);
    clip.add_rotation_keyframe(1, 0.0, Quat::IDENTITY);
    clip.add_rotation_keyframe(1, 0.5, Quat::from_rotation_y(std::f32::consts::FRAC_PI_6));
    clip.add_rotation_keyframe(1, 1.0, Quat::IDENTITY);
    clip
}

fn create_recoil_clip() -> AnimationClip {
    let mut clip = AnimationClip::new("Recoil".to_string(), 0.5);
    clip.add_rotation_keyframe(2, 0.0, Quat::IDENTITY);
    clip.add_rotation_keyframe(2, 0.25, Quat::from_rotation_x(-std::f32::consts::FRAC_PI_8));
    clip.add_rotation_keyframe(2, 0.5, Quat::IDENTITY);
    clip
}

// ============================================================================
// Rendering Helpers
// ============================================================================

fn compute_bone_world_transforms(skeleton: &Skeleton, pose: &AnimatedPose) -> Vec<Mat4> {
    let mut world_transforms = Vec::new();

    for bone_index in 0..skeleton.bone_count() {
        let local_transform = pose
            .local_transform(bone_index)
            .unwrap_or(Mat4::IDENTITY);

        let world_transform = if let Some(bone) = skeleton.bone(bone_index) {
            if let Some(parent_index) = bone.parent_index {
                world_transforms[parent_index] * local_transform
            } else {
                local_transform
            }
        } else {
            local_transform
        };

        world_transforms.push(world_transform);
    }

    world_transforms
}

// ============================================================================
// App Structure
// ============================================================================

struct App {
    window: Option<Arc<Window>>,
    world: Option<World>,
    render_context: Option<RenderContext>,
    cursor_locked: bool,
    last_frame_time: Option<Instant>,
    camera_controller: CameraController,
    input_state: InputState,
    input_map: InputMap,
    demo_state: DemoState,
    ik_entity: Option<praxis_ecs::Entity>,
    retarget_entity: Option<praxis_ecs::Entity>,
    additive_entity: Option<praxis_ecs::Entity>,
}

impl Default for App {
    fn default() -> Self {
        let mut input_map = InputMap::default();
        input_map.bind_key(&Action::new("forward"), KeyCode::KeyW);
        input_map.bind_key(&Action::new("backward"), KeyCode::KeyS);
        input_map.bind_key(&Action::new("left"), KeyCode::KeyA);
        input_map.bind_key(&Action::new("right"), KeyCode::KeyD);
        input_map.bind_key(&Action::new("up"), KeyCode::Space);
        input_map.bind_key(&Action::new("down"), KeyCode::ControlLeft);
        input_map.bind_key(&Action::new("sprint"), KeyCode::ShiftLeft);

        let camera_controller = CameraController {
            move_speed: 5.0,
            ..CameraController::default()
        };

        Self {
            window: None,
            world: None,
            render_context: None,
            cursor_locked: false,
            last_frame_time: None,
            camera_controller,
            input_state: InputState::default(),
            input_map,
            demo_state: DemoState::new(),
            ik_entity: None,
            retarget_entity: None,
            additive_entity: None,
        }
    }
}

impl App {
    async fn setup_scene(
        window: Arc<Window>,
    ) -> Result<(
        World,
        RenderContext,
        praxis_ecs::Entity,
        praxis_ecs::Entity,
        praxis_ecs::Entity,
        praxis_ecs::Entity,
    )> {
        info!("Setting up advanced animation demo scene");

        let mut render_context = RenderContext::new(window.clone()).await?;
        Self::load_assets(&mut render_context)?;

        let mut world = World::new();

        // IK Demo character (at origin)
        let ik_skeleton = create_ik_skeleton();
        let mut ik_pose = AnimatedPose::new(ik_skeleton.bone_count());
        for i in 0..ik_skeleton.bone_count() {
            if let Some(bone) = ik_skeleton.bone(i) {
                ik_pose.set_local_transform(i, bone.bind_pose_matrix());
            }
        }
        ik_pose.update_world_transforms(&ik_skeleton);

        let ik_entity = world.spawn((
            Transform::from_xyz(-4.0, 0.0, 0.0),
            ik_skeleton,
            ik_pose,
            IKCharacter,
            praxis_ecs::Name::new("IK Character"),
        ));

        // Spawn bone markers for IK character
        for i in 0..4 {
            world.spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                praxis_ecs::MeshHandle::new("sphere"),
                praxis_ecs::TextureHandle::new("white"),
                praxis_ecs::Name::new(format!("IK Bone {i}")),
            ));
        }

        // Retargeting Demo character (in middle)
        let target_skeleton = create_target_skeleton();
        let mut retarget_pose = AnimatedPose::new(target_skeleton.bone_count());
        for i in 0..target_skeleton.bone_count() {
            if let Some(bone) = target_skeleton.bone(i) {
                retarget_pose.set_local_transform(i, bone.bind_pose_matrix());
            }
        }
        retarget_pose.update_world_transforms(&target_skeleton);

        let retarget_entity = world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            target_skeleton,
            retarget_pose,
            RetargetCharacter,
            praxis_ecs::Name::new("Retarget Character"),
        ));

        // Spawn bone markers for retarget character
        for i in 0..4 {
            world.spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                praxis_ecs::MeshHandle::new("sphere"),
                praxis_ecs::TextureHandle::new("white"),
                praxis_ecs::Name::new(format!("Retarget Bone {i}")),
            ));
        }

        // Additive Demo character (on right)
        let additive_skeleton = create_additive_skeleton();
        let mut additive_pose = AnimatedPose::new(additive_skeleton.bone_count());
        for i in 0..additive_skeleton.bone_count() {
            if let Some(bone) = additive_skeleton.bone(i) {
                additive_pose.set_local_transform(i, bone.bind_pose_matrix());
            }
        }
        additive_pose.update_world_transforms(&additive_skeleton);

        let additive_entity = world.spawn((
            Transform::from_xyz(4.0, 0.0, 0.0),
            additive_skeleton,
            additive_pose,
            AdditiveCharacter,
            praxis_ecs::Name::new("Additive Character"),
        ));

        // Spawn bone markers for additive character
        for i in 0..3 {
            world.spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                praxis_ecs::MeshHandle::new("sphere"),
                praxis_ecs::TextureHandle::new("white"),
                praxis_ecs::Name::new(format!("Additive Bone {i}")),
            ));
        }

        let camera_entity = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(0.0, 2.0, 8.0),
            70.0_f32.to_radians(),
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        ));

        info!("Scene setup complete");
        Ok((
            world,
            render_context,
            camera_entity,
            ik_entity,
            retarget_entity,
            additive_entity,
        ))
    }

    fn load_assets(render_context: &mut RenderContext) -> Result<()> {
        info!("Loading assets...");

        render_context
            .mesh_manager_mut()
            .load_mesh("sphere", sphere_mesh(0.1, 12, 8, [1.0, 1.0, 1.0]))?;

        let white_pixels: Vec<u8> = vec![255, 255, 255, 255];
        render_context
            .texture_manager_mut()
            .load_texture_from_bytes("white", &white_pixels, 1, 1)?;

        info!("Assets loaded");
        Ok(())
    }

    fn handle_input(&mut self) {
        if self.input_state.is_key_just_pressed(KeyCode::Digit1) {
            self.demo_state.current_mode = DemoMode::IK;
            println!("Switched to IK demo");
        } else if self.input_state.is_key_just_pressed(KeyCode::Digit2) {
            self.demo_state.current_mode = DemoMode::Retargeting;
            println!("Switched to Retargeting demo");
        } else if self.input_state.is_key_just_pressed(KeyCode::Digit3) {
            self.demo_state.current_mode = DemoMode::Additive;
            println!("Switched to Additive Blending demo");
        }
    }

    fn update_animations(&mut self, delta_time: f32) {
        if let Some(world) = &mut self.world {
            // Update IK demo
            self.demo_state.ik_time += delta_time;
            let t = self.demo_state.ik_time;
            let target_x = 2.0 + (t * 2.0).sin() * 0.5;
            let target_y = 1.5 + (t * 3.0).cos() * 0.3;

            if let Some(ik_entity) = self.ik_entity {
                let (skeleton, mut pose) = {
                    let inner = world.inner();
                    let skeleton = inner.get::<Skeleton>(ik_entity).unwrap().clone();
                    let pose = inner.get::<AnimatedPose>(ik_entity).unwrap().clone();
                    (skeleton, pose)
                };

                // Reset to bind pose
                for i in 0..skeleton.bone_count() {
                    if let Some(bone) = skeleton.bone(i) {
                        pose.set_local_transform(i, bone.bind_pose_matrix());
                    }
                }

                // Apply IK
                let ik_controller = create_ik_controller(Vec3::new(target_x, target_y, 0.0));
                ik_controller.apply(&mut pose, &skeleton);

                // Update entity
                if let Some(mut entity_pose) = world.inner_mut().get_mut::<AnimatedPose>(ik_entity)
                {
                    *entity_pose = pose;
                }
            }

            // Update Retargeting demo
            self.demo_state.retarget_time += delta_time;
            let anim_time = self.demo_state.retarget_time % 2.0;

            if let Some(retarget_entity) = self.retarget_entity {
                let source_skeleton = create_source_skeleton();
                let target_skeleton = {
                    let inner = world.inner();
                    inner
                        .get::<Skeleton>(retarget_entity)
                        .unwrap()
                        .clone()
                };

                let retargeter = AnimationRetargeter::auto(&source_skeleton, &target_skeleton);
                let source_clip = create_source_animation();
                let retargeted_clip = retargeter.retarget_clip(&source_clip, &target_skeleton);

                // Sample animation at current time
                let mut pose = AnimatedPose::new(target_skeleton.bone_count());
                for i in 0..target_skeleton.bone_count() {
                    if let Some(bone) = target_skeleton.bone(i) {
                        pose.set_local_transform(i, bone.bind_pose_matrix());
                    }
                }

                // Apply animation
                for (bone_index, track) in retargeted_clip.bone_tracks() {
                    if let Some(bone) = target_skeleton.bone(*bone_index) {
                        let translation = track
                            .sample_translation(anim_time)
                            .unwrap_or(bone.bind_pose_translation);
                        let rotation = track
                            .sample_rotation(anim_time)
                            .unwrap_or(bone.bind_pose_rotation);
                        let scale = track
                            .sample_scale(anim_time)
                            .unwrap_or(bone.bind_pose_scale);

                        let transform =
                            Mat4::from_scale_rotation_translation(scale, rotation, translation);
                        pose.set_local_transform(*bone_index, transform);
                    }
                }

                pose.update_world_transforms(&target_skeleton);

                // Update entity
                if let Some(mut entity_pose) =
                    world.inner_mut().get_mut::<AnimatedPose>(retarget_entity)
                {
                    *entity_pose = pose;
                }
            }

            // Update Additive demo
            self.demo_state.additive_time += delta_time;
            let additive_anim_time = self.demo_state.additive_time % 1.0;

            if let Some(additive_entity) = self.additive_entity {
                let skeleton = {
                    let inner = world.inner();
                    inner.get::<Skeleton>(additive_entity).unwrap().clone()
                };

                let walk_clip = create_walk_clip();
                let recoil_clip = create_recoil_clip();

                // Create base pose from walk animation
                let mut pose = AnimatedPose::new(skeleton.bone_count());
                for i in 0..skeleton.bone_count() {
                    if let Some(bone) = skeleton.bone(i) {
                        pose.set_local_transform(i, bone.bind_pose_matrix());
                    }
                }

                // Apply walk animation
                for (bone_index, track) in walk_clip.bone_tracks() {
                    if let Some(bone) = skeleton.bone(*bone_index) {
                        let rotation = track
                            .sample_rotation(additive_anim_time)
                            .unwrap_or(bone.bind_pose_rotation);
                        let translation = track
                            .sample_translation(additive_anim_time)
                            .unwrap_or(bone.bind_pose_translation);
                        let scale = track
                            .sample_scale(additive_anim_time)
                            .unwrap_or(bone.bind_pose_scale);

                        let transform =
                            Mat4::from_scale_rotation_translation(scale, rotation, translation);
                        pose.set_local_transform(*bone_index, transform);
                    }
                }

                // Apply additive animation
                let mut additive = AdditiveAnimation::new("Walk".to_string(), "Recoil".to_string())
                    .with_weight(1.0)
                    .with_mode(AdditiveMode::Local);

                additive.compute_reference_from_skeleton(&skeleton);
                let recoil_time = (self.demo_state.additive_time * 2.0) % 0.5;
                additive.apply(&mut pose, &recoil_clip, recoil_time, &skeleton);

                pose.update_world_transforms(&skeleton);

                // Update entity
                if let Some(mut entity_pose) =
                    world.inner_mut().get_mut::<AnimatedPose>(additive_entity)
                {
                    *entity_pose = pose;
                }
            }
        }
    }

    fn lock_cursor(&mut self) {
        if let Some(window) = &self.window {
            window.set_cursor_visible(false);
            let _ = window
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));
            self.cursor_locked = true;
        }
    }

    fn unlock_cursor(&mut self) {
        if let Some(window) = &self.window {
            window.set_cursor_visible(true);
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            self.cursor_locked = false;
        }
    }

    fn render_scene(&mut self) -> Result<()> {
        let world = self.world.as_mut().unwrap();
        let render_context = self.render_context.as_mut().unwrap();

        let camera_entity = self.camera_controller.camera_entity.unwrap();
        let matrices_copy = *world
            .inner()
            .get::<praxis_ecs::CameraMatrices>(camera_entity)
            .unwrap();

        let lighting = LightingUniforms {
            ambient_color: [0.5, 0.5, 0.5, 1.0],
            ..LightingUniforms::default()
        };

        // Update bone markers for IK character
        if let Some(ik_entity) = self.ik_entity {
            let (skeleton, pose, base_transform) = {
                let inner = world.inner();
                let skeleton = inner.get::<Skeleton>(ik_entity).unwrap();
                let pose = inner.get::<AnimatedPose>(ik_entity).unwrap();
                let transform = inner.get::<Transform>(ik_entity).unwrap();
                (skeleton.clone(), pose.clone(), *transform)
            };

            let world_transforms = compute_bone_world_transforms(&skeleton, &pose);

            let mut marker_query = world.query::<(&praxis_ecs::Name, &mut Transform)>();
            for (name, mut transform) in marker_query.iter_mut(world.inner_mut()) {
                if name.as_str().starts_with("IK Bone") {
                    if let Some(idx_str) = name.as_str().strip_prefix("IK Bone ") {
                        if let Ok(bone_index) = idx_str.parse::<usize>() {
                            if bone_index < world_transforms.len() {
                                let bone_world =
                                    base_transform.compute_matrix() * world_transforms[bone_index];
                                transform.translation = bone_world.col(3).truncate();
                                transform.scale = Vec3::splat(if bone_index == 3 { 1.2 } else { 0.8 });
                            }
                        }
                    }
                }
            }
        }

        // Update bone markers for Retarget character
        if let Some(retarget_entity) = self.retarget_entity {
            let (skeleton, pose, base_transform) = {
                let inner = world.inner();
                let skeleton = inner.get::<Skeleton>(retarget_entity).unwrap();
                let pose = inner.get::<AnimatedPose>(retarget_entity).unwrap();
                let transform = inner.get::<Transform>(retarget_entity).unwrap();
                (skeleton.clone(), pose.clone(), *transform)
            };

            let world_transforms = compute_bone_world_transforms(&skeleton, &pose);

            let mut marker_query = world.query::<(&praxis_ecs::Name, &mut Transform)>();
            for (name, mut transform) in marker_query.iter_mut(world.inner_mut()) {
                if name.as_str().starts_with("Retarget Bone") {
                    if let Some(idx_str) = name.as_str().strip_prefix("Retarget Bone ") {
                        if let Ok(bone_index) = idx_str.parse::<usize>() {
                            if bone_index < world_transforms.len() {
                                let bone_world =
                                    base_transform.compute_matrix() * world_transforms[bone_index];
                                transform.translation = bone_world.col(3).truncate();
                                transform.scale = Vec3::splat(if bone_index == 0 { 1.2 } else { 0.9 });
                            }
                        }
                    }
                }
            }
        }

        // Update bone markers for Additive character
        if let Some(additive_entity) = self.additive_entity {
            let (skeleton, pose, base_transform) = {
                let inner = world.inner();
                let skeleton = inner.get::<Skeleton>(additive_entity).unwrap();
                let pose = inner.get::<AnimatedPose>(additive_entity).unwrap();
                let transform = inner.get::<Transform>(additive_entity).unwrap();
                (skeleton.clone(), pose.clone(), *transform)
            };

            let world_transforms = compute_bone_world_transforms(&skeleton, &pose);

            let mut marker_query = world.query::<(&praxis_ecs::Name, &mut Transform)>();
            for (name, mut transform) in marker_query.iter_mut(world.inner_mut()) {
                if name.as_str().starts_with("Additive Bone") {
                    if let Some(idx_str) = name.as_str().strip_prefix("Additive Bone ") {
                        if let Ok(bone_index) = idx_str.parse::<usize>() {
                            if bone_index < world_transforms.len() {
                                let bone_world =
                                    base_transform.compute_matrix() * world_transforms[bone_index];
                                transform.translation = bone_world.col(3).truncate();
                                transform.scale = Vec3::splat(1.0);
                            }
                        }
                    }
                }
            }
        }

        let mut draw_commands = Vec::new();
        {
            let mut query = world.query::<(
                &Transform,
                &praxis_ecs::MeshHandle,
                &praxis_ecs::TextureHandle,
                &praxis_ecs::Name,
            )>();

            for (transform, mesh_handle, texture_handle, name) in query.iter(world.inner()) {
                let material = if name.as_str().starts_with("IK Bone") {
                    let color = if name.as_str().contains('3') {
                        [1.0, 0.2, 0.2, 1.0] // Red for hand (target)
                    } else if name.as_str().contains('0') {
                        [1.0, 0.8, 0.2, 1.0] // Yellow for shoulder
                    } else {
                        [0.2, 1.0, 0.2, 1.0] // Green for elbow/wrist
                    };
                    Some(
                        praxis_graphics::MaterialProperties::new()
                            .with_base_color(color)
                            .with_metallic(0.1)
                            .with_roughness(0.4),
                    )
                } else if name.as_str().starts_with("Retarget Bone") {
                    let color = if name.as_str().contains('0') {
                        [0.2, 0.5, 1.0, 1.0] // Blue for hips
                    } else if name.as_str().contains('1') {
                        [0.5, 0.7, 1.0, 1.0] // Light blue for spine
                    } else {
                        [0.7, 0.9, 1.0, 1.0] // Lighter blue for arms
                    };
                    Some(
                        praxis_graphics::MaterialProperties::new()
                            .with_base_color(color)
                            .with_metallic(0.1)
                            .with_roughness(0.4),
                    )
                } else if name.as_str().starts_with("Additive Bone") {
                    let color = if name.as_str().contains('0') {
                        [1.0, 0.5, 0.0, 1.0] // Orange for root
                    } else if name.as_str().contains('1') {
                        [1.0, 0.7, 0.2, 1.0] // Light orange for spine
                    } else {
                        [1.0, 0.3, 0.6, 1.0] // Pink for chest (affected by additive)
                    };
                    Some(
                        praxis_graphics::MaterialProperties::new()
                            .with_base_color(color)
                            .with_metallic(0.1)
                            .with_roughness(0.4),
                    )
                } else {
                    None
                };

                draw_commands.push(DrawCommand {
                    mesh_id: mesh_handle.id.clone(),
                    model: transform.compute_matrix(),
                    texture_name: Some(texture_handle.id.clone()),
                    material_properties: material,
                });
            }
        }

        let cmds = RenderCommands {
            view: matrices_copy.view,
            proj: matrices_copy.projection,
            draw_commands: &draw_commands,
            lighting: Some(&lighting),
        };

        render_context.render(&cmds)?;

        Ok(())
    }

    fn update_camera(
        camera_entity: praxis_ecs::Entity,
        camera_controller: &CameraController,
        input_state: &InputState,
        input_map: &InputMap,
        world: &mut World,
    ) {
        let mut velocity = Vec3::ZERO;

        if input_map.is_action_pressed(&Action::new("forward"), input_state) {
            velocity.z -= 1.0;
        }
        if input_map.is_action_pressed(&Action::new("backward"), input_state) {
            velocity.z += 1.0;
        }
        if input_map.is_action_pressed(&Action::new("left"), input_state) {
            velocity.x -= 1.0;
        }
        if input_map.is_action_pressed(&Action::new("right"), input_state) {
            velocity.x += 1.0;
        }
        if input_map.is_action_pressed(&Action::new("up"), input_state) {
            velocity.y += 1.0;
        }
        if input_map.is_action_pressed(&Action::new("down"), input_state) {
            velocity.y -= 1.0;
        }

        if velocity.length_squared() > 0.0 {
            velocity = velocity.normalize();
        }

        let mut speed = camera_controller.move_speed;
        if input_map.is_action_pressed(&Action::new("sprint"), input_state) {
            speed *= camera_controller.sprint_multiplier;
        }

        let dt = 1.0 / 60.0;

        if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(camera_entity) {
            transform.rotation = camera_controller.get_rotation();

            let forward = transform.rotation * Vec3::NEG_Z;
            let right = transform.rotation * Vec3::X;
            let up = Vec3::Y;

            transform.translation += forward * velocity.z * speed * dt;
            transform.translation += right * velocity.x * speed * dt;
            transform.translation += up * velocity.y * speed * dt;
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Application resumed, initializing...");

        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_title("Praxis - Advanced Animation Demo")
                .with_resizable(true),
        ) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                eprintln!("Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        let (world, render_context, camera_entity, ik_entity, retarget_entity, additive_entity) =
            match pollster::block_on(Self::setup_scene(window.clone())) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to setup scene: {e}");
                    event_loop.exit();
                    return;
                }
            };

        self.camera_controller.camera_entity = Some(camera_entity);
        self.ik_entity = Some(ik_entity);
        self.retarget_entity = Some(retarget_entity);
        self.additive_entity = Some(additive_entity);

        println!("\n╔════════════════════════════════════════════════════════╗");
        println!("║    PRAXIS - ADVANCED ANIMATION FEATURES DEMO         ║");
        println!("╚════════════════════════════════════════════════════════╝");
        println!("\n✨ FEATURES DEMONSTRATED:");
        println!("  🎯 Inverse Kinematics (IK) - Procedural limb positioning");
        println!("  🔄 Animation Retargeting - Transfer animations between skeletons");
        println!("  ➕ Additive Animation Blending - Layer animations on top of base");
        println!("  👁️  Visual rendering with 3D bone markers");
        println!("\n⌨️  CAMERA CONTROLS:");
        println!("  WASD        - Move horizontally");
        println!("  Space       - Move up");
        println!("  Left Ctrl   - Move down");
        println!("  Left Shift  - Sprint");
        println!("  Mouse       - Look around");
        println!("\n🎮 DEMO SELECTION:");
        println!("  1           - IK Demo (left) - Arm reaching for moving target");
        println!("  2           - Retargeting Demo (center) - Different skeleton sizes");
        println!("  3           - Additive Demo (right) - Walk + Recoil combined");
        println!("\n💾 SYSTEM:");
        println!("  ESC         - Toggle cursor / Exit");
        println!("\n💡 BONE COLOR LEGEND:");
        println!("  Left (IK):          🟡 Yellow=Shoulder  🟢 Green=Elbow/Wrist  🔴 Red=Hand");
        println!("  Center (Retarget):  🔵 Blue shades show retargeted skeleton");
        println!("  Right (Additive):   🟠 Orange=Root/Spine  🎀 Pink=Chest (recoil)");
        println!("\n▶️  Starting with IK demo...\n");

        self.window = Some(window);
        self.world = Some(world);
        self.render_context = Some(render_context);
        self.last_frame_time = Some(Instant::now());

        self.lock_cursor();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if self.world.is_none() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested, exiting...");
                event_loop.exit();
            }
            WindowEvent::Focused(focused) => {
                if focused && self.cursor_locked {
                    self.lock_cursor();
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(render_context) = &mut self.render_context {
                    render_context.configure_surface(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta = if let Some(last_time) = self.last_frame_time {
                    now.duration_since(last_time).as_secs_f32()
                } else {
                    1.0 / 60.0
                };
                self.last_frame_time = Some(now);

                self.input_state.update();
                self.handle_input();

                self.update_animations(delta);

                if let Some(camera_entity) = self.camera_controller.camera_entity {
                    if let Some(world) = &mut self.world {
                        Self::update_camera(
                            camera_entity,
                            &self.camera_controller,
                            &self.input_state,
                            &self.input_map,
                            world,
                        );
                    }
                }

                if let Some(camera_entity) = self.camera_controller.camera_entity {
                    if let Some(world) = &mut self.world {
                        let inner = world.inner_mut();
                        if let Some(transform) = inner.get::<Transform>(camera_entity) {
                            if let Some(projection) =
                                inner.get::<praxis_ecs::PerspectiveProjection>(camera_entity)
                            {
                                let view = praxis_math::Mat4::look_at_rh(
                                    transform.translation,
                                    transform.translation
                                        + (transform.rotation * praxis_math::Vec3::NEG_Z),
                                    praxis_math::Vec3::Y,
                                );
                                let proj_matrix = projection.compute_matrix();

                                if let Some(mut matrices) =
                                    inner.get_mut::<praxis_ecs::CameraMatrices>(camera_entity)
                                {
                                    matrices.update(view, proj_matrix);
                                }
                            }
                        }
                    }
                }

                if let Err(e) = self.render_scene() {
                    eprintln!("Render error: {e}");
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if self.cursor_locked {
                    println!("Cursor unlocked. Press ESC again to exit.");
                    self.unlock_cursor();
                } else {
                    info!("Exiting...");
                    event_loop.exit();
                }
            }
            _ => {
                praxis_input::winit_integration::process_window_event(
                    &mut self.input_state,
                    &event,
                );
            }
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if !self.cursor_locked {
            return;
        }

        if let DeviceEvent::MouseMotion { delta } = event {
            self.camera_controller
                .update_rotation(delta.0 as f32, delta.1 as f32);
        }
    }
}

#[cfg(not(feature = "headless"))]
fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_input::init()?;
    praxis_ecs::init()?;

    let event_loop = EventLoop::new()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!(
        "animation_advanced_demo example requires graphics support and cannot run in headless mode"
    );
    Ok(())
}
