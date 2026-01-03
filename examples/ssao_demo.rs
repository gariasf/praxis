//! SSAO (Screen-Space Ambient Occlusion) demonstration.
//!
//! This example demonstrates the SSAO system working with deferred rendering.
//! It renders a simple scene and applies SSAO to darken occluded areas.

use praxis::prelude::*;
use praxis_graphics::{
    deferred::DeferredRenderer, lighting::LightingUniforms, solid_cube_mesh, DrawCommand,
    SsaoConfig, SsaoRenderer,
};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_utils::Result;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<()> {
    praxis_utils::init()?;

    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("SSAO Demo - Praxis Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(1920, 1080))
            .build(&event_loop)?,
    );

    let mut app = pollster::block_on(SsaoDemo::new(window))?;

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                WindowEvent::Resized(size) => {
                    if size.width > 0 && size.height > 0 {
                        app.resize(size.width, size.height);
                    }
                }
                WindowEvent::RedrawRequested => {
                    if let Err(e) = app.render() {
                        eprintln!("Render error: {}", e);
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                app.window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}

struct SsaoDemo {
    window: Arc<winit::window::Window>,
    render_context: praxis_graphics::RenderContext,
    deferred_renderer: DeferredRenderer,
    ssao_renderer: SsaoRenderer,
    time: f32,
}

impl SsaoDemo {
    async fn new(window: Arc<winit::window::Window>) -> Result<Self> {
        // Initialize render context
        let mut render_context = praxis_graphics::RenderContext::new(window.clone()).await?;

        // Create deferred renderer
        let size = window.inner_size();
        let deferred_renderer = DeferredRenderer::new(
            render_context.device.clone(),
            render_context.memory_allocator.clone(),
            render_context.descriptor_set_allocator.clone(),
            size.width,
            size.height,
        )?;

        // Create SSAO renderer
        let ssao_config = SsaoConfig::default()
            .with_kernel_size(64)
            .with_radius(0.5)
            .with_bias(0.025)
            .with_power(1.2);

        let ssao_renderer = SsaoRenderer::new(
            render_context.device.clone(),
            render_context.memory_allocator.clone(),
            size.width,
            size.height,
            ssao_config,
        )?;

        // Load cube mesh
        render_context
            .mesh_manager_mut()
            .load_mesh("cube", solid_cube_mesh())?;

        // Setup lighting
        let mut lighting = LightingUniforms::new();
        lighting.ambient_color = [0.3, 0.3, 0.35, 1.0];
        lighting.add_directional_light([0.5, -1.0, 0.3], [1.0, 0.95, 0.9], 1.5);
        render_context.lighting_buffer_mut().update(&lighting)?;

        Ok(Self {
            window,
            render_context,
            deferred_renderer,
            ssao_renderer,
            time: 0.0,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        if let Err(e) = self.deferred_renderer.resize(width, height) {
            eprintln!("Failed to resize deferred renderer: {}", e);
        }
        if let Err(e) = self.ssao_renderer.resize(width, height) {
            eprintln!("Failed to resize SSAO renderer: {}", e);
        }
    }

    fn render(&mut self) -> Result<()> {
        self.time += 0.016;

        // Camera setup
        let camera_distance = 8.0;
        let camera_height = 4.0;
        let eye = Vec3::new(
            (self.time * 0.3).cos() * camera_distance,
            camera_height,
            (self.time * 0.3).sin() * camera_distance,
        );
        let center = Vec3::new(0.0, 0.0, 0.0);
        let up = Vec3::Y;

        let view = Mat4::look_at_rh(eye, center, up);
        let proj = Mat4::perspective_rh(
            45.0_f32.to_radians(),
            self.window.inner_size().width as f32 / self.window.inner_size().height as f32,
            0.1,
            100.0,
        );

        // Create draw commands for a grid of cubes
        let mut draw_commands = Vec::new();
        let grid_size = 5;
        let spacing = 2.0;
        let offset = (grid_size as f32 - 1.0) * spacing * 0.5;

        for x in 0..grid_size {
            for z in 0..grid_size {
                let pos_x = x as f32 * spacing - offset;
                let pos_z = z as f32 * spacing - offset;

                // Vary the height for more interesting occlusion
                let height_variation = ((x + z) as f32 * 0.3).sin() * 0.5;

                let model = Mat4::from_translation(Vec3::new(pos_x, height_variation, pos_z))
                    * Mat4::from_quat(Quat::from_rotation_y(self.time * 0.5 + x as f32))
                    * Mat4::from_scale(Vec3::splat(0.7));

                draw_commands.push(DrawCommand {
                    mesh_id: "cube".to_string(),
                    model,
                    texture_name: None,
                    material_properties: None,
                });
            }
        }

        // Record command buffer
        let mut builder = vulkano::command_buffer::AutoCommandBufferBuilder::primary(
            self.render_context.command_buffer_allocator.clone(),
            self.render_context.graphics_queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

        // Render geometry to G-buffer and get SSAO texture
        let gbuffer = self
            .deferred_renderer
            .gbuffer
            .as_ref()
            .ok_or_else(|| praxis_utils::eyre::eyre!("G-buffer not initialized"))?;

        // Run SSAO pass
        let ssao_texture = self
            .ssao_renderer
            .render(&mut builder, gbuffer, proj, view)?;

        // Build final command buffer
        let command_buffer = builder
            .build()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to build command buffer: {}", e))?;

        // Execute command buffer
        use vulkano::sync::GpuFuture;
        let future = vulkano::sync::now(self.render_context.device.clone())
            .then_execute(self.render_context.graphics_queue.clone(), command_buffer)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to execute command buffer: {}", e))?
            .then_signal_fence_and_flush()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to flush: {}", e))?;

        future
            .wait(None)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to wait for GPU: {}", e))?;

        println!("SSAO texture generated: {:?}", ssao_texture);

        Ok(())
    }
}
