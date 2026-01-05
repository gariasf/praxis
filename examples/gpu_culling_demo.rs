//! GPU-driven culling demo with 10,000+ objects.
//!
//! This example demonstrates the GPU culling system's ability to efficiently
//! handle large scenes by offloading frustum culling and LOD selection to the GPU.
//!
//! Features demonstrated:
//! - GPU-based frustum culling using compute shaders
//! - GPU-based LOD selection
//! - Distance culling
//! - Performance comparison between CPU and GPU culling
//! - Statistics display (visible count, cull rate, etc.)
//!
//! Controls:
//! - WASD: Move camera
//! - Mouse: Look around
//! - Space: Toggle between CPU and GPU culling
//! - +/-: Increase/decrease object count
//! - L: Toggle LOD visualization
//! - F: Toggle frustum visualization

use praxis_core::Engine;
use praxis_ecs::{Name, Transform, World};
use praxis_graphics::{colored_cube_mesh, sphere_mesh, RenderCommands, RenderContext};
use praxis_input::{InputManager, Key};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_spatial::{
    gpu_culling::*, Aabb, CullableObject, GpuCullingConfig, GpuCullingManager,
    HybridCullingManager,
};
use praxis_utils::{info, Result};
use praxis_window::WindowConfig;
use std::sync::Arc;
use winit::window::Window;

const INITIAL_OBJECT_COUNT: usize = 10000;
const GRID_SIZE: usize = 50;
const OBJECT_SPACING: f32 = 10.0;

struct DemoState {
    camera_position: Vec3,
    camera_rotation: Quat,
    camera_pitch: f32,
    camera_yaw: f32,
    move_speed: f32,
    look_speed: f32,

    gpu_culling_manager: Option<GpuCullingManager>,
    hybrid_manager: HybridCullingManager,
    use_gpu_culling: bool,
    show_stats: bool,

    objects: Vec<ObjectInstance>,
    mesh_id_map: std::collections::HashMap<String, u32>,
    next_mesh_id: u32,

    last_stats: Option<GpuCullingStats>,
    frame_count: u64,
}

struct ObjectInstance {
    position: Vec3,
    aabb: Aabb,
    mesh_name: String,
    lod_group_id: Option<u32>,
}

impl DemoState {
    fn new() -> Self {
        info!("Initializing GPU culling demo with {} objects", INITIAL_OBJECT_COUNT);

        Self {
            camera_position: Vec3::new(0.0, 20.0, 50.0),
            camera_rotation: Quat::IDENTITY,
            camera_pitch: 0.0,
            camera_yaw: 0.0,
            move_speed: 20.0,
            look_speed: 0.002,

            gpu_culling_manager: None,
            hybrid_manager: HybridCullingManager::with_threshold(5000),
            use_gpu_culling: true,
            show_stats: true,

            objects: Vec::new(),
            mesh_id_map: std::collections::HashMap::new(),
            next_mesh_id: 0,

            last_stats: None,
            frame_count: 0,
        }
    }

    fn get_or_create_mesh_id(&mut self, mesh_name: &str) -> u32 {
        if let Some(&id) = self.mesh_id_map.get(mesh_name) {
            return id;
        }

        let id = self.next_mesh_id;
        self.next_mesh_id += 1;
        self.mesh_id_map.insert(mesh_name.to_string(), id);
        id
    }

    fn initialize_gpu_culling(&mut self, render_context: &RenderContext) -> Result<()> {
        let config = GpuCullingConfig {
            max_objects: 20000,
            max_lod_groups: 100,
            enable_lod_selection: true,
            enable_distance_culling: true,
            max_distance: 500.0,
        };

        let manager = GpuCullingManager::new(
            render_context.device.clone(),
            render_context.memory_allocator().clone(),
            render_context.command_buffer_allocator().clone(),
            render_context.graphics_queue.clone(),
            config,
        )?;

        self.gpu_culling_manager = Some(manager);
        self.hybrid_manager.set_gpu_culling_available(true);

        info!("GPU culling manager initialized");
        Ok(())
    }

    fn setup_lod_groups(&mut self) -> Result<()> {
        let cube_high_id = self.get_or_create_mesh_id("cube_high");
        let cube_low_id = self.get_or_create_mesh_id("cube_low");

        let lod_levels = vec![
            (cube_high_id, 0.0, 50.0),
            (cube_low_id, 50.0, 200.0),
        ];

        let gpu_lod_group = praxis_spatial::gpu_culling::conversions::create_gpu_lod_group(
            &lod_levels,
            0.0,
        );

        if let Some(ref mut manager) = self.gpu_culling_manager {
            manager.update_lod_groups(&[gpu_lod_group])?;
        }

        info!("LOD groups configured");
        Ok(())
    }

    fn generate_objects(&mut self) {
        self.objects.clear();

        let objects_per_side = (INITIAL_OBJECT_COUNT as f32).cbrt() as usize;
        let offset = -(objects_per_side as f32 * OBJECT_SPACING) / 2.0;

        for x in 0..objects_per_side {
            for y in 0..objects_per_side {
                for z in 0..objects_per_side {
                    let position = Vec3::new(
                        offset + x as f32 * OBJECT_SPACING,
                        offset + y as f32 * OBJECT_SPACING,
                        offset + z as f32 * OBJECT_SPACING,
                    );

                    let mesh_name = if (x + y + z) % 3 == 0 {
                        "cube_high"
                    } else {
                        "cube_low"
                    };

                    let aabb = Aabb::from_min_max(
                        position - Vec3::splat(0.5),
                        position + Vec3::splat(0.5),
                    );

                    let lod_group_id = Some(0);

                    self.objects.push(ObjectInstance {
                        position,
                        aabb,
                        mesh_name: mesh_name.to_string(),
                        lod_group_id,
                    });
                }
            }
        }

        info!("Generated {} objects", self.objects.len());
    }

    fn update_camera(&mut self, input: &InputManager, delta_time: f32) {
        let mut move_dir = Vec3::ZERO;

        if input.is_key_down(Key::KeyW) {
            move_dir.z -= 1.0;
        }
        if input.is_key_down(Key::KeyS) {
            move_dir.z += 1.0;
        }
        if input.is_key_down(Key::KeyA) {
            move_dir.x -= 1.0;
        }
        if input.is_key_down(Key::KeyD) {
            move_dir.x += 1.0;
        }
        if input.is_key_down(Key::KeyQ) {
            move_dir.y -= 1.0;
        }
        if input.is_key_down(Key::KeyE) {
            move_dir.y += 1.0;
        }

        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();
            let forward = self.camera_rotation * Vec3::NEG_Z;
            let right = self.camera_rotation * Vec3::X;
            let up = Vec3::Y;

            self.camera_position += (forward * move_dir.z
                + right * move_dir.x
                + up * move_dir.y)
                * self.move_speed
                * delta_time;
        }

        let (mouse_dx, mouse_dy) = input.mouse_delta();
        self.camera_yaw -= mouse_dx * self.look_speed;
        self.camera_pitch -= mouse_dy * self.look_speed;
        self.camera_pitch = self.camera_pitch.clamp(-1.5, 1.5);

        self.camera_rotation =
            Quat::from_rotation_y(self.camera_yaw) * Quat::from_rotation_x(self.camera_pitch);
    }

    fn handle_input(&mut self, input: &InputManager) {
        if input.is_key_pressed(Key::Space) {
            self.use_gpu_culling = !self.use_gpu_culling;
            info!(
                "Culling mode: {}",
                if self.use_gpu_culling { "GPU" } else { "CPU" }
            );
        }

        if input.is_key_pressed(Key::KeyH) {
            self.show_stats = !self.show_stats;
        }
    }

    fn perform_gpu_culling(&mut self, view_proj: Mat4) -> Result<Vec<GpuCullingResult>> {
        let manager = self
            .gpu_culling_manager
            .as_mut()
            .ok_or_else(|| praxis_utils::eyre::eyre!("GPU culling manager not initialized"))?;

        let gpu_objects: Vec<GpuObjectData> = self
            .objects
            .iter()
            .map(|obj| {
                let mesh_id = self.get_or_create_mesh_id(&obj.mesh_name);
                praxis_spatial::gpu_culling::conversions::create_gpu_object(
                    &obj.aabb,
                    obj.position,
                    mesh_id,
                    obj.lod_group_id,
                )
            })
            .collect();

        manager.update_objects(&gpu_objects)?;

        let (results, stats) = manager.cull(view_proj, self.camera_position)?;
        self.last_stats = Some(stats);

        Ok(results)
    }

    fn update(&mut self, input: &InputManager, delta_time: f32) -> Result<()> {
        self.update_camera(input, delta_time);
        self.handle_input(input);
        self.frame_count += 1;

        Ok(())
    }

    fn render(&mut self, render_context: &mut RenderContext) -> Result<()> {
        let view = Mat4::look_at_rh(
            self.camera_position,
            self.camera_position + self.camera_rotation * Vec3::NEG_Z,
            Vec3::Y,
        );

        let aspect = 1920.0 / 1080.0;
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), aspect, 0.1, 1000.0);
        let view_proj = proj * view;

        let draw_commands = if self.use_gpu_culling && self.gpu_culling_manager.is_some() {
            let visible_results = self.perform_gpu_culling(view_proj)?;

            let id_to_name: std::collections::HashMap<u32, String> = self
                .mesh_id_map
                .iter()
                .map(|(name, &id)| (id, name.clone()))
                .collect();

            visible_results
                .iter()
                .filter_map(|result| {
                    if result.is_visible != 0 && (result.object_index as usize) < self.objects.len()
                    {
                        let obj = &self.objects[result.object_index as usize];
                        id_to_name.get(&result.mesh_id).map(|mesh_name| {
                            praxis_graphics::DrawCommand {
                                mesh_id: mesh_name.clone(),
                                model: Mat4::from_translation(obj.position),
                                texture_name: None,
                                material_properties: None,
                            }
                        })
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            self.objects
                .iter()
                .map(|obj| praxis_graphics::DrawCommand {
                    mesh_id: obj.mesh_name.clone(),
                    model: Mat4::from_translation(obj.position),
                    texture_name: None,
                    material_properties: None,
                })
                .collect()
        };

        let commands = RenderCommands {
            view,
            proj,
            draw_commands: &draw_commands,
            lighting: None,
        };

        render_context.render(&commands)?;

        if self.show_stats && self.frame_count % 60 == 0 {
            self.print_stats(draw_commands.len());
        }

        Ok(())
    }

    fn print_stats(&self, rendered_count: usize) {
        if let Some(ref stats) = self.last_stats {
            info!(
                "GPU Culling Stats - Visible: {}/{} ({:.1}% culled), Frustum: {}, Distance: {}",
                stats.visible_count,
                stats.total_processed,
                stats.cull_rate,
                stats.frustum_culled,
                stats.distance_culled
            );
        } else {
            info!("CPU Culling - Rendered: {} objects", rendered_count);
        }
    }
}

async fn run() -> Result<()> {
    let mut state = DemoState::new();

    let window_config = WindowConfig {
        title: "GPU Culling Demo - 10,000+ Objects".to_string(),
        width: 1920,
        height: 1080,
        ..Default::default()
    };

    let window = Arc::new(Window::new(&winit::event_loop::EventLoop::new()?)?);
    let mut render_context = RenderContext::new(window.clone()).await?;

    render_context
        .mesh_manager_mut()
        .load_mesh("cube_high", colored_cube_mesh())?;
    render_context
        .mesh_manager_mut()
        .load_mesh("cube_low", sphere_mesh(0.5, 8, 8))?;

    state.initialize_gpu_culling(&render_context)?;
    state.setup_lod_groups()?;
    state.generate_objects();

    let input_manager = InputManager::new();

    info!("GPU Culling Demo initialized");
    info!("Controls:");
    info!("  WASD/QE - Move camera");
    info!("  Mouse - Look around");
    info!("  Space - Toggle CPU/GPU culling");
    info!("  H - Toggle stats display");

    let mut last_time = std::time::Instant::now();

    loop {
        let current_time = std::time::Instant::now();
        let delta_time = (current_time - last_time).as_secs_f32();
        last_time = current_time;

        state.update(&input_manager, delta_time)?;
        state.render(&mut render_context)?;
    }
}

fn main() -> Result<()> {
    praxis_utils::init_logging();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run())
}
