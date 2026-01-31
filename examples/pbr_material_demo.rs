//! PBR Material System Demo
//!
//! Demonstrates the shader pipeline and material system with:
//! - Pipeline state objects
//! - Material creation with PBR properties
//! - Descriptor set binding
//! - Forward rendering with physically-based lighting

use praxis_core::App;
use praxis_ecs::World;
use praxis_graphics::{
    material::{Material, MaterialProperties},
    pipeline_state::{PipelineStateConfig, PipelineCache},
    shader_reflection::{PipelineReflection, ShaderReflection, ShaderStage},
    RenderContext, RenderCommands, DrawCommand,
};
use praxis_math::{Mat4, Vec3};
use praxis_utils::{info, Result};
use praxis_window::WindowConfig;
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

struct PbrMaterialDemo {
    world: World,
    render_context: Option<RenderContext>,
    camera_distance: f32,
    camera_angle: f32,
    pipeline_cache: Option<PipelineCache>,
}

impl PbrMaterialDemo {
    fn new() -> Self {
        Self {
            world: World::new(),
            render_context: None,
            camera_distance: 5.0,
            camera_angle: 0.0,
            pipeline_cache: None,
        }
    }

    async fn initialize(&mut self, window: Arc<winit::window::Window>) -> Result<()> {
        info!("Initializing PBR material demo...");

        // Create render context
        let mut render_context = RenderContext::new(window).await?;

        // Load a simple mesh (cube)
        let cube_mesh = praxis_graphics::colored_cube_mesh();
        render_context.mesh_manager_mut().load_mesh("cube", cube_mesh)?;

        // Create materials with different PBR properties

        // Material 1: Rough non-metallic (like concrete)
        let rough_material = MaterialProperties::new()
            .with_base_color([0.8, 0.8, 0.8, 1.0])
            .with_metallic(0.0)
            .with_roughness(0.9);

        // Material 2: Smooth metallic (like polished metal)
        let metal_material = MaterialProperties::new()
            .with_base_color([1.0, 0.8, 0.5, 1.0])
            .with_metallic(1.0)
            .with_roughness(0.1);

        // Material 3: Semi-metallic with medium roughness (like copper)
        let copper_material = MaterialProperties::new()
            .with_base_color([0.95, 0.64, 0.54, 1.0])
            .with_metallic(1.0)
            .with_roughness(0.5);

        info!("Created materials with varying PBR properties");

        // Setup lighting
        let mut lighting_data = praxis_graphics::lighting::LightingUniforms::new();
        lighting_data.set_ambient_color([0.1, 0.1, 0.15]);
        
        // Add directional light (sun)
        lighting_data.add_directional_light(
            Vec3::new(-0.3, -1.0, -0.5).normalize(),
            Vec3::new(1.0, 0.95, 0.9),
            1.5,
        );

        // Upload lighting to GPU
        render_context.update_lighting(&lighting_data)?;

        info!("Lighting configured");

        // Initialize pipeline cache
        let device = render_context.device.clone();
        let pipeline_cache = PipelineCache::new(device);

        self.render_context = Some(render_context);
        self.pipeline_cache = Some(pipeline_cache);

        info!("PBR material demo initialized successfully");
        Ok(())
    }

    fn update(&mut self, delta_time: f32) {
        // Rotate camera
        self.camera_angle += delta_time * 0.5;
    }

    fn render(&mut self) -> Result<()> {
        let render_context = self.render_context.as_mut().unwrap();

        // Setup camera
        let camera_pos = Vec3::new(
            self.camera_distance * self.camera_angle.cos(),
            2.0,
            self.camera_distance * self.camera_angle.sin(),
        );
        let view = Mat4::look_at_rh(
            camera_pos,
            Vec3::ZERO,
            Vec3::new(0.0, 1.0, 0.0),
        );

        let aspect_ratio = 1920.0 / 1080.0;
        let proj = Mat4::perspective_rh(
            std::f32::consts::PI / 4.0,
            aspect_ratio,
            0.1,
            100.0,
        );

        // Create draw commands for cubes with different materials
        let draw_commands = vec![
            // Rough non-metallic cube (left)
            DrawCommand {
                mesh_id: "cube".to_string(),
                model: Mat4::from_translation(Vec3::new(-2.5, 0.0, 0.0)),
                texture_name: None,
                material_properties: Some(
                    MaterialProperties::new()
                        .with_base_color([0.8, 0.8, 0.8, 1.0])
                        .with_metallic(0.0)
                        .with_roughness(0.9),
                ),
                material_instance_id: None,
                bone_matrices: None,
            },
            // Smooth metallic cube (center)
            DrawCommand {
                mesh_id: "cube".to_string(),
                model: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                texture_name: None,
                material_properties: Some(
                    MaterialProperties::new()
                        .with_base_color([1.0, 0.8, 0.5, 1.0])
                        .with_metallic(1.0)
                        .with_roughness(0.1),
                ),
                material_instance_id: None,
                bone_matrices: None,
            },
            // Copper-like cube (right)
            DrawCommand {
                mesh_id: "cube".to_string(),
                model: Mat4::from_translation(Vec3::new(2.5, 0.0, 0.0)),
                texture_name: None,
                material_properties: Some(
                    MaterialProperties::new()
                        .with_base_color([0.95, 0.64, 0.54, 1.0])
                        .with_metallic(1.0)
                        .with_roughness(0.5),
                ),
                material_instance_id: None,
                bone_matrices: None,
            },
        ];

        let commands = RenderCommands {
            view,
            proj,
            draw_commands: &draw_commands,
            lighting: None, // Use previously uploaded lighting
        };

        render_context.render(&commands)?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    praxis_utils::init_logging();

    info!("Starting PBR Material Demo");

    let event_loop = EventLoop::new();
    let window = WindowConfig::default()
        .with_title("PBR Material Demo - Shader Pipeline & Materials")
        .build(&event_loop)?;
    let window = Arc::new(window);

    let mut demo = PbrMaterialDemo::new();
    demo.initialize(window.clone()).await?;

    let mut last_frame_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Close requested, shutting down");
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                let now = std::time::Instant::now();
                let delta_time = (now - last_frame_time).as_secs_f32();
                last_frame_time = now;

                demo.update(delta_time);

                if let Err(e) = demo.render() {
                    eprintln!("Render error: {}", e);
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}
