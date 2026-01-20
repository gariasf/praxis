//! Comprehensive memory profiling and VRAM tracking demonstration.
//!
//! This example demonstrates the GPU memory profiling system including:
//! - Texture allocation tracking
//! - Mesh buffer monitoring
//! - Descriptor set overhead tracking
//! - Memory correlation with render statistics
//! - Historical trend analysis
//! - CSV export for external analysis
//!
//! The demo creates various GPU resources and shows how memory usage correlates
//! with rendering optimizations like LOD and culling.

use praxis_core::{App, AppConfig, FrameInput};
use praxis_graphics::{
    colored_cube_mesh, utilities::memory_profiler::MemoryCategory, DrawCommand, RenderCommands,
    RenderContext,
};
use praxis_math::{Mat4, Vec3};
use praxis_utils::Result;
use std::sync::Arc;
use winit::window::Window;

struct MemoryProfilingDemo {
    render_context: RenderContext,
    frame_count: u64,
    camera_distance: f32,
    rotation: f32,
    show_stats: bool,
}

impl MemoryProfilingDemo {
    async fn new(window: Arc<Window>) -> Result<Self> {
        let mut render_context = RenderContext::new(window).await?;

        // Enable both render stats and memory profiling
        render_context.set_render_stats_enabled(true);
        render_context.set_memory_profiling_enabled(true);

        // Load some test meshes
        println!("\n=== Loading Test Meshes ===");
        let mesh_mgr = render_context.mesh_manager_mut();

        // Small cube
        let small_cube = colored_cube_mesh();
        mesh_mgr.load_mesh("small_cube", small_cube.clone())?;

        // Medium cube (scaled up in code)
        mesh_mgr.load_mesh("medium_cube", small_cube.clone())?;

        // Large cube
        mesh_mgr.load_mesh("large_cube", small_cube)?;

        // Load test textures
        println!("\n=== Creating Test Textures ===");
        let tex_mgr = render_context.texture_manager_mut();

        // Create procedural textures of various sizes
        let sizes = vec![
            (256, "small"),
            (512, "medium"),
            (1024, "large"),
            (2048, "huge"),
        ];

        for (size, name) in sizes {
            let mut data = vec![0u8; (size * size * 4) as usize];
            // Create a simple gradient pattern
            for y in 0..size {
                for x in 0..size {
                    let idx = ((y * size + x) * 4) as usize;
                    data[idx] = ((x * 255) / size) as u8; // R
                    data[idx + 1] = ((y * 255) / size) as u8; // G
                    data[idx + 2] = 128; // B
                    data[idx + 3] = 255; // A
                }
            }

            tex_mgr.load_texture_from_bytes(format!("gradient_{}", name), &data, size, size)?;

            println!("Created {}x{} texture: gradient_{}", size, size, name);
        }

        println!("\n=== Initial Memory State ===");
        let profiler = render_context.memory_profiler();
        println!("Total VRAM: {:.2} MB", profiler.total_allocated_mb());
        println!(
            "Texture memory: {:.2} MB",
            profiler.category_mb(MemoryCategory::Texture)
        );
        println!(
            "Mesh buffers: {:.2} MB",
            profiler.category_mb(MemoryCategory::MeshBuffer)
        );
        println!("Active allocations: {}", profiler.allocation_count());

        Ok(Self {
            render_context,
            frame_count: 0,
            camera_distance: 10.0,
            rotation: 0.0,
            show_stats: true,
        })
    }

    fn update(&mut self, _delta_time: f32) -> Result<()> {
        self.frame_count += 1;
        self.rotation += 0.01;

        // Print memory stats every 60 frames
        if self.show_stats && self.frame_count % 60 == 0 {
            println!("\n=== Frame {} Memory Stats ===", self.frame_count);

            let profiler = self.render_context.memory_profiler();
            println!(
                "Total VRAM: {:.2} MB (Peak: {:.2} MB)",
                profiler.total_allocated_mb(),
                profiler.peak_mb()
            );

            println!("\nBreakdown by category:");
            for category in [
                MemoryCategory::Texture,
                MemoryCategory::MeshBuffer,
                MemoryCategory::DescriptorSet,
                MemoryCategory::UniformBuffer,
                MemoryCategory::ComputeBuffer,
                MemoryCategory::RenderTarget,
            ] {
                let mb = profiler.category_mb(category);
                if mb > 0.0 {
                    println!("  {}: {:.2} MB", category.name(), mb);
                }
            }

            // Show render stats correlation
            if let Some(latest_stats) = self.render_context.render_stats_history().latest() {
                println!("\nRendering metrics:");
                println!("  Visible objects: {}", latest_stats.visible_objects);
                println!("  Draw calls: {}", latest_stats.draw_calls);
                println!(
                    "  Culling efficiency: {:.1}%",
                    latest_stats.culling_efficiency()
                );

                if let Some(ref mem) = latest_stats.memory_snapshot {
                    println!("  VRAM at frame: {:.2} MB", mem.total_mb());
                }
            }

            // Show memory history trends
            let history = profiler.history();
            if !history.is_empty() {
                println!("\nMemory trends (last {} frames):", history.len());
                println!(
                    "  Average total: {:.2} MB",
                    history.avg_total_bytes() / 1_048_576.0
                );
                println!("  Global peak: {:.2} MB", history.global_peak_mb());
            }
        }

        // Export stats every 300 frames
        if self.frame_count == 300 {
            println!("\n=== Exporting Statistics ===");

            if let Err(e) = self
                .render_context
                .export_render_stats_csv("render_stats_with_memory.csv")
            {
                eprintln!("Failed to export render stats: {}", e);
            } else {
                println!(
                    "Exported render statistics with memory data to render_stats_with_memory.csv"
                );
            }
        }

        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        // Create camera matrices
        let view = Mat4::look_at_rh(
            Vec3::new(
                self.camera_distance * self.rotation.cos(),
                5.0,
                self.camera_distance * self.rotation.sin(),
            ),
            Vec3::ZERO,
            Vec3::Y,
        );

        let aspect = 1920.0 / 1080.0;
        let proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 1000.0);

        // Create a grid of objects to demonstrate memory usage with scale
        let mut draw_commands = Vec::new();

        for x in -2..=2 {
            for z in -2..=2 {
                let position = Vec3::new(x as f32 * 3.0, 0.0, z as f32 * 3.0);

                draw_commands.push(DrawCommand {
                    mesh_id: "small_cube".to_string(),
                    model: Mat4::from_translation(position),
                    texture_name: Some("gradient_medium".to_string()),
                    material_properties: None,
                    material_instance_id: None,
                    bone_matrices: None,
                });
            }
        }

        let cmds = RenderCommands {
            view,
            proj,
            draw_commands: &draw_commands,
            lighting: None,
        };

        self.render_context.render(&cmds)?;

        Ok(())
    }
}

impl App for MemoryProfilingDemo {
    fn update(&mut self, _input: &FrameInput) -> praxis_utils::Result<()> {
        self.update(0.016)?;
        self.render()?;
        Ok(())
    }
}

fn main() -> Result<()> {
    praxis_utils::logging::init()?;

    let config = AppConfig {
        title: "Memory Profiling Demo".to_string(),
        width: 1920,
        height: 1080,
        ..Default::default()
    };

    println!("=== Memory Profiling and VRAM Tracking Demo ===\n");
    println!("This demo demonstrates GPU memory tracking:");
    println!("  - Texture allocations (various sizes)");
    println!("  - Mesh buffer tracking");
    println!("  - Memory correlation with render stats");
    println!("  - Historical trend analysis");
    println!("\nMemory stats are printed every 60 frames.");
    println!("Statistics will be exported to CSV after 300 frames.");
    println!("\nPress Ctrl+C to exit.\n");

    praxis_core::run(config, MemoryProfilingDemo::new)
}
