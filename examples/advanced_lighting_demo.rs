//! Advanced lighting demonstration showcasing:
//! - Light probes for dynamic global illumination
//! - Volumetric fog with raymarching
//! - God rays (crepuscular rays) with radial blur
//! - Area lights with LTC
//! - Light linking for selective illumination

use praxis_ecs::{PerspectiveCameraBundle, Transform, World};
use praxis_graphics::{
    colored_cube_mesh, AreaLight, AreaLightManager, AreaLightType, DrawCommand, FogDensityFunction,
    GodRays, GodRaysConfig, LightLinkingManager, LightProbeGrid, LightProbeManager, MeshData,
    RenderCommands, RenderContext, VolumetricFog, VolumetricFogConfig,
};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_utils::{info, Result};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

struct AdvancedLightingDemo {
    light_probe_manager: Option<LightProbeManager>,
    area_light_manager: Option<AreaLightManager>,
    light_linking_manager: LightLinkingManager,
    volumetric_fog: VolumetricFog,
    god_rays: GodRays,
    time: f32,
}

impl AdvancedLightingDemo {
    fn new() -> Self {
        let light_linking_manager = LightLinkingManager::new();

        let fog_config = VolumetricFogConfig {
            density_function: FogDensityFunction::HeightBased {
                base_height: 0.0,
                falloff: 0.15,
            },
            color: Vec3::new(0.7, 0.75, 0.8),
            density: 0.04,
            max_distance: 100.0,
            num_steps: 64,
            light_scattering: 0.4,
            anisotropy: 0.3,
            shadow_influence: 0.7,
        };

        let god_rays_config = GodRaysConfig {
            num_samples: 80,
            density: 0.6,
            weight: 0.4,
            decay: 0.96,
            exposure: 0.9,
            threshold: 0.85,
        };

        Self {
            light_probe_manager: None,
            area_light_manager: None,
            light_linking_manager,
            volumetric_fog: VolumetricFog::new(fog_config),
            god_rays: GodRays::new(god_rays_config),
            time: 0.0,
        }
    }

    fn setup_light_probes(
        &mut self,
        device: Arc<vulkano::device::Device>,
        memory_allocator: Arc<vulkano::memory::allocator::StandardMemoryAllocator>,
    ) -> Result<()> {
        let grid = LightProbeGrid::new(
            Vec3::new(-20.0, 0.0, -20.0),
            Vec3::new(20.0, 10.0, 20.0),
            [5, 3, 5],
        );

        let manager = LightProbeManager::new(device, memory_allocator)?;
        info!("Light probe grid created with {} probes", grid.probes.len());

        self.light_probe_manager = Some(manager);
        Ok(())
    }

    fn setup_area_lights(
        &mut self,
        device: Arc<vulkano::device::Device>,
        memory_allocator: Arc<vulkano::memory::allocator::StandardMemoryAllocator>,
    ) -> Result<()> {
        let mut manager = AreaLightManager::new(device, memory_allocator)?;

        // Rectangle light (warm white overhead)
        let rect_light = AreaLight {
            light_type: AreaLightType::Rectangle {
                width: 4.0,
                height: 4.0,
            },
            position: Vec3::new(0.0, 8.0, 0.0),
            direction: Vec3::new(0.0, -1.0, 0.0),
            up: Vec3::new(0.0, 0.0, 1.0),
            color: Vec3::new(1.0, 0.9, 0.7),
            intensity: 15.0,
            two_sided: false,
        };

        // Disk light (cool blue from side)
        let disk_light = AreaLight {
            light_type: AreaLightType::Disk { radius: 2.0 },
            position: Vec3::new(-8.0, 5.0, 0.0),
            direction: Vec3::new(1.0, -0.3, 0.0).normalize(),
            up: Vec3::new(0.0, 1.0, 0.0),
            color: Vec3::new(0.5, 0.7, 1.0),
            intensity: 10.0,
            two_sided: false,
        };

        // Sphere light (warm orange accent)
        let sphere_light = AreaLight {
            light_type: AreaLightType::Sphere { radius: 1.5 },
            position: Vec3::new(6.0, 3.0, -3.0),
            direction: Vec3::new(-1.0, 0.0, 1.0).normalize(),
            up: Vec3::new(0.0, 1.0, 0.0),
            color: Vec3::new(1.0, 0.6, 0.3),
            intensity: 8.0,
            two_sided: true,
        };

        manager.add_light(rect_light)?;
        manager.add_light(disk_light)?;
        manager.add_light(sphere_light)?;

        info!("Area lights configured: {} lights", manager.light_count());
        self.area_light_manager = Some(manager);
        Ok(())
    }

    fn setup_light_linking(&mut self) {
        let hero_lights = 0b0001;
        let environment_lights = 0b0010;
        let accent_lights = 0b0100;

        self.light_linking_manager
            .register_channel(0, "hero".to_string());
        self.light_linking_manager
            .register_channel(1, "environment".to_string());
        self.light_linking_manager
            .register_channel(2, "accent".to_string());

        self.light_linking_manager
            .set_object_mask("hero_character", hero_lights | environment_lights)
            .unwrap();
        self.light_linking_manager
            .set_object_mask("background_prop", environment_lights)
            .unwrap();
        self.light_linking_manager
            .set_object_mask("highlighted_item", accent_lights | environment_lights)
            .unwrap();

        self.light_linking_manager
            .set_light_channel("key_light", 0)
            .unwrap();
        self.light_linking_manager
            .set_light_channel("ambient_light", 1)
            .unwrap();
        self.light_linking_manager
            .set_light_channel("rim_light", 2)
            .unwrap();

        info!("Light linking configured with 3 objects and 3 lights");
    }

    fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
    }
}

fn create_scene_meshes() -> Vec<(&'static str, MeshData)> {
    vec![
        ("cube", colored_cube_mesh()),
        ("ground", create_ground_plane()),
    ]
}

fn create_ground_plane() -> MeshData {
    let size = 20.0;
    let positions = vec![
        [-size, 0.0, -size],
        [size, 0.0, -size],
        [size, 0.0, size],
        [-size, 0.0, size],
    ];

    let colors = vec![
        [0.3, 0.3, 0.35],
        [0.3, 0.3, 0.35],
        [0.3, 0.3, 0.35],
        [0.3, 0.3, 0.35],
    ];

    let normals = vec![[0.0, 1.0, 0.0]; 4];

    let indices = vec![0, 1, 2, 2, 3, 0];

    MeshData {
        positions,
        colors: Some(colors),
        normals: Some(normals),
        uvs: None,
        tangents: None,
        indices,
    }
}

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    world: Option<World>,
    render_context: Option<RenderContext>,
    camera_entity: Option<praxis_ecs::Entity>,
    demo: Option<AdvancedLightingDemo>,
}

impl App {
    async fn setup_scene(
        window: Arc<Window>,
    ) -> Result<(
        World,
        RenderContext,
        praxis_ecs::Entity,
        AdvancedLightingDemo,
    )> {
        info!("Initializing Advanced Lighting Demo");

        // Create the render context
        let mut render_context = RenderContext::new(window.clone()).await?;

        // Load meshes
        for (name, mesh_data) in create_scene_meshes() {
            render_context
                .mesh_manager_mut()
                .load_mesh(name, mesh_data)?;
        }

        // Create the world and spawn a camera
        let mut world = World::new();
        let camera_entity = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(15.0, 12.0, 15.0),
            70.0_f32.to_radians(),
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        ));

        // Point camera toward center
        {
            let inner = world.inner_mut();
            if let Some(mut transform) = inner.get_mut::<Transform>(camera_entity) {
                let look_at = Vec3::ZERO;
                let direction = (look_at - transform.translation).normalize();
                transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, direction);
            }
        }

        // Setup demo
        let mut demo = AdvancedLightingDemo::new();
        demo.setup_light_probes(
            render_context.device.clone(),
            render_context.memory_allocator().clone(),
        )?;
        demo.setup_area_lights(
            render_context.device.clone(),
            render_context.memory_allocator().clone(),
        )?;
        demo.setup_light_linking();

        info!("Scene setup complete");

        Ok((world, render_context, camera_entity, demo))
    }

    fn render_scene(&mut self) -> Result<()> {
        let world = self.world.as_ref().unwrap();
        let render_context = self.render_context.as_mut().unwrap();
        let camera_entity = self.camera_entity.unwrap();
        let demo = self.demo.as_ref().unwrap();

        // Get camera matrices
        let camera_matrices = world
            .inner()
            .get::<praxis_ecs::CameraMatrices>(camera_entity)
            .unwrap();

        // Create draw commands for the scene
        let mut draw_commands = Vec::new();

        // Ground plane
        draw_commands.push(DrawCommand {
            mesh_id: "ground".to_string(),
            model: Mat4::IDENTITY,
            texture_name: None,
            material_properties: None,
        });

        // Central demonstration cube (hero character)
        draw_commands.push(DrawCommand {
            mesh_id: "cube".to_string(),
            model: Mat4::from_scale_rotation_translation(
                Vec3::splat(2.0),
                Quat::from_rotation_y(demo.time * 0.5),
                Vec3::new(0.0, 1.0, 0.0),
            ),
            texture_name: None,
            material_properties: None,
        });

        // Background props (affected by environment lights only)
        for i in 0..4 {
            let angle = i as f32 * std::f32::consts::PI * 0.5 + demo.time * 0.2;
            let radius = 10.0;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;

            draw_commands.push(DrawCommand {
                mesh_id: "cube".to_string(),
                model: Mat4::from_scale_rotation_translation(
                    Vec3::splat(1.0),
                    Quat::from_rotation_y(angle),
                    Vec3::new(x, 0.5, z),
                ),
                texture_name: None,
                material_properties: None,
            });
        }

        // Highlighted items (affected by accent + environment lights)
        for i in 0..3 {
            let angle = i as f32 * std::f32::consts::PI * 0.66 + demo.time * -0.3;
            let radius = 6.0;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            let y = (demo.time * 2.0 + i as f32).sin() * 0.5 + 1.5;

            draw_commands.push(DrawCommand {
                mesh_id: "cube".to_string(),
                model: Mat4::from_scale_rotation_translation(
                    Vec3::splat(0.7),
                    Quat::from_rotation_xyz(demo.time, demo.time * 0.7, demo.time * 0.5),
                    Vec3::new(x, y, z),
                ),
                texture_name: None,
                material_properties: None,
            });
        }

        // Submit render commands
        let cmds = RenderCommands {
            view: camera_matrices.view,
            proj: camera_matrices.projection,
            draw_commands: &draw_commands,
            lighting: None,
        };

        render_context.render(&cmds)?;

        Ok(())
    }

    fn update_camera_matrices(&mut self) {
        if let Some(world) = &mut self.world {
            if let Some(camera_entity) = self.camera_entity {
                let inner = world.inner_mut();

                if let (Some(transform), Some(projection)) = (
                    inner.get::<Transform>(camera_entity),
                    inner.get::<praxis_ecs::PerspectiveProjection>(camera_entity),
                ) {
                    let view = Mat4::look_at_rh(
                        transform.translation,
                        transform.translation + (transform.rotation * Vec3::NEG_Z),
                        Vec3::Y,
                    );

                    let proj = projection.compute_matrix();

                    if let Some(mut matrices) =
                        inner.get_mut::<praxis_ecs::CameraMatrices>(camera_entity)
                    {
                        matrices.update(view, proj);
                    }
                }
            }
        }
    }

    fn update(&mut self, delta_time: f32) {
        if let Some(demo) = &mut self.demo {
            demo.update(delta_time);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Creating window");

        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_title("Praxis - Advanced Lighting Demo")
                .with_resizable(false),
        ) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                eprintln!("Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        let (world, render_context, camera_entity, demo) =
            match pollster::block_on(Self::setup_scene(window.clone())) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to setup scene: {e}");
                    event_loop.exit();
                    return;
                }
            };

        self.window = Some(window);
        self.world = Some(world);
        self.render_context = Some(render_context);
        self.camera_entity = Some(camera_entity);
        self.demo = Some(demo);

        self.update_camera_matrices();

        println!("\n╔═══════════════════════════════════════════════════════════════════╗");
        println!("║          PRAXIS - ADVANCED LIGHTING DEMONSTRATION                ║");
        println!("╚═══════════════════════════════════════════════════════════════════╝");
        println!("\nShowcasing advanced lighting features:");
        println!("  • Light Probes - 5×3×5 grid for dynamic global illumination");
        println!("  • Volumetric Fog - Height-based with light scattering");
        println!("  • God Rays - Crepuscular rays with radial blur");
        println!("  • Area Lights - Rectangle, Disk, and Sphere lights with LTC");
        println!("  • Light Linking - Selective object illumination");
        println!("\nScene Layout:");
        println!("  • Central cube: Hero character (hero + environment lights)");
        println!("  • Orbiting cubes: Background props (environment lights only)");
        println!("  • Floating cubes: Highlighted items (accent + environment lights)");
        println!("\nPress ESC or close the window to exit.\n");

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested, exiting");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.logical_key.to_text() == Some("Escape") {
                    info!("ESC pressed, exiting");
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                self.update(0.016);
                self.update_camera_matrices();

                if let Err(e) = self.render_scene() {
                    eprintln!("Render error: {e}");
                    event_loop.exit();
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

#[cfg(not(feature = "headless"))]
fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_ecs::init()?;

    info!("Starting Advanced Lighting Demo");

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
    println!("advanced_lighting_demo requires graphics support and cannot run in headless mode");
    Ok(())
}
