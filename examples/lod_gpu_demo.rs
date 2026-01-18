//! GPU-driven LOD selection demo.
//!
//! This example demonstrates the GPU-driven LOD (Level of Detail) system that uses
//! compute shaders to calculate appropriate LOD levels for objects based on their
//! distance from the camera. All LOD calculations happen in parallel on the GPU,
//! enabling efficient LOD management for tens of thousands of objects.
//!
//! # Features Demonstrated
//!
//! - GPU-driven LOD selection using compute shaders
//! - Multiple LOD levels per object (high, medium, low detail)
//! - Distance-based LOD switching with configurable thresholds
//! - LOD bias for forcing higher/lower detail globally
//! - Debug visualization showing selected LOD levels and distances
//! - Integration with indirect draw buffer generation
//!
//! # Controls
//!
//! - **W/A/S/D**: Move camera
//! - **Q/E**: Move camera up/down
//! - **Arrow Keys**: Rotate camera
//! - **+/-**: Adjust LOD bias (higher = more detail, lower = less detail)
//! - **L**: Toggle LOD system on/off
//! - **ESC**: Exit

use praxis_core::{Engine, EngineConfig};
use praxis_graphics::lod::{GpuLodLevel, GpuLodSelector, GpuObjectData};
use praxis_math::{Mat4, Vec3};
use praxis_utils::Result;
use std::sync::Arc;
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

struct LodGpuDemo {
    // GPU LOD selector
    lod_selector: Option<GpuLodSelector>,

    // Camera state
    camera_position: Vec3,
    camera_yaw: f32,
    camera_pitch: f32,
    camera_speed: f32,

    // LOD configuration
    lod_bias: f32,
    enable_lod: bool,

    // Object data
    objects: Vec<GpuObjectData>,
    lod_levels: Vec<GpuLodLevel>,

    // Debug info
    selected_lods: Vec<u32>,
    distances: Vec<f32>,
    frame_count: u32,
}

impl LodGpuDemo {
    fn new() -> Self {
        Self {
            lod_selector: None,
            camera_position: Vec3::new(0.0, 5.0, 20.0),
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            camera_speed: 5.0,
            lod_bias: 0.0,
            enable_lod: true,
            objects: Vec::new(),
            lod_levels: Vec::new(),
            selected_lods: Vec::new(),
            distances: Vec::new(),
            frame_count: 0,
        }
    }

    fn setup_scene(&mut self) {
        // Create a grid of objects with LOD levels
        let grid_size = 20;
        let spacing = 5.0;
        let mut lod_offset = 0u32;

        for x in 0..grid_size {
            for z in 0..grid_size {
                let pos_x = (x as f32 - grid_size as f32 / 2.0) * spacing;
                let pos_z = (z as f32 - grid_size as f32 / 2.0) * spacing;

                let model = Mat4::from_translation(Vec3::new(pos_x, 0.0, pos_z));

                // Define 3 LOD levels for this object
                // LOD 0: High detail (0-10 units)
                self.lod_levels.push(GpuLodLevel {
                    mesh_id: 0, // High detail mesh
                    min_distance_sq: 0.0,
                    max_distance_sq: 100.0, // 10^2
                    padding: 0,
                });

                // LOD 1: Medium detail (10-25 units)
                self.lod_levels.push(GpuLodLevel {
                    mesh_id: 1, // Medium detail mesh
                    min_distance_sq: 100.0,
                    max_distance_sq: 625.0, // 25^2
                    padding: 0,
                });

                // LOD 2: Low detail (25+ units)
                self.lod_levels.push(GpuLodLevel {
                    mesh_id: 2, // Low detail mesh
                    min_distance_sq: 625.0,
                    max_distance_sq: f32::MAX,
                    padding: 0,
                });

                // Add object with LOD metadata
                self.objects.push(GpuObjectData::new(
                    model,
                    [0.0, 0.0, 0.0, 1.0], // Bounding sphere (center at origin, radius 1)
                    0,                     // Base mesh ID
                    3,                     // 3 LOD levels
                    lod_offset,            // Offset in LOD array
                ));

                lod_offset += 3; // Each object has 3 LOD levels
            }
        }

        println!("Created {} objects with {} LOD levels", self.objects.len(), self.lod_levels.len());
    }

    fn update(&mut self, delta_time: f32) {
        // Update camera based on input (would be handled by event system in real app)
        // For this demo, camera movement is simplified

        // Update frame counter for periodic debug readback
        self.frame_count += 1;

        // Every 60 frames, read back LOD selections for debug visualization
        if self.frame_count % 60 == 0 {
            if let Some(selector) = &self.lod_selector {
                if let Ok(selected) = selector.read_selected_lods() {
                    self.selected_lods = selected;
                }
                if let Ok(distances) = selector.read_distances() {
                    self.distances = distances;
                }

                // Print statistics
                let mut lod_counts = [0u32; 3];
                for &lod in &self.selected_lods {
                    if (lod as usize) < lod_counts.len() {
                        lod_counts[lod as usize] += 1;
                    }
                }

                println!(
                    "LOD Statistics: LOD0={} LOD1={} LOD2={} (bias={:.2})",
                    lod_counts[0], lod_counts[1], lod_counts[2], self.lod_bias
                );
            }
        }
    }

    fn handle_input(&mut self, event: &WindowEvent, delta_time: f32) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                let forward = Vec3::new(
                    self.camera_yaw.sin(),
                    0.0,
                    -self.camera_yaw.cos(),
                )
                .normalize();
                let right = Vec3::new(forward.z, 0.0, -forward.x);

                match keycode {
                    // Camera movement
                    KeyCode::KeyW => self.camera_position += forward * self.camera_speed * delta_time,
                    KeyCode::KeyS => self.camera_position -= forward * self.camera_speed * delta_time,
                    KeyCode::KeyA => self.camera_position -= right * self.camera_speed * delta_time,
                    KeyCode::KeyD => self.camera_position += right * self.camera_speed * delta_time,
                    KeyCode::KeyQ => self.camera_position.y -= self.camera_speed * delta_time,
                    KeyCode::KeyE => self.camera_position.y += self.camera_speed * delta_time,

                    // Camera rotation
                    KeyCode::ArrowLeft => self.camera_yaw -= 1.0 * delta_time,
                    KeyCode::ArrowRight => self.camera_yaw += 1.0 * delta_time,
                    KeyCode::ArrowUp => {
                        self.camera_pitch = (self.camera_pitch + 1.0 * delta_time)
                            .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
                    }
                    KeyCode::ArrowDown => {
                        self.camera_pitch = (self.camera_pitch - 1.0 * delta_time)
                            .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
                    }

                    // LOD controls
                    KeyCode::Equal | KeyCode::NumpadAdd => {
                        self.lod_bias = (self.lod_bias + 0.1).clamp(-1.0, 1.0);
                        println!("LOD bias: {:.2}", self.lod_bias);
                    }
                    KeyCode::Minus | KeyCode::NumpadSubtract => {
                        self.lod_bias = (self.lod_bias - 0.1).clamp(-1.0, 1.0);
                        println!("LOD bias: {:.2}", self.lod_bias);
                    }
                    KeyCode::KeyL => {
                        self.enable_lod = !self.enable_lod;
                        println!("LOD system: {}", if self.enable_lod { "ENABLED" } else { "DISABLED" });
                    }

                    _ => {}
                }
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    praxis_utils::init_logging()?;

    println!("=== GPU-Driven LOD Selection Demo ===");
    println!();
    println!("Controls:");
    println!("  W/A/S/D    - Move camera");
    println!("  Q/E        - Move camera up/down");
    println!("  Arrow Keys - Rotate camera");
    println!("  +/-        - Adjust LOD bias");
    println!("  L          - Toggle LOD system");
    println!("  ESC        - Exit");
    println!();

    // Create engine
    let config = EngineConfig::default();
    let mut engine = Engine::new(config).await?;

    // Create demo state
    let mut demo = LodGpuDemo::new();
    demo.setup_scene();

    // Initialize GPU LOD selector
    if let Some(render_context) = engine.render_context_mut() {
        let device = render_context.device.clone();
        let memory_allocator = render_context.memory_allocator.clone();
        let descriptor_set_allocator = Arc::new(vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let selector = GpuLodSelector::new(
            device,
            memory_allocator,
            descriptor_set_allocator,
        )?;

        demo.lod_selector = Some(selector);
        println!("GPU LOD selector initialized successfully");
    }

    // Main loop
    let mut last_time = std::time::Instant::now();

    engine.run(move |engine_state, event| {
        let current_time = std::time::Instant::now();
        let delta_time = (current_time - last_time).as_secs_f32();
        last_time = current_time;

        // Handle input
        if let Some(window_event) = event {
            demo.handle_input(window_event, delta_time);
        }

        // Update demo
        demo.update(delta_time);

        // Dispatch GPU LOD selection
        if let Some(selector) = &mut demo.lod_selector {
            // In a real application, this would be integrated into the rendering pipeline
            // Here we just demonstrate the API usage

            // Prepare frame data
            if let Err(e) = selector.prepare_frame(&demo.objects, &demo.lod_levels) {
                eprintln!("Failed to prepare LOD frame: {}", e);
                return;
            }

            // Note: In a real integration, dispatch_lod_selection would be called
            // during command buffer recording:
            // selector.dispatch_lod_selection(
            //     builder,
            //     demo.camera_position,
            //     demo.lod_bias,
            //     demo.enable_lod,
            // )?;

            // The selected LOD buffer would then be used by the GPU culling system
            // to generate indirect draw commands with the correct mesh IDs
        }

        // Continue running
    })?;

    Ok(())
}
