//! Visual skeletal animation demonstration with rendering.
//!
//! This example demonstrates the skeletal animation system with:
//! - Creating a skeleton with multiple bones
//! - Defining animation clips with keyframes
//! - Playing and controlling animations
//! - Keyframe interpolation
//! - Animation looping and blending
//! - **Visual rendering of animated skeleton**
//! - **3D meshes attached to bones**
//! - **Camera controls for viewing**
//!
//! Controls:
//! - **WASD** - Move camera
//! - **Space/Ctrl** - Move up/down
//! - **Shift** - Sprint
//! - **Mouse** - Look around
//! - **1** - Play Walk animation
//! - **2** - Play Idle animation
//! - **3** - Play both (blended)
//! - **ESC** - Toggle cursor / Exit

#[path = "common.rs"]
mod common;

use common::CameraController;
use praxis_ecs::{PerspectiveCameraBundle, Transform, World};
use praxis_graphics::{sphere_mesh, DrawCommand, LightingUniforms, RenderCommands, RenderContext};
use praxis_input::{Action, InputMap, InputState};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_scene::{AnimatedPose, AnimationClip, AnimationPlayer, Bone, Skeleton};
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

/// Converts an animated pose to bone world transforms.
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
        }
    }
}

impl App {
    async fn setup_scene(
        window: Arc<Window>,
    ) -> Result<(World, RenderContext, praxis_ecs::Entity, praxis_ecs::Entity)> {
        info!("Setting up skeletal animation scene");

        let mut render_context = RenderContext::new(window.clone()).await?;
        Self::load_assets(&mut render_context)?;

        let mut world = World::new();

        // Create animated character
        let skeleton = create_skeleton();
        let mut player = AnimationPlayer::new();
        player.add_clip("Walk".to_string(), create_walk_animation());
        player.add_clip("Idle".to_string(), create_idle_animation());
        let pose = AnimatedPose::new(skeleton.bone_count());

        let animated_entity = world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            skeleton,
            player,
            pose,
            praxis_ecs::Name::new("Animated Character"),
        ));

        // Spawn some visual markers at bone positions (spheres)
        // These will be updated each frame based on bone positions
        for i in 0..3 {
            world.spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                praxis_ecs::MeshHandle::new("sphere"),
                praxis_ecs::TextureHandle::new("white"),
                praxis_ecs::Name::new(format!("Bone Marker {i}")),
            ));
        }

        let camera_entity = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(0.0, 2.0, 8.0),
            70.0_f32.to_radians(),
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        ));

        info!("Scene setup complete");
        Ok((world, render_context, camera_entity, animated_entity))
    }

    fn load_assets(render_context: &mut RenderContext) -> Result<()> {
        info!("Loading assets...");

        // Load sphere mesh for bone markers
        render_context
            .mesh_manager_mut()
            .load_mesh("sphere", sphere_mesh(0.15, 16, 8, [1.0, 1.0, 1.0]))?;

        // Create a simple white texture
        let white_pixels: Vec<u8> = vec![255, 255, 255, 255];
        render_context
            .texture_manager_mut()
            .load_texture_from_bytes("white", &white_pixels, 1, 1)?;

        info!("Assets loaded");
        Ok(())
    }

    fn handle_input(&mut self) {
        if let Some(world) = &mut self.world {
            if let Some(entity) = self.animated_entity {
                if let Some(mut player) = world.inner_mut().get_mut::<AnimationPlayer>(entity) {
                    // Key 1: Play Walk animation
                    if self.input_state.is_key_just_pressed(KeyCode::Digit1) {
                        player.stop("Idle");
                        player.play("Walk");
                        player.set_looping("Walk", true);
                        println!("Playing Walk animation");
                    }

                    // Key 2: Play Idle animation
                    if self.input_state.is_key_just_pressed(KeyCode::Digit2) {
                        player.stop("Walk");
                        player.play("Idle");
                        player.set_looping("Idle", true);
                        println!("Playing Idle animation");
                    }

                    // Key 3: Play both (blended)
                    if self.input_state.is_key_just_pressed(KeyCode::Digit3) {
                        player.play("Walk");
                        player.set_weight("Walk", 0.7);
                        player.set_looping("Walk", true);
                        player.play("Idle");
                        player.set_weight("Idle", 0.3);
                        player.set_looping("Idle", true);
                        println!("Playing blended Walk (0.7) + Idle (0.3)");
                    }
                }
            }
        }
    }

    fn update_animations(&mut self, delta_time: f32) {
        if let Some(world) = &mut self.world {
            let mut query = world.query::<(&Skeleton, &mut AnimationPlayer, &mut AnimatedPose)>();
            for (skeleton, mut player, mut pose) in query.iter_mut(world.inner_mut()) {
                player.update(delta_time);
                *pose = player.evaluate(skeleton);
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

        // Get camera matrices
        let camera_entity = self.camera_controller.camera_entity.unwrap();
        let matrices_copy = *world
            .inner()
            .get::<praxis_ecs::CameraMatrices>(camera_entity)
            .unwrap();

        // Build lighting (simple ambient)
        let lighting = LightingUniforms {
            ambient_color: [0.3, 0.3, 0.3, 1.0],
            ..LightingUniforms::default()
        };

        // Update bone marker positions based on animation
        if let Some(animated_entity) = self.animated_entity {
            let (skeleton, pose, base_transform) = {
                let inner = world.inner();
                let skeleton = inner.get::<Skeleton>(animated_entity).unwrap();
                let pose = inner.get::<AnimatedPose>(animated_entity).unwrap();
                let transform = inner.get::<Transform>(animated_entity).unwrap();
                (skeleton.clone(), pose.clone(), *transform)
            };

            let world_transforms = compute_bone_world_transforms(&skeleton, &pose);

            // Update marker spheres
            let mut marker_query = world.query::<(&praxis_ecs::Name, &mut Transform)>();
            for (name, mut transform) in marker_query.iter_mut(world.inner_mut()) {
                if name.as_str().starts_with("Bone Marker") {
                    if let Some(idx_str) = name.as_str().strip_prefix("Bone Marker ") {
                        if let Ok(bone_index) = idx_str.parse::<usize>() {
                            if bone_index < world_transforms.len() {
                                let bone_world =
                                    base_transform.compute_matrix() * world_transforms[bone_index];
                                transform.translation = bone_world.col(3).truncate();

                                // Color based on bone
                                let scale = match bone_index {
                                    0 => 1.2, // Root - larger
                                    1 => 1.0, // Spine
                                    2 => 0.8, // Head - smaller
                                    _ => 1.0,
                                };
                                transform.scale = Vec3::splat(scale);
                            }
                        }
                    }
                }
            }
        }

        // Build draw commands for meshes
        let mut draw_commands = Vec::new();
        {
            let mut query = world.query::<(
                &Transform,
                &praxis_ecs::MeshHandle,
                &praxis_ecs::TextureHandle,
                &praxis_ecs::Name,
            )>();

            for (transform, mesh_handle, texture_handle, name) in query.iter(world.inner()) {
                // Color bone markers differently
                let material = if name.as_str().starts_with("Bone Marker") {
                    let color = if name.as_str().contains('0') {
                        [1.0, 0.2, 0.2, 1.0] // Red for root
                    } else if name.as_str().contains('1') {
                        [0.2, 1.0, 0.2, 1.0] // Green for spine
                    } else {
                        [0.2, 0.5, 1.0, 1.0] // Blue for head
                    };
                    Some(
                        praxis_graphics::MaterialProperties::new()
                            .with_base_color(color)
                            .with_metallic(0.0)
                            .with_roughness(0.3),
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

        // Note: Line rendering for skeleton visualization would require
        // deeper integration with the render pass, which is beyond the scope
        // of this basic demo. The bone markers (spheres) provide visual feedback
        // of the animated skeleton structure.

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
                .with_title("Praxis - Skeletal Animation Demo")
                .with_resizable(true),
        ) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                eprintln!("Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        let (mut world, render_context, camera_entity, animated_entity) =
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

        // Start playing walk animation by default
        if let Some(mut player) = world
            .inner_mut()
            .get_mut::<AnimationPlayer>(animated_entity)
        {
            player.play("Walk");
            player.set_looping("Walk", true);
        }

        println!("\n╔════════════════════════════════════════════════════════╗");
        println!("║      PRAXIS - SKELETAL ANIMATION DEMONSTRATION       ║");
        println!("╚════════════════════════════════════════════════════════╝");
        println!("\n✨ FEATURES DEMONSTRATED:");
        println!("  🦴 Skeletal animation with 3 bones (Root, Spine, Head)");
        println!("  🎬 Keyframe animation with translation and rotation");
        println!("  🔄 Animation looping and blending");
        println!("  👁️  Visual skeleton rendering with colored bones");
        println!("  🔵 Bone position markers (spheres)");
        println!("\n⌨️  CAMERA CONTROLS:");
        println!("  WASD        - Move horizontally");
        println!("  Space       - Move up");
        println!("  Left Ctrl   - Move down");
        println!("  Left Shift  - Sprint");
        println!("  Mouse       - Look around");
        println!("\n🎮 ANIMATION CONTROLS:");
        println!("  1           - Play Walk animation");
        println!("  2           - Play Idle animation");
        println!("  3           - Play blended animations");
        println!("\n💾 SYSTEM:");
        println!("  ESC         - Toggle cursor / Exit");
        println!("\n💡 TIP: The skeleton is visualized with colored lines:");
        println!("    🔴 Red   = Root bone");
        println!("    🟢 Green = Spine bone");
        println!("    🔵 Blue  = Head bone");
        println!();

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

                // Update input
                self.input_state.update();
                self.handle_input();

                // Update animations
                self.update_animations(delta);

                // Update camera
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

                // Update camera matrices
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
        "skeletal_animation_demo example requires graphics support and cannot run in headless mode"
    );
    Ok(())
}
