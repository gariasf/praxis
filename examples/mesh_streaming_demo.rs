//! Demonstrates async mesh streaming with background loading and frustum culling.
//!
//! This example shows:
//! - Background thread mesh loading with priority queue
//! - Frustum-based on-demand mesh loading
//! - Loading state visualization
//! - Priority-based loading based on distance and visibility

use praxis::praxis_ecs::{Camera, CameraMatrices, PerspectiveProjection, Transform};
use praxis::praxis_graphics::{
    colored_cube_mesh, MeshData, MeshStreamingState, MeshStreamingSystem, RenderContext,
};
use praxis::praxis_math::{Mat4, Vec3};
use praxis::praxis_spatial::Frustum;
use praxis::praxis_utils::{info, trace, Result};
use praxis::praxis_window::WindowManager;
use std::collections::HashMap;
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

struct MeshStreamingDemo {
    render_context: RenderContext,
    streaming_system: MeshStreamingSystem,
    mesh_database: HashMap<String, MeshData>,
    camera_position: Vec3,
    camera_rotation: f32,
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
        
        // Register multiple meshes for streaming
        for i in 0..20 {
            let mesh_id = format!("cube_{}", i);
            let mesh_data = colored_cube_mesh();
            mesh_database.insert(mesh_id, mesh_data);
        }

        let camera_position = Vec3::new(0.0, 5.0, 20.0);
        let camera_rotation = 0.0;

        Ok(Self {
            render_context,
            streaming_system,
            mesh_database,
            camera_position,
            camera_rotation,
        })
    }

    fn register_meshes(&mut self) -> Result<()> {
        info!("Registering {} meshes for streaming", self.mesh_database.len());

        for (mesh_id, mesh_data) in &self.mesh_database {
            self.streaming_system
                .register_mesh(mesh_id, mesh_data.clone())?;
        }

        Ok(())
    }

    fn update(&mut self, delta_time: f32) -> Result<()> {
        // Update camera rotation
        self.camera_rotation += delta_time * 0.5;
        self.camera_position.x = self.camera_rotation.cos() * 20.0;
        self.camera_position.z = self.camera_rotation.sin() * 20.0;

        // Update streaming system (process completed loads)
        self.streaming_system.update();

        // Setup camera matrices for frustum culling
        let view = Mat4::look_at_rh(self.camera_position, Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(
            70.0_f32.to_radians(),
            16.0 / 9.0,
            0.1,
            1000.0,
        );

        let frustum = Frustum::from_view_projection(proj * view);

        // Update priorities based on visibility and distance
        self.streaming_system
            .update_priorities(&frustum, self.camera_position);

        // Trigger loading for visible meshes
        let mesh_database = &self.mesh_database;
        self.streaming_system
            .load_visible_meshes(&|id: &str| mesh_database.get(id).cloned());

        // Log streaming statistics
        trace!(
            "Streaming stats: {}/{} meshes loaded",
            self.streaming_system.loaded_count(),
            self.streaming_system.total_count()
        );

        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        // For now, just update without rendering
        // Full rendering integration would happen here
        Ok(())
    }
}

#[pollster::main]
async fn main() -> Result<()> {
    praxis::praxis_utils::setup_logging();

    info!("Starting mesh streaming demo");

    let event_loop = EventLoop::new().map_err(|e| praxis::praxis_utils::eyre::eyre!("{}", e))?;
    let window = WindowManager::create_window(&event_loop, "Mesh Streaming Demo", 1280, 720)?;

    let mut demo = MeshStreamingDemo::new(window.clone()).await?;
    demo.register_meshes()?;

    info!("Mesh streaming demo initialized");

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
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    demo.render_context.handle_resize();
                }
                Event::AboutToWait => {
                    let now = std::time::Instant::now();
                    let delta_time = (now - last_frame).as_secs_f32();
                    last_frame = now;

                    if let Err(e) = demo.update(delta_time) {
                        praxis::praxis_utils::error!("Update error: {}", e);
                        elwt.exit();
                    }

                    if let Err(e) = demo.render() {
                        praxis::praxis_utils::error!("Render error: {}", e);
                        elwt.exit();
                    }
                }
                _ => {}
            }
        })
        .map_err(|e| praxis::praxis_utils::eyre::eyre!("{}", e))?;

    Ok(())
}
