//! Demonstrates async mesh streaming with background loading and frustum culling.
//!
//! This example shows:
//! - Background thread mesh loading with priority queue
//! - Frustum-based on-demand mesh loading
//! - Loading state visualization with color indicators
//! - Priority-based loading based on distance and visibility
//! - Camera controls to stress test the streaming system
//!
//! # Controls
//!
//! - **W/A/S/D**: Move camera forward/left/back/right
//! - **Q/E**: Move camera down/up
//! - **Arrow Keys**: Rotate camera
//! - **Space**: Reset camera to default position
//! - **R**: Toggle rapid camera movement (stress test)
//! - **ESC**: Exit
//!
//! # Loading State Visualization
//!
//! Objects are colored based on their streaming state:
//! - **Gray**: Unloaded (outside frustum, low priority)
//! - **Yellow**: Queued for loading (in frustum, waiting)
//! - **Cyan**: Currently loading (background thread)
//! - **Green**: Loaded and ready (fully streamed)
//! - **Red**: Failed to load (error state)

use praxis::praxis_graphics::{
    colored_cube_mesh, solid_cube_mesh, sphere_mesh, DrawCommand, MeshData, MeshStreamingState,
    MeshStreamingSystem, RenderCommands, RenderContext,
};
use praxis::praxis_math::{Mat4, Quat, Vec3};
use praxis::praxis_spatial::Frustum;
use praxis::praxis_utils::{error, info, trace, Result};
use praxis::praxis_window::WindowManager;
use std::collections::HashMap;
use std::sync::Arc;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Types of meshes we can stream
#[derive(Debug, Clone, Copy)]
enum MeshType {
    Cube,
    Sphere,
}

/// Object in the scene with position and mesh info
struct SceneObject {
    position: Vec3,
    mesh_id: String,
    mesh_type: MeshType,
}

/// Camera controller with keyboard input
struct CameraController {
    position: Vec3,
    rotation: Quat,
    yaw: f32,
    pitch: f32,
    move_speed: f32,
    rotate_speed: f32,
    rapid_movement: bool,
    // Input states
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
    rotate_left: bool,
    rotate_right: bool,
    rotate_up: bool,
    rotate_down: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 20.0, 60.0),
            rotation: Quat::IDENTITY,
            yaw: 0.0,
            pitch: -0.3,
            move_speed: 30.0,
            rotate_speed: 2.0,
            rapid_movement: false,
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            move_up: false,
            move_down: false,
            rotate_left: false,
            rotate_right: false,
            rotate_up: false,
            rotate_down: false,
        }
    }
}

impl CameraController {
    fn reset(&mut self) {
        self.position = Vec3::new(0.0, 20.0, 60.0);
        self.yaw = 0.0;
        self.pitch = -0.3;
        self.update_rotation();
    }

    fn update_rotation(&mut self) {
        self.rotation = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);
    }

    fn update(&mut self, delta_time: f32) {
        // Update rotation
        let mut yaw_delta = 0.0;
        let mut pitch_delta = 0.0;

        if self.rotate_left {
            yaw_delta += self.rotate_speed * delta_time;
        }
        if self.rotate_right {
            yaw_delta -= self.rotate_speed * delta_time;
        }
        if self.rotate_up {
            pitch_delta += self.rotate_speed * delta_time;
        }
        if self.rotate_down {
            pitch_delta -= self.rotate_speed * delta_time;
        }

        self.yaw += yaw_delta;
        self.pitch = (self.pitch + pitch_delta).clamp(-1.5, 1.5);
        self.update_rotation();

        // Update position
        let forward = self.rotation * Vec3::new(0.0, 0.0, -1.0);
        let right = self.rotation * Vec3::new(1.0, 0.0, 0.0);
        let up = Vec3::Y;

        let mut velocity = Vec3::ZERO;

        if self.move_forward {
            velocity += forward;
        }
        if self.move_backward {
            velocity -= forward;
        }
        if self.move_right {
            velocity += right;
        }
        if self.move_left {
            velocity -= right;
        }
        if self.move_up {
            velocity += up;
        }
        if self.move_down {
            velocity -= up;
        }

        if velocity.length_squared() > 0.0 {
            velocity = velocity.normalize();
            let speed = if self.rapid_movement {
                self.move_speed * 5.0
            } else {
                self.move_speed
            };
            self.position += velocity * speed * delta_time;
        }
    }

    fn handle_input(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state,
                        ..
                    },
                ..
            } => {
                let pressed = *state == ElementState::Pressed;

                match keycode {
                    // Camera movement
                    KeyCode::KeyW => self.move_forward = pressed,
                    KeyCode::KeyS => self.move_backward = pressed,
                    KeyCode::KeyA => self.move_left = pressed,
                    KeyCode::KeyD => self.move_right = pressed,
                    KeyCode::KeyQ => self.move_down = pressed,
                    KeyCode::KeyE => self.move_up = pressed,
                    // Camera rotation
                    KeyCode::ArrowLeft => self.rotate_left = pressed,
                    KeyCode::ArrowRight => self.rotate_right = pressed,
                    KeyCode::ArrowUp => self.rotate_up = pressed,
                    KeyCode::ArrowDown => self.rotate_down = pressed,
                    // Reset camera
                    KeyCode::Space if pressed => {
                        self.reset();
                        info!("Camera reset to default position");
                    }
                    // Toggle rapid movement
                    KeyCode::KeyR if pressed => {
                        self.rapid_movement = !self.rapid_movement;
                        info!(
                            "Rapid camera movement: {} (stress test mode)",
                            if self.rapid_movement { "ON" } else { "OFF" }
                        );
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn get_view_matrix(&self) -> Mat4 {
        let forward = self.rotation * Vec3::new(0.0, 0.0, -1.0);
        let target = self.position + forward;
        Mat4::look_at_rh(self.position, target, Vec3::Y)
    }
}

struct MeshStreamingDemo {
    render_context: RenderContext,
    streaming_system: MeshStreamingSystem,
    mesh_database: HashMap<String, MeshData>,
    scene_objects: Vec<SceneObject>,
    camera: CameraController,
    stats_timer: f32,
    frame_count: u32,
}

impl MeshStreamingDemo {
    async fn new(window: Arc<winit::window::Window>) -> Result<Self> {
        let mut render_context = RenderContext::new(window).await?;

        let streaming_system = MeshStreamingSystem::new(
            render_context.allocator().clone(),
            render_context.command_buffer_allocator().clone(),
            render_context.graphics_queue.clone(),
        );

        // Create mesh database with various meshes
        let mut mesh_database = HashMap::new();

        // Create different mesh types
        let cube_mesh = colored_cube_mesh();
        let sphere_mesh_data = sphere_mesh(1.0, 16, 16, [0.7, 0.7, 0.9]);

        // Create state indicator meshes (small cubes above each object) - load immediately
        let unloaded_indicator = solid_cube_mesh([0.5, 0.5, 0.5]); // Gray
        let queued_indicator = solid_cube_mesh([1.0, 1.0, 0.0]); // Yellow
        let loading_indicator = solid_cube_mesh([0.0, 1.0, 1.0]); // Cyan
        let loaded_indicator = solid_cube_mesh([0.0, 1.0, 0.0]); // Green
        let failed_indicator = solid_cube_mesh([1.0, 0.0, 0.0]); // Red

        render_context
            .mesh_manager_mut()
            .load_mesh("indicator_unloaded", unloaded_indicator)?;
        render_context
            .mesh_manager_mut()
            .load_mesh("indicator_queued", queued_indicator)?;
        render_context
            .mesh_manager_mut()
            .load_mesh("indicator_loading", loading_indicator)?;
        render_context
            .mesh_manager_mut()
            .load_mesh("indicator_loaded", loaded_indicator)?;
        render_context
            .mesh_manager_mut()
            .load_mesh("indicator_failed", failed_indicator)?;

        // Create scene objects in a grid pattern
        let mut scene_objects = Vec::new();
        const GRID_SIZE: i32 = 10;
        const SPACING: f32 = 10.0;
        let mut object_count = 0;

        for x in -GRID_SIZE..=GRID_SIZE {
            for z in -GRID_SIZE..=GRID_SIZE {
                // Vary height for visual interest
                let y = ((x as f32 * 0.3).sin() + (z as f32 * 0.3).cos()) * 3.0;
                let position = Vec3::new(x as f32 * SPACING, y, z as f32 * SPACING);

                // Alternate between cube and sphere
                let mesh_type = if (x + z) % 2 == 0 {
                    MeshType::Cube
                } else {
                    MeshType::Sphere
                };

                let mesh_id = format!("mesh_{}_{}", x, z);

                // Register mesh in database
                mesh_database.insert(
                    mesh_id.clone(),
                    match mesh_type {
                        MeshType::Cube => cube_mesh.clone(),
                        MeshType::Sphere => sphere_mesh_data.clone(),
                    },
                );

                scene_objects.push(SceneObject {
                    position,
                    mesh_id,
                    mesh_type,
                });

                object_count += 1;
            }
        }

        info!(
            "Created {} scene objects in a {}x{} grid",
            object_count,
            GRID_SIZE * 2 + 1,
            GRID_SIZE * 2 + 1
        );

        Ok(Self {
            render_context,
            streaming_system,
            mesh_database,
            scene_objects,
            camera: CameraController::default(),
            stats_timer: 0.0,
            frame_count: 0,
        })
    }

    fn register_meshes(&mut self) -> Result<()> {
        info!(
            "Registering {} meshes for streaming",
            self.mesh_database.len()
        );

        for (mesh_id, mesh_data) in &self.mesh_database {
            self.streaming_system
                .register_mesh(mesh_id, mesh_data.clone())?;
        }

        Ok(())
    }

    fn update(&mut self, delta_time: f32) -> Result<()> {
        // Update camera
        self.camera.update(delta_time);

        // Update streaming system (process completed loads)
        self.streaming_system.update();

        // Setup camera matrices for frustum culling
        let view = self.camera.get_view_matrix();
        let aspect_ratio = 1280.0 / 720.0;
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), aspect_ratio, 0.1, 1000.0);
        let view_proj = proj * view;

        let frustum = Frustum::from_view_projection(view_proj);

        // Update priorities for all objects based on visibility and distance
        for object in &self.scene_objects {
            let is_visible = |center: Vec3, radius: f32| -> bool {
                // Check if the bounding sphere is in the frustum
                frustum.contains_point(center)
                    || (center - self.camera.position).length() < radius * 3.0
            };

            self.streaming_system.update_priorities(
                is_visible,
                self.camera.position,
                object.position,
            );
        }

        // Trigger loading for visible meshes
        let mesh_database = &self.mesh_database;
        self.streaming_system
            .load_visible_meshes(&|id: &str| mesh_database.get(id).cloned());

        // Sync loaded meshes to render context
        self.sync_loaded_meshes()?;

        // Update stats
        self.stats_timer += delta_time;
        self.frame_count += 1;

        if self.stats_timer >= 1.0 {
            let fps = self.frame_count as f32 / self.stats_timer;
            let loaded = self.streaming_system.loaded_count();
            let total = self.streaming_system.total_count();

            // Count meshes by state
            let mut state_counts = [0; 5]; // unloaded, queued, loading, loaded, failed
            for object in &self.scene_objects {
                if let Some(state) = self.streaming_system.get_mesh_state(&object.mesh_id) {
                    state_counts[state as usize] += 1;
                }
            }

            info!(
                "FPS: {:.1} | Loaded: {}/{} ({:.0}%) | States [U:{} Q:{} L:{} OK:{} F:{}] | Cam: ({:.0},{:.0},{:.0})",
                fps,
                loaded,
                total,
                (loaded as f32 / total as f32) * 100.0,
                state_counts[0], // Unloaded
                state_counts[1], // Queued
                state_counts[2], // Loading
                state_counts[3], // Loaded
                state_counts[4], // Failed
                self.camera.position.x,
                self.camera.position.y,
                self.camera.position.z
            );

            self.stats_timer = 0.0;
            self.frame_count = 0;
        }

        Ok(())
    }

    fn sync_loaded_meshes(&mut self) -> Result<()> {
        // Transfer newly loaded meshes from streaming system to render context
        for object in &self.scene_objects {
            if self.streaming_system.is_mesh_loaded(&object.mesh_id) {
                if !self
                    .render_context
                    .mesh_manager()
                    .contains_mesh(&object.mesh_id)
                {
                    if let Some(mesh_data) = self.mesh_database.get(&object.mesh_id) {
                        self.render_context
                            .mesh_manager_mut()
                            .load_mesh(&object.mesh_id, mesh_data.clone())?;
                        trace!(
                            "Synced streamed mesh '{}' to render context",
                            object.mesh_id
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        let view = self.camera.get_view_matrix();
        let aspect_ratio = 1280.0 / 720.0;
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), aspect_ratio, 0.1, 1000.0);

        let mut draw_commands = Vec::new();

        // Render scene objects with their loading state indicators
        for object in &self.scene_objects {
            // Render the actual mesh if loaded and available in render context
            if self.streaming_system.is_mesh_loaded(&object.mesh_id)
                && self
                    .render_context
                    .mesh_manager()
                    .contains_mesh(&object.mesh_id)
            {
                let model = Mat4::from_translation(object.position);
                draw_commands.push(DrawCommand {
                    mesh_id: object.mesh_id.clone(),
                    model,
                    texture_name: None,
                    material_properties: None,
                    material_instance_id: None,
                    bone_matrices: None,
                });
            }

            // Render state indicator above the object
            let indicator_mesh = self.get_state_indicator(&object.mesh_id);
            let indicator_offset = Vec3::new(0.0, 3.0, 0.0);
            let indicator_scale = 0.3;
            let indicator_model = Mat4::from_scale_rotation_translation(
                Vec3::splat(indicator_scale),
                Quat::IDENTITY,
                object.position + indicator_offset,
            );

            draw_commands.push(DrawCommand {
                mesh_id: indicator_mesh.to_string(),
                model: indicator_model,
                texture_name: None,
                material_properties: None,
                material_instance_id: None,
                bone_matrices: None,
            });
        }

        let render_commands = RenderCommands {
            view,
            proj,
            draw_commands: &draw_commands,
            lighting: None,
        };

        self.render_context.render(&render_commands)?;

        Ok(())
    }

    fn get_state_indicator(&self, mesh_id: &str) -> &str {
        // Get the streaming state for this mesh
        if let Some(state) = self.streaming_system.get_mesh_state(mesh_id) {
            match state {
                MeshStreamingState::Unloaded => "indicator_unloaded",
                MeshStreamingState::Queued => "indicator_queued",
                MeshStreamingState::Loading => "indicator_loading",
                MeshStreamingState::Loaded => "indicator_loaded",
                MeshStreamingState::Failed => "indicator_failed",
            }
        } else {
            "indicator_unloaded"
        }
    }
}

#[pollster::main]
async fn main() -> Result<()> {
    praxis::praxis_utils::setup_logging();

    info!("=== Mesh Streaming Demo ===");
    info!("");
    info!("This demo demonstrates async mesh streaming with priority-based loading.");
    info!("Move the camera around to see meshes stream in based on visibility and distance.");
    info!("");
    info!("Loading State Indicators (colored cubes above objects):");
    info!("  GRAY   - Unloaded (outside frustum, low priority)");
    info!("  YELLOW - Queued for loading (in frustum, waiting)");
    info!("  CYAN   - Currently loading (background thread)");
    info!("  GREEN  - Loaded and ready (fully streamed)");
    info!("  RED    - Failed to load (error state)");
    info!("");
    info!("Controls:");
    info!("  W/A/S/D    - Move camera (forward/left/back/right)");
    info!("  Q/E        - Move camera down/up");
    info!("  Arrow Keys - Rotate camera");
    info!("  Space      - Reset camera to default position");
    info!("  R          - Toggle rapid movement (stress test)");
    info!("  ESC        - Exit");
    info!("");

    let event_loop = EventLoop::new().map_err(|e| praxis::praxis_utils::eyre::eyre!("{}", e))?;
    let window = WindowManager::create_window(&event_loop, "Mesh Streaming Demo", 1280, 720)?;

    let mut demo = MeshStreamingDemo::new(window.clone()).await?;
    demo.register_meshes()?;

    info!(
        "Mesh streaming demo initialized - {} objects ready",
        demo.scene_objects.len()
    );

    let mut last_frame = std::time::Instant::now();

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    info!("Close requested, shutting down");
                    elwt.exit();
                }
                Event::WindowEvent { ref event, .. } => {
                    demo.camera.handle_input(event);

                    if let WindowEvent::Resized(_) = event {
                        demo.render_context.handle_resize();
                    }
                }
                Event::AboutToWait => {
                    let now = std::time::Instant::now();
                    let delta_time = (now - last_frame).as_secs_f32().min(0.1);
                    last_frame = now;

                    if let Err(e) = demo.update(delta_time) {
                        error!("Update error: {}", e);
                        elwt.exit();
                    }

                    if let Err(e) = demo.render() {
                        error!("Render error: {}", e);
                        elwt.exit();
                    }
                }
                _ => {}
            }
        })
        .map_err(|e| praxis::praxis_utils::eyre::eyre!("{}", e))?;

    Ok(())
}
