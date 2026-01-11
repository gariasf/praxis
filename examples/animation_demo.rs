//! Interactive animation demonstration with blend transitions.
//!
//! This example showcases:
//! - Animated character with skeletal animation
//! - Smooth cross-fade transitions between animations
//! - 1D blend trees for speed-based blending
//! - Interactive controls to switch between animations
//! - Real-time animation state updates
//! - **Visual rendering of animated skeleton with meshes**
//! - **3D spheres attached to bones showing animation**
//!
//! Controls:
//! - WASD - Move camera
//! - Space/Ctrl - Move up/down
//! - Shift - Sprint
//! - Mouse - Look around
//! - 1 - Switch to Idle animation (with cross-fade)
//! - 2 - Switch to Walk animation (with cross-fade)
//! - 3 - Switch to Run animation (with cross-fade)
//! - 4 - Activate speed blend tree
//! - Arrow Up/Down - Adjust speed parameter (in blend tree mode)
//! - ESC - Toggle cursor / Exit

#[path = "common.rs"]
mod common;

use common::CameraController;
use praxis_ecs::{Component, PerspectiveCameraBundle, Transform, World};
use praxis_graphics::{sphere_mesh, DrawCommand, LightingUniforms, RenderCommands, RenderContext};
use praxis_input::{Action, InputMap, InputState};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_scene::{AnimatedPose, AnimationBlender, AnimationClip, BlendNode1D, Bone, Skeleton};
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
}

impl DemoState {
    fn new() -> Self {
        Self {
            current_mode: AnimationMode::Idle,
            speed_parameter: 0.0,
            last_mode_change: Instant::now(),
        }
    }
}

#[derive(Component)]
struct AnimatedCharacter;

fn create_character_skeleton() -> Skeleton {
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

    blender.add_clip("Idle", create_idle_animation());
    blender.add_clip("Walk", create_walk_animation());
    blender.add_clip("Run", create_run_animation());

    let mut blend_tree = BlendNode1D::new();
    blend_tree.add_clip("Idle", 0.0);
    blend_tree.add_clip("Walk", 0.5);
    blend_tree.add_clip("Run", 1.0);

    blender.add_blend_tree("SpeedBlend", blend_tree.into());

    blender
}

fn compute_bone_world_transforms(skeleton: &Skeleton, pose: &AnimatedPose) -> Vec<Mat4> {
    let mut world_transforms = Vec::new();

    for bone_index in 0..skeleton.bone_count() {
        let local_transform = pose.local_transform(bone_index).unwrap_or(Mat4::IDENTITY);

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

struct App {
    window: Option<Arc<Window>>,
    world: Option<World>,
    render_context: Option<RenderContext>,
    cursor_locked: bool,
    last_frame_time: Option<Instant>,
    camera_controller: CameraController,
    input_state: InputState,
    input_map: InputMap,
    animated_entity: Option<praxis_ecs::Entity>,
    demo_state: DemoState,
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
            animated_entity: None,
            demo_state: DemoState::new(),
        }
    }
}

impl App {
    async fn setup_scene(
        window: Arc<Window>,
    ) -> Result<(World, RenderContext, praxis_ecs::Entity, praxis_ecs::Entity)> {
        info!("Setting up animation blend demo scene");

        let mut render_context = RenderContext::new(window.clone()).await?;
        Self::load_assets(&mut render_context)?;

        let mut world = World::new();

        let skeleton = create_character_skeleton();
        let mut blender = create_animation_blender();
        blender.play("Idle");
        let pose = AnimatedPose::new(skeleton.bone_count());

        let animated_entity = world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            skeleton,
            blender,
            pose,
            AnimatedCharacter,
            praxis_ecs::Name::new("Animated Character"),
        ));

        // Spawn bone marker spheres
        for i in 0..10 {
            world.spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                praxis_ecs::MeshHandle::new("sphere"),
                praxis_ecs::TextureHandle::new("white"),
                praxis_ecs::Name::new(format!("Bone Marker {i}")),
            ));
        }

        let camera_entity = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(0.0, 2.0, 5.0),
            70.0_f32.to_radians(),
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        ));

        info!("Scene setup complete");
        Ok((world, render_context, camera_entity, animated_entity))
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
        let now = Instant::now();
        let time_since_change = now
            .duration_since(self.demo_state.last_mode_change)
            .as_secs_f32();

        if time_since_change < 0.3 {
            return;
        }

        let mut mode_changed = false;
        let old_mode = self.demo_state.current_mode;

        if self.input_state.is_key_just_pressed(KeyCode::Digit1) {
            self.demo_state.current_mode = AnimationMode::Idle;
            mode_changed = true;
        } else if self.input_state.is_key_just_pressed(KeyCode::Digit2) {
            self.demo_state.current_mode = AnimationMode::Walk;
            mode_changed = true;
        } else if self.input_state.is_key_just_pressed(KeyCode::Digit3) {
            self.demo_state.current_mode = AnimationMode::Run;
            mode_changed = true;
        } else if self.input_state.is_key_just_pressed(KeyCode::Digit4) {
            self.demo_state.current_mode = AnimationMode::BlendTree;
            mode_changed = true;
        }

        if self.demo_state.current_mode == AnimationMode::BlendTree {
            if self.input_state.is_key_pressed(KeyCode::ArrowUp) {
                self.demo_state.speed_parameter = (self.demo_state.speed_parameter + 0.02).min(1.0);
            }
            if self.input_state.is_key_pressed(KeyCode::ArrowDown) {
                self.demo_state.speed_parameter = (self.demo_state.speed_parameter - 0.02).max(0.0);
            }
        }

        if mode_changed {
            if let Some(world) = &mut self.world {
                if let Some(entity) = self.animated_entity {
                    if let Some(mut blender) = world.inner_mut().get_mut::<AnimationBlender>(entity)
                    {
                        match self.demo_state.current_mode {
                            AnimationMode::Idle => {
                                if old_mode != AnimationMode::Idle {
                                    let from = match old_mode {
                                        AnimationMode::Walk => "Walk",
                                        AnimationMode::Run => "Run",
                                        _ => "Idle",
                                    };
                                    blender.cross_fade(from, "Idle", 0.3);
                                    println!("Cross-fading to Idle animation");
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
                                    println!("Cross-fading to Walk animation");
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
                                    println!("Cross-fading to Run animation");
                                }
                            }
                            AnimationMode::BlendTree => {
                                blender.activate_blend_tree("SpeedBlend");
                                println!("Activated speed blend tree (use arrow keys to adjust)");
                            }
                        }
                    }
                }
            }
            self.demo_state.last_mode_change = now;
        }

        if self.demo_state.current_mode == AnimationMode::BlendTree {
            if let Some(world) = &mut self.world {
                if let Some(entity) = self.animated_entity {
                    if let Some(mut blender) = world.inner_mut().get_mut::<AnimationBlender>(entity)
                    {
                        blender.set_blend_parameter("SpeedBlend", self.demo_state.speed_parameter);
                    }
                }
            }
        }
    }

    fn update_animations(&mut self, delta_time: f32) {
        if let Some(world) = &mut self.world {
            let mut query = world.query::<(&Skeleton, &mut AnimationBlender, &mut AnimatedPose)>();
            for (skeleton, mut blender, mut pose) in query.iter_mut(world.inner_mut()) {
                blender.update(delta_time);
                *pose = blender.evaluate(skeleton);
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
            ambient_color: [0.4, 0.4, 0.4, 1.0],
            ..LightingUniforms::default()
        };

        if let Some(animated_entity) = self.animated_entity {
            let (skeleton, pose, base_transform) = {
                let inner = world.inner();
                let skeleton = inner.get::<Skeleton>(animated_entity).unwrap();
                let pose = inner.get::<AnimatedPose>(animated_entity).unwrap();
                let transform = inner.get::<Transform>(animated_entity).unwrap();
                (skeleton.clone(), pose.clone(), *transform)
            };

            let world_transforms = compute_bone_world_transforms(&skeleton, &pose);

            let mut marker_query = world.query::<(&praxis_ecs::Name, &mut Transform)>();
            for (name, mut transform) in marker_query.iter_mut(world.inner_mut()) {
                if name.as_str().starts_with("Bone Marker") {
                    if let Some(idx_str) = name.as_str().strip_prefix("Bone Marker ") {
                        if let Ok(bone_index) = idx_str.parse::<usize>() {
                            if bone_index < world_transforms.len() {
                                let bone_world =
                                    base_transform.compute_matrix() * world_transforms[bone_index];
                                transform.translation = bone_world.col(3).truncate();

                                let scale = match bone_index {
                                    0 => 1.5,     // Root - larger
                                    1 | 2 => 1.2, // Spine/Chest
                                    3 => 1.0,     // Head
                                    _ => 0.8,     // Limbs
                                };
                                transform.scale = Vec3::splat(scale);
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
                let material = if name.as_str().starts_with("Bone Marker") {
                    let color = if name.as_str().contains('0') {
                        [1.0, 0.2, 0.2, 1.0] // Red for root
                    } else if name.as_str().contains('1') {
                        [1.0, 0.8, 0.2, 1.0] // Yellow for spine
                    } else if name.as_str().contains('2') {
                        [1.0, 0.5, 0.2, 1.0] // Orange for chest
                    } else if name.as_str().contains('3') {
                        [0.2, 0.5, 1.0, 1.0] // Blue for head
                    } else if name.as_str().contains('4') || name.as_str().contains('6') {
                        [0.5, 1.0, 0.5, 1.0] // Green for shoulders
                    } else if name.as_str().contains('5') || name.as_str().contains('7') {
                        [0.2, 1.0, 0.2, 1.0] // Bright green for arms
                    } else {
                        [0.8, 0.2, 1.0, 1.0] // Purple for legs
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
                    bone_matrices: None, // Bone markers don't use skeletal animation
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
                .with_title("Praxis - Animation Blend Demo")
                .with_resizable(true),
        ) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                eprintln!("Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        let (world, render_context, camera_entity, animated_entity) =
            match pollster::block_on(Self::setup_scene(window.clone())) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to setup scene: {e}");
                    event_loop.exit();
                    return;
                }
            };

        self.camera_controller.camera_entity = Some(camera_entity);
        self.animated_entity = Some(animated_entity);

        println!("\n╔════════════════════════════════════════════════════════╗");
        println!("║    PRAXIS - ANIMATION BLENDING DEMONSTRATION         ║");
        println!("╚════════════════════════════════════════════════════════╝");
        println!("\n✨ FEATURES DEMONSTRATED:");
        println!("  🦴 Skeletal animation with 10 bones (humanoid character)");
        println!("  🎬 Cross-fade transitions between animations");
        println!("  🔀 1D blend tree for speed-based blending");
        println!("  👁️  Visual skeleton rendering with colored bone markers");
        println!("  🎨 Different colors per bone (Root=Red, Spine=Yellow, etc.)");
        println!("\n⌨️  CAMERA CONTROLS:");
        println!("  WASD        - Move horizontally");
        println!("  Space       - Move up");
        println!("  Left Ctrl   - Move down");
        println!("  Left Shift  - Sprint");
        println!("  Mouse       - Look around");
        println!("\n🎮 ANIMATION CONTROLS:");
        println!("  1           - Switch to Idle animation (cross-fade)");
        println!("  2           - Switch to Walk animation (cross-fade)");
        println!("  3           - Switch to Run animation (cross-fade)");
        println!("  4           - Activate speed blend tree");
        println!("  Arrow Up    - Increase speed parameter (blend tree mode)");
        println!("  Arrow Down  - Decrease speed parameter (blend tree mode)");
        println!("\n💾 SYSTEM:");
        println!("  ESC         - Toggle cursor / Exit");
        println!("\n💡 BONE COLOR LEGEND:");
        println!("    🔴 Red        = Root/Hips");
        println!("    🟡 Yellow     = Spine");
        println!("    🟠 Orange     = Chest");
        println!("    🔵 Blue       = Head");
        println!("    🟢 Green      = Shoulders");
        println!("    🟢 Bright Grn = Arms");
        println!("    🟣 Purple     = Legs");
        println!("\n▶️  Starting with Idle animation...\n");

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
    println!("animation_demo example requires graphics support and cannot run in headless mode");
    Ok(())
}
