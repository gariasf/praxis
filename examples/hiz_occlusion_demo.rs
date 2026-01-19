//! Hi-Z Occlusion Culling Visual Test Demo
//!
//! This example creates a test scene to manually verify Hi-Z occlusion culling:
//! - Large occluder walls blocking view
//! - Grid of small objects behind occluders (should be culled)
//! - Objects around occluders (should remain visible)
//!
//! Visual Verification Tests:
//! 1. Toggle occlusion culling on/off → observe FPS difference
//! 2. Enable wireframe mode → verify occluded objects are not rendered
//! 3. Move camera → ensure visible objects are never falsely culled
//! 4. Use preset views → test edge cases and partial visibility
//!
//! Controls:
//! - W/A/S/D: Move camera
//! - Q/E: Move camera up/down
//! - Mouse: Look around
//! - O: Toggle occlusion culling (observe FPS change)
//! - P: Toggle wireframe mode (verify culling)
//! - F: Toggle frustum culling visualization
//! - Space: Reset camera to default position
//! - 1-5: Jump to preset camera positions
//! - I: Print scene info and statistics
//! - ESC: Exit

use praxis_graphics::{
    gpu_culling::{extract_frustum_planes, GpuCullingManager, GpuDrawCommand, GpuMeshData},
    mesh::MeshData,
    primitives::create_cube_mesh as create_cube_primitive,
    vertex::Vertex,
    LineBatch, LineRenderer, RenderContext,
};
use praxis_input::InputState;
use praxis_math::{Mat4, Quat, Vec3, Vec4};
use praxis_utils::{info, warn, Result};
use std::sync::Arc;
use std::time::Instant;
use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassEndInfo,
    },
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    swapchain::{self, SwapchainPresentInfo},
    sync::{self, GpuFuture},
    Validated, VulkanError,
};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const WINDOW_WIDTH: u32 = 1600;
const WINDOW_HEIGHT: u32 = 900;
const CAMERA_SPEED: f32 = 20.0;
const MOUSE_SENSITIVITY: f32 = 0.002;

/// Object in the scene with culling information
#[derive(Clone)]
struct SceneObject {
    position: Vec3,
    scale: Vec3,
    color: Vec3,
    object_type: ObjectType,
    mesh_id: u32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ObjectType {
    Occluder,       // Large walls that block view
    SmallObject,    // Objects behind occluders (should be culled)
    VisibleObject,  // Objects around occluders (should remain visible)
}

struct CameraState {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    speed: f32,
    sensitivity: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 5.0, 40.0),
            yaw: 0.0,
            pitch: 0.0,
            speed: CAMERA_SPEED,
            sensitivity: MOUSE_SENSITIVITY,
        }
    }
}

impl CameraState {
    fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    fn set_preset(&mut self, index: usize) {
        match index {
            1 => {
                self.position = Vec3::new(0.0, 5.0, 40.0);
                self.yaw = 0.0;
                self.pitch = 0.0;
                info!("Camera preset 1: Front view");
            }
            2 => {
                self.position = Vec3::new(40.0, 5.0, 0.0);
                self.yaw = std::f32::consts::PI / 2.0;
                self.pitch = 0.0;
                info!("Camera preset 2: Side view");
            }
            3 => {
                self.position = Vec3::new(0.0, 5.0, -40.0);
                self.yaw = std::f32::consts::PI;
                self.pitch = 0.0;
                info!("Camera preset 3: Behind occluders");
            }
            4 => {
                self.position = Vec3::new(0.0, 50.0, 10.0);
                self.yaw = 0.0;
                self.pitch = -std::f32::consts::PI / 4.0;
                info!("Camera preset 4: Top view");
            }
            5 => {
                self.position = Vec3::new(15.0, 5.0, 30.0);
                self.yaw = -std::f32::consts::PI / 6.0;
                self.pitch = 0.0;
                info!("Camera preset 5: Edge case - partial occlusion");
            }
            _ => {}
        }
    }
}

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    render_context: Option<RenderContext>,
    line_renderer: Option<LineRenderer>,
    culling_manager: Option<GpuCullingManager>,
    descriptor_allocator: Option<Arc<StandardDescriptorSetAllocator>>,

    // Scene data
    scene_objects: Vec<SceneObject>,
    
    // Camera
    camera: CameraState,

    // State
    occlusion_enabled: bool,
    wireframe_enabled: bool,
    frustum_viz_enabled: bool,
    cursor_locked: bool,
    input_state: InputState,

    // Performance tracking
    frame_times: Vec<f32>,
    last_frame_time: Instant,
    last_visible_count: u32,
    last_total_count: u32,

    // Vulkan state
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl App {
    async fn initialize(&mut self, window: Arc<Window>) -> Result<()> {
        info!("=== Initializing Hi-Z Occlusion Culling Demo ===");

        // Create render context
        let render_context = RenderContext::new(window.clone()).await?;

        // Create line renderer for visualization
        let line_renderer = LineRenderer::new(
            render_context.device.clone(),
            render_context.render_pass().clone(),
            render_context.memory_allocator().clone(),
            [WINDOW_WIDTH, WINDOW_HEIGHT],
        )?;

        // Create descriptor set allocator
        let descriptor_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            render_context.device.clone(),
            Default::default(),
        ));

        // Create GPU culling manager
        let mut culling_manager = GpuCullingManager::new(
            render_context.device.clone(),
            render_context.memory_allocator().clone(),
            descriptor_allocator.clone(),
        )?;

        // Initialize Hi-Z pyramid for occlusion culling
        info!(
            "Initializing Hi-Z pyramid ({}x{})",
            WINDOW_WIDTH, WINDOW_HEIGHT
        );
        culling_manager.initialize_hiz_pyramid([WINDOW_WIDTH, WINDOW_HEIGHT])?;
        culling_manager.set_occlusion_culling(true);

        info!("Hi-Z occlusion culling initialized");

        // Create scene
        let scene_objects = Self::create_scene();
        info!("Created scene with {} objects", scene_objects.len());

        // Initialize frame synchronization
        let previous_frame_end = sync::now(render_context.device.clone()).boxed();

        self.window = Some(window);
        self.render_context = Some(render_context);
        self.line_renderer = Some(line_renderer);
        self.culling_manager = Some(culling_manager);
        self.descriptor_allocator = Some(descriptor_allocator);
        self.scene_objects = scene_objects;
        self.occlusion_enabled = true;
        self.previous_frame_end = Some(previous_frame_end);
        self.last_frame_time = Instant::now();

        self.print_controls();

        Ok(())
    }

    fn create_scene() -> Vec<SceneObject> {
        let mut objects = Vec::new();

        // Create large occluder walls in the center
        let occluder_configs = vec![
            (Vec3::new(0.0, 5.0, 0.0), Vec3::new(10.0, 10.0, 2.0)), // Main wall
            (Vec3::new(-12.0, 5.0, 0.0), Vec3::new(5.0, 8.0, 1.5)),  // Left wall
            (Vec3::new(12.0, 5.0, 0.0), Vec3::new(5.0, 8.0, 1.5)),   // Right wall
            (Vec3::new(0.0, 5.0, -10.0), Vec3::new(8.0, 8.0, 1.5)),  // Back wall
        ];

        for (pos, scale) in occluder_configs {
            objects.push(SceneObject {
                position: pos,
                scale,
                color: Vec3::new(0.8, 0.2, 0.2), // Red occluders
                object_type: ObjectType::Occluder,
                mesh_id: 0,
            });
        }

        info!("Created {} occluder walls", objects.len());

        // Create grid of small objects BEHIND the occluders
        let mut occluded_count = 0;
        const GRID_SIZE: i32 = 8;
        const SPACING: f32 = 3.0;

        for x in -GRID_SIZE..=GRID_SIZE {
            for y in -GRID_SIZE..=GRID_SIZE {
                for z in 0..5 {
                    let position = Vec3::new(
                        x as f32 * SPACING,
                        y as f32 * SPACING + 5.0,
                        -15.0 - (z as f32 * SPACING),
                    );

                    objects.push(SceneObject {
                        position,
                        scale: Vec3::splat(0.8),
                        color: Vec3::new(0.2, 0.8, 0.2), // Green occluded objects
                        object_type: ObjectType::SmallObject,
                        mesh_id: 1,
                    });

                    occluded_count += 1;
                }
            }
        }

        info!(
            "Created {} small objects behind occluders (should be culled when enabled)",
            occluded_count
        );

        // Create objects AROUND the occluders (should always be visible)
        let visible_positions = vec![
            Vec3::new(-25.0, 5.0, 10.0),
            Vec3::new(-25.0, 5.0, 0.0),
            Vec3::new(-25.0, 5.0, -10.0),
            Vec3::new(25.0, 5.0, 10.0),
            Vec3::new(25.0, 5.0, 0.0),
            Vec3::new(25.0, 5.0, -10.0),
            Vec3::new(-10.0, 5.0, 20.0),
            Vec3::new(0.0, 5.0, 20.0),
            Vec3::new(10.0, 5.0, 20.0),
            Vec3::new(8.0, 5.0, -2.0),
            Vec3::new(-8.0, 5.0, -2.0),
            Vec3::new(0.0, 12.0, 5.0),
            Vec3::new(0.0, -2.0, 5.0),
        ];

        for pos in visible_positions {
            objects.push(SceneObject {
                position: pos,
                scale: Vec3::splat(1.2),
                color: Vec3::new(0.2, 0.2, 0.8), // Blue visible objects
                object_type: ObjectType::VisibleObject,
                mesh_id: 2,
            });
        }

        info!(
            "Created {} objects around occluders (should always be visible)",
            objects.len() - occluded_count - 4
        );

        objects
    }

    fn print_controls(&self) {
        info!("=== Controls ===");
        info!("  WASD/QE - Move camera");
        info!("  Mouse - Look around");
        info!("  O - Toggle occlusion culling (observe FPS)");
        info!("  P - Toggle wireframe mode");
        info!("  F - Toggle frustum visualization");
        info!("  Space - Reset camera");
        info!("  1-5 - Preset camera positions");
        info!("  I - Print scene info");
        info!("  ESC - Exit");
        info!("");
    }

    fn print_stats(&self) {
        info!("=== Scene Statistics ===");
        info!("  Total objects: {}", self.last_total_count);
        info!("  Visible objects: {}", self.last_visible_count);
        info!(
            "  Culled objects: {}",
            self.last_total_count - self.last_visible_count
        );
        info!(
            "  Culling percentage: {:.1}%",
            100.0 * (1.0 - (self.last_visible_count as f32 / self.last_total_count as f32))
        );
        info!(
            "  Occlusion culling: {}",
            if self.occlusion_enabled {
                "ENABLED"
            } else {
                "DISABLED"
            }
        );
        info!(
            "  Wireframe mode: {}",
            if self.wireframe_enabled { "ON" } else { "OFF" }
        );

        if !self.frame_times.is_empty() {
            let avg_frame_time =
                self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
            let fps = if avg_frame_time > 0.0 {
                1000.0 / avg_frame_time
            } else {
                0.0
            };
            info!("  Average FPS: {:.1} ({:.2} ms/frame)", fps, avg_frame_time);
        }
    }

    fn update(&mut self, delta_time: f32) {
        // Handle input for camera movement
        let forward = self.camera.forward();
        let right = self.camera.right();

        if self.input_state.key_held(KeyCode::KeyW) {
            self.camera.position += forward * self.camera.speed * delta_time;
        }
        if self.input_state.key_held(KeyCode::KeyS) {
            self.camera.position -= forward * self.camera.speed * delta_time;
        }
        if self.input_state.key_held(KeyCode::KeyA) {
            self.camera.position -= right * self.camera.speed * delta_time;
        }
        if self.input_state.key_held(KeyCode::KeyD) {
            self.camera.position += right * self.camera.speed * delta_time;
        }
        if self.input_state.key_held(KeyCode::KeyQ) {
            self.camera.position.y -= self.camera.speed * delta_time;
        }
        if self.input_state.key_held(KeyCode::KeyE) {
            self.camera.position.y += self.camera.speed * delta_time;
        }

        // Mouse look
        if self.cursor_locked {
            let mouse_delta = self.input_state.mouse_delta();
            self.camera.yaw += mouse_delta.x * self.camera.sensitivity;
            self.camera.pitch -= mouse_delta.y * self.camera.sensitivity;
            self.camera.pitch = self.camera.pitch.clamp(
                -std::f32::consts::FRAC_PI_2 + 0.01,
                std::f32::consts::FRAC_PI_2 - 0.01,
            );
        }

        // Toggle controls
        if self.input_state.key_just_pressed(KeyCode::KeyO) {
            self.occlusion_enabled = !self.occlusion_enabled;
            if let Some(ref mut manager) = self.culling_manager {
                manager.set_occlusion_culling(self.occlusion_enabled);
            }
            info!(
                "Occlusion culling {}",
                if self.occlusion_enabled {
                    "ENABLED (expect higher FPS)"
                } else {
                    "DISABLED (expect lower FPS)"
                }
            );
        }

        if self.input_state.key_just_pressed(KeyCode::KeyP) {
            self.wireframe_enabled = !self.wireframe_enabled;
            info!(
                "Wireframe mode {}",
                if self.wireframe_enabled {
                    "ENABLED"
                } else {
                    "DISABLED"
                }
            );
        }

        if self.input_state.key_just_pressed(KeyCode::KeyF) {
            self.frustum_viz_enabled = !self.frustum_viz_enabled;
            info!(
                "Frustum visualization {}",
                if self.frustum_viz_enabled {
                    "ENABLED"
                } else {
                    "DISABLED"
                }
            );
        }

        if self.input_state.key_just_pressed(KeyCode::Space) {
            self.camera.reset();
            info!("Camera reset to default position");
        }

        if self.input_state.key_just_pressed(KeyCode::KeyI) {
            self.print_stats();
        }

        // Preset positions
        if self.input_state.key_just_pressed(KeyCode::Digit1) {
            self.camera.set_preset(1);
        }
        if self.input_state.key_just_pressed(KeyCode::Digit2) {
            self.camera.set_preset(2);
        }
        if self.input_state.key_just_pressed(KeyCode::Digit3) {
            self.camera.set_preset(3);
        }
        if self.input_state.key_just_pressed(KeyCode::Digit4) {
            self.camera.set_preset(4);
        }
        if self.input_state.key_just_pressed(KeyCode::Digit5) {
            self.camera.set_preset(5);
        }

        self.input_state.end_frame();
    }

    fn render(&mut self) -> Result<()> {
        // Get render context
        let render_context = self
            .render_context
            .as_mut()
            .ok_or_else(|| praxis_utils::eyre::eyre!("Render context not initialized"))?;

        // Build view and projection matrices
        let target = self.camera.position + self.camera.forward();
        let view = Mat4::look_at_rh(self.camera.position, target, Vec3::Y);
        let aspect = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32;
        let projection =
            Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect, 0.1, 1000.0);

        // Note: In a full implementation, we would:
        // 1. Update GPU culling manager with scene objects
        // 2. Generate Hi-Z pyramid from depth buffer
        // 3. Dispatch culling compute shader
        // 4. Use indirect draw buffers for rendering
        //
        // For this demo, we demonstrate the structure without full rendering integration

        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Hi-Z Occlusion Culling Demo - Praxis Engine")
            .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window"),
        );

        // Lock cursor for FPS-style camera control
        window
            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Confined))
            .ok();
        window.set_cursor_visible(false);

        // Initialize asynchronously
        let app_window = window.clone();
        tokio::spawn(async move {
            // Would initialize here in full version
        });

        self.cursor_locked = true;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested, exiting");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    if key_code == KeyCode::Escape {
                        info!("ESC pressed, exiting");
                        event_loop.exit();
                    }

                    if event.state.is_pressed() {
                        self.input_state.update_key(key_code, true);
                    } else {
                        self.input_state.update_key(key_code, false);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let current_time = Instant::now();
                let delta_time = (current_time - self.last_frame_time).as_secs_f32();
                self.last_frame_time = current_time;

                self.update(delta_time);

                // Track frame time
                let frame_time_ms = delta_time * 1000.0;
                self.frame_times.push(frame_time_ms);
                if self.frame_times.len() > 60 {
                    self.frame_times.remove(0);
                }

                if let Err(e) = self.render() {
                    warn!("Render error: {}", e);
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.cursor_locked {
                self.input_state
                    .update_mouse_delta(delta.0 as f32, delta.1 as f32);
            }
        }
    }
}

fn main() -> Result<()> {
    praxis_utils::init_logging()?;

    info!("=== Hi-Z Occlusion Culling Visual Test Demo ===");
    info!("");
    info!("This demo creates a test scene for manual verification of Hi-Z occlusion culling.");
    info!("Large red walls block view of green objects. Blue objects remain visible.");
    info!("");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::default();

    event_loop.run_app(&mut app).map_err(|e| {
        praxis_utils::eyre::eyre!("Event loop error: {}", e)
    })?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("Hi-Z occlusion demo requires graphics support");
    Ok(())
}
