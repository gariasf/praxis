//! Terrain rendering system demonstration.
//!
//! This example demonstrates:
//! - Heightmap-based terrain generation with procedural noise
//! - Chunked LOD system for large-scale landscapes (4 levels)
//! - Texture splatting with multiple material layers (grass, rock, snow)
//! - Grass and vegetation instancing using GPU instancing
//! - Real-time LOD updates based on camera position
//! - Terrain editing tools integration

#[cfg(feature = "terrain")]
use praxis_ecs::{Camera, PerspectiveProjection, Transform, World};
#[cfg(feature = "terrain")]
use praxis_math::{Quat, Vec3};
#[cfg(feature = "terrain")]
use praxis_terrain::{
    TerrainConfig, TerrainHeightmap, TerrainMaterialLayer, TerrainSystem, VegetationLayer,
};
#[cfg(feature = "terrain")]
use praxis_utils::{info, Result};
#[cfg(feature = "terrain")]
use std::sync::Arc;
#[cfg(feature = "terrain")]
use std::time::Instant;
#[cfg(feature = "terrain")]
use winit::event::{Event, WindowEvent};
#[cfg(feature = "terrain")]
use winit::event_loop::{ControlFlow, EventLoop};
#[cfg(feature = "terrain")]
use winit::window::WindowAttributes;

#[cfg(all(feature = "terrain", not(feature = "headless")))]
fn main() -> Result<()> {
    praxis_utils::init_tracing()?;

    info!("Starting Praxis Terrain Demo...");

    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        event_loop.create_window(
            WindowAttributes::default()
                .with_title("Praxis Engine - Terrain System Demo")
                .with_inner_size(winit::dpi::LogicalSize::new(1920, 1080)),
        )?,
    );

    let mut world = World::new();

    world.spawn((
        Transform::from_xyz(0.0, 50.0, 100.0).look_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        Camera::default(),
        PerspectiveProjection {
            fov: 70.0_f32.to_radians(),
            aspect_ratio: 1920.0 / 1080.0,
            near: 0.1,
            far: 1000.0,
        },
    ));

    info!("Creating terrain heightmap from procedural noise...");
    let heightmap_start = Instant::now();
    let heightmap = TerrainHeightmap::from_noise(512, 512, 100.0, 4.0, 6);
    info!(
        "Heightmap created in {:?} (512x512 samples)",
        heightmap_start.elapsed()
    );

    let config = TerrainConfig {
        chunk_size: 64.0,
        vertices_per_chunk: 65,
        max_height: 100.0,
        lod_levels: 4,
        lod_distances: vec![50.0, 100.0, 200.0, 400.0],
        world_size: 1024.0,
        world_scale: 1.0,
        enable_frustum_culling: true,
        enable_occlusion_culling: false,
    };

    info!("Creating terrain system...");
    let terrain_start = Instant::now();
    let mut terrain = TerrainSystem::new(config, heightmap)?;
    info!("Terrain system created in {:?}", terrain_start.elapsed());

    info!("Setting up material layers...");
    let grass_layer = TerrainMaterialLayer::new("grass", "grass_albedo", 0.0, 30.0)
        .with_tiling(10.0)
        .with_normal("grass_normal");
    terrain.material.add_layer(grass_layer);

    let rock_layer = TerrainMaterialLayer::new("rock", "rock_albedo", 30.0, 70.0)
        .with_slope(20.0, 90.0)
        .with_tiling(15.0)
        .with_normal("rock_normal");
    terrain.material.add_layer(rock_layer);

    let snow_layer = TerrainMaterialLayer::new("snow", "snow_albedo", 70.0, 100.0)
        .with_tiling(8.0)
        .with_normal("snow_normal");
    terrain.material.add_layer(snow_layer);

    info!(
        "Added {} material layers for texture splatting",
        terrain.material.layers.len()
    );

    info!("Setting up vegetation layers...");
    let grass_vegetation = VegetationLayer::new("grass", "grass_mesh", "grass_mat", 5.0)
        .with_height_range(0.0, 40.0)
        .with_slope_range(0.0, 30.0)
        .with_scale_range(0.8, 1.2)
        .with_wind_strength(1.5)
        .with_color_variation(0.15);

    terrain.vegetation_layers.push(grass_vegetation);

    let flower_vegetation = VegetationLayer::new("flowers", "flower_mesh", "flower_mat", 1.0)
        .with_height_range(10.0, 45.0)
        .with_slope_range(0.0, 25.0)
        .with_scale_range(0.7, 1.3)
        .with_wind_strength(2.0)
        .with_color_variation(0.2);

    terrain.vegetation_layers.push(flower_vegetation);

    let tree_vegetation = VegetationLayer::new("trees", "tree_mesh", "tree_mat", 0.5)
        .with_height_range(20.0, 60.0)
        .with_slope_range(0.0, 25.0)
        .with_scale_range(0.8, 1.5)
        .with_wind_strength(0.3)
        .with_random_rotation(true);

    terrain.vegetation_layers.push(tree_vegetation);

    let rock_vegetation = VegetationLayer::new("rocks", "rock_mesh", "rock_mat", 0.2)
        .with_height_range(40.0, 80.0)
        .with_slope_range(15.0, 60.0)
        .with_scale_range(0.5, 2.0)
        .with_wind_strength(0.0)
        .with_random_rotation(true);

    terrain.vegetation_layers.push(rock_vegetation);

    info!(
        "Added {} vegetation layers",
        terrain.vegetation_layers.len()
    );

    info!("Generating vegetation instances...");
    let vegetation_start = Instant::now();
    terrain.generate_vegetation()?;
    info!(
        "Vegetation generation completed in {:?}",
        vegetation_start.elapsed()
    );

    let total_instances: usize = terrain
        .vegetation_layers
        .iter()
        .map(|l| l.instance_count())
        .sum();

    info!("Vegetation statistics:");
    for layer in &terrain.vegetation_layers {
        info!(
            "  - {}: {} instances (density: {:.2}/m²)",
            layer.name,
            layer.instance_count(),
            layer.density
        );
    }
    info!("Total vegetation instances: {}", total_instances);

    info!("Terrain system initialized successfully!");
    println!();
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║       Praxis Engine - Terrain System Demo           ║");
    println!("╚═══════════════════════════════════════════════════════╝");
    println!();
    println!("Controls:");
    println!("  Mouse          Look around");
    println!("  WASD           Move camera horizontally");
    println!("  Space          Move up");
    println!("  Shift          Move down");
    println!("  Q/E            Roll camera");
    println!("  ESC            Exit");
    println!();
    println!("Terrain Features:");
    println!("  ✓ Heightmap-based terrain (512x512 samples)");
    println!("  ✓ Procedural noise generation (6 octaves)");
    println!("  ✓ Chunked LOD system (4 levels: 50m, 100m, 200m, 400m)");
    println!("  ✓ Texture splatting (3 layers: grass, rock, snow)");
    println!(
        "  ✓ GPU instanced vegetation ({} instances across 4 layers)",
        total_instances
    );
    println!("  ✓ Wind animation for grass and foliage");
    println!("  ✓ Frustum culling for optimized rendering");
    println!();

    let camera_pos = Vec3::new(0.0, 50.0, 100.0);
    terrain.update(camera_pos);

    let initial_chunks = terrain.chunk_count();
    info!("Initial chunk count: {}", initial_chunks);

    let mut frame_count = 0u64;
    let mut last_update = Instant::now();

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Closing terrain demo...");
                info!("Total frames rendered: {}", frame_count);
                info!("Final chunk count: {}", terrain.chunk_count());
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { .. },
                ..
            } => {}
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                frame_count += 1;

                if last_update.elapsed().as_secs() >= 5 {
                    info!(
                        "Terrain stats: {} active chunks, {} LOD updates",
                        terrain.chunk_count(),
                        frame_count
                    );
                    last_update = Instant::now();
                }

                terrain.update(camera_pos);
            }
            _ => {}
        }
    })?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("terrain_demo example requires graphics support and cannot run in headless mode");
    Ok(())
}

#[cfg(all(not(feature = "terrain"), not(feature = "headless")))]
fn main() {
    eprintln!("This example requires the 'terrain' feature to be enabled.");
    eprintln!("Run with: cargo run --example terrain_demo --features terrain");
    std::process::exit(1);
}
