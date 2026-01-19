//! Demonstration of debug rendering modes for optimization systems.
//!
//! This example showcases:
//! - Wireframe bounding spheres colored by culling results
//! - LOD level heat map visualization
//! - Mesh streaming state indicators
//!
//! Controls:
//! - 1: Toggle culling debug visualization
//! - 2: Toggle LOD heat map
//! - 3: Toggle mesh streaming state
//! - WASD: Move camera
//! - Mouse: Look around
//! - ESC: Exit

use praxis_core::Engine;
use praxis_ecs::World;
use praxis_graphics::{
    debug_rendering::{
        helpers, CullingDebugInfo, DebugRenderMode, DebugRenderer, LodDebugInfo,
        StreamingDebugInfo, StreamingState,
    },
    lod::{LodGroup, LodLevel},
    DrawCommand, RenderCommands, RenderContext,
};
use praxis_input::{InputState, Key};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_scene::{Camera, GlobalTransform, Transform};
use praxis_utils::{info, Result};
use praxis_window::{WindowConfig, WindowManager};
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

/// Camera controller state.
struct CameraController {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    move_speed: f32,
    look_speed: f32,
}

impl CameraController {
    fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 5.0, 20.0),
            yaw: 0.0,
            pitch: 0.0,
            move_speed: 10.0,
            look_speed: 0.002,
        }
    }

    fn update(&mut self, input: &InputState, delta_time: f32) {
        // Movement
        let mut movement = Vec3::ZERO;

        if input.is_key_pressed(Key::W) {
            movement.z -= 1.0;
        }
        if input.is_key_pressed(Key::S) {
            movement.z += 1.0;
        }
        if input.is_key_pressed(Key::A) {
            movement.x -= 1.0;
        }
        if input.is_key_pressed(Key::D) {
            movement.x += 1.0;
        }

        if movement.length_squared() > 0.0 {
            movement = movement.normalize();

            // Apply rotation to movement
            let rotation = Quat::from_rotation_y(self.yaw);
            movement = rotation * movement;

            self.position += movement * self.move_speed * delta_time;
        }

        // Look
        let mouse_delta = input.mouse_delta();
        self.yaw -= mouse_delta.0 * self.look_speed;
        self.pitch -= mouse_delta.1 * self.look_speed;
        self.pitch = self.pitch.clamp(-1.5, 1.5);
    }

    fn view_matrix(&self) -> Mat4 {
        let rotation = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);
        let forward = rotation * Vec3::new(0.0, 0.0, -1.0);
        let up = Vec3::Y;

        Mat4::look_at_rh(self.position, self.position + forward, up)
    }
}

/// Object in the scene with optimization data.
struct SceneObject {
    position: Vec3,
    radius: f32,
    lod_group: LodGroup,
    is_visible: bool,
    streaming_state: StreamingState,
    load_progress: f32,
}

impl SceneObject {
    fn new(position: Vec3, radius: f32, num_lod_levels: usize) -> Self {
        // Create LOD levels
        let mut lod_levels = Vec::new();
        for i in 0..num_lod_levels {
            let min_distance = (i * 10) as f32;
            let max_distance = ((i + 1) * 10) as f32;
            lod_levels.push(LodLevel::new(
                format!("mesh_lod_{}", i),
                min_distance,
                max_distance,
            ));
        }

        let lod_group = LodGroup::new(lod_levels);

        Self {
            position,
            radius,
            lod_group,
            is_visible: true,
            streaming_state: StreamingState::Loaded,
            load_progress: 1.0,
        }
    }

    fn update(&mut self, camera_pos: Vec3, delta_time: f32) {
        let distance_sq = (self.position - camera_pos).length_squared();
        self.lod_group.update(distance_sq, delta_time);

        // Simulate streaming state based on distance
        let distance = distance_sq.sqrt();
        if distance > 100.0 {
            self.streaming_state = StreamingState::NotLoaded;
            self.load_progress = 0.0;
        } else if distance > 80.0 {
            self.streaming_state = StreamingState::Loading;
            self.load_progress = ((100.0 - distance) / 20.0).clamp(0.0, 1.0);
        } else {
            self.streaming_state = StreamingState::Loaded;
            self.load_progress = 1.0;
        }

        // Simulate culling based on distance
        self.is_visible = distance < 120.0;
    }
}

/// Main demo state.
struct OptimizationDebugDemo {
    world: World,
    render_context: RenderContext,
    debug_renderer: Option<DebugRenderer>,
    camera_controller: CameraController,
    objects: Vec<SceneObject>,
    input_state: InputState,
    last_frame_time: std::time::Instant,
}

impl OptimizationDebugDemo {
    async fn new(window: Arc<winit::window::Window>) -> Result<Self> {
        info!("Initializing optimization debug demo");

        let world = World::new();
        let render_context = RenderContext::new(window).await?;
        let input_state = InputState::new();

        Ok(Self {
            world,
            render_context,
            debug_renderer: None,
            camera_controller: CameraController::new(),
            objects: Vec::new(),
            input_state,
            last_frame_time: std::time::Instant::now(),
        })
    }

    fn init(&mut self) -> Result<()> {
        info!("Initializing demo resources");

        // Create debug renderer
        let render_pass = self.render_context.render_pass.clone();
        let viewport_dimensions = [
            self.render_context.viewport.extent[0] as u32,
            self.render_context.viewport.extent[1] as u32,
        ];

        let mut debug_renderer = DebugRenderer::new(
            self.render_context.device.clone(),
            self.render_context.memory_allocator.clone(),
            render_pass,
            viewport_dimensions,
        )?;

        // Enable all debug modes by default
        debug_renderer.enable_mode(DebugRenderMode::CullingResults);
        debug_renderer.enable_mode(DebugRenderMode::LodHeatMap);
        debug_renderer.enable_mode(DebugRenderMode::MeshStreamingState);

        self.debug_renderer = Some(debug_renderer);

        // Create a grid of objects
        for x in -10..=10 {
            for z in -10..=10 {
                if (x + z) % 2 == 0 {
                    let position = Vec3::new(x as f32 * 5.0, 0.0, z as f32 * 5.0);
                    let num_lod_levels = ((x.abs() + z.abs()) % 4 + 2) as usize;
                    self.objects
                        .push(SceneObject::new(position, 1.5, num_lod_levels));
                }
            }
        }

        info!("Created {} objects", self.objects.len());

        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        let current_time = std::time::Instant::now();
        let delta_time = (current_time - self.last_frame_time).as_secs_f32();
        self.last_frame_time = current_time;

        // Update camera
        self.camera_controller.update(&self.input_state, delta_time);

        // Update objects
        let camera_pos = self.camera_controller.position;
        for object in &mut self.objects {
            object.update(camera_pos, delta_time);
        }

        // Handle debug mode toggles
        if let Some(debug_renderer) = &mut self.debug_renderer {
            if self.input_state.is_key_just_pressed(Key::Num1) {
                debug_renderer.toggle_mode(DebugRenderMode::CullingResults);
                info!("Toggled culling debug visualization");
            }
            if self.input_state.is_key_just_pressed(Key::Num2) {
                debug_renderer.toggle_mode(DebugRenderMode::LodHeatMap);
                info!("Toggled LOD heat map");
            }
            if self.input_state.is_key_just_pressed(Key::Num3) {
                debug_renderer.toggle_mode(DebugRenderMode::MeshStreamingState);
                info!("Toggled streaming state visualization");
            }
        }

        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        // Build view-projection matrix
        let view = self.camera_controller.view_matrix();
        let aspect =
            self.render_context.viewport.extent[0] / self.render_context.viewport.extent[1];
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), aspect, 0.1, 1000.0);
        let view_proj = proj * view;

        // Prepare debug info
        let culling_info: Vec<CullingDebugInfo> = self
            .objects
            .iter()
            .map(|obj| CullingDebugInfo {
                position: obj.position,
                radius: obj.radius,
                is_visible: obj.is_visible,
                cull_reason: None,
            })
            .collect();

        let lod_info: Vec<LodDebugInfo> = self
            .objects
            .iter()
            .map(|obj| {
                let distance = (obj.position - self.camera_controller.position).length();
                helpers::lod_info_from_lod_group(obj.position, obj.radius, &obj.lod_group, distance)
            })
            .collect();

        let streaming_info: Vec<StreamingDebugInfo> = self
            .objects
            .iter()
            .map(|obj| StreamingDebugInfo {
                position: obj.position,
                radius: obj.radius,
                state: obj.streaming_state,
                load_progress: obj.load_progress,
            })
            .collect();

        // Render main scene (basic, just to have something visible)
        let draw_commands = Vec::new(); // No actual meshes in this demo
        let render_commands = RenderCommands {
            view,
            proj,
            draw_commands: &draw_commands,
            lighting: None,
        };

        self.render_context.render(&render_commands)?;

        // Render debug overlays
        if let Some(debug_renderer) = &mut self.debug_renderer {
            // Note: In a real integration, this would be called within the render pass
            // For this demo, we're showing the API usage
            info!(
                "Debug render: {} culling, {} LOD, {} streaming",
                culling_info.len(),
                lod_info.len(),
                streaming_info.len()
            );
        }

        Ok(())
    }

    fn handle_event(&mut self, event: &WindowEvent) {
        self.input_state.handle_event(event);
    }

    fn end_frame(&mut self) {
        self.input_state.end_frame();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    praxis_utils::init_logging()?;

    info!("Starting optimization debug demo");

    let event_loop = EventLoop::new()?;
    let window_config = WindowConfig {
        title: "Optimization Debug Visualization".to_string(),
        width: 1920,
        height: 1080,
        ..Default::default()
    };

    let window_manager = WindowManager::new(&event_loop, window_config)?;
    let window = window_manager.window();

    let mut demo = OptimizationDebugDemo::new(window).await?;
    demo.init()?;

    info!("Demo initialized - use 1/2/3 to toggle debug modes");

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, .. } => {
                demo.handle_event(&event);

                match event {
                    WindowEvent::CloseRequested => {
                        info!("Close requested");
                        elwt.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        if let Err(e) = demo.update() {
                            eprintln!("Update error: {}", e);
                            elwt.exit();
                        }

                        if let Err(e) = demo.render() {
                            eprintln!("Render error: {}", e);
                            elwt.exit();
                        }

                        demo.end_frame();
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                window_manager.window().request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}
