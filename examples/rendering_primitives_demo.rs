//! Demonstration of core rendering primitives.
//!
//! This example shows how to use the core rendering primitives:
//! - Vertex structure with bytemuck
//! - Mesh with vertex/index buffers and staging
//! - Buffer abstractions (GpuBuffer, StagingBuffer)
//! - Texture management
//! - Descriptor set caching
//! - GPU resource lifetime tracking
//!
//! Run with: cargo run --example rendering_primitives_demo

use praxis_core::AppConfig;
use praxis_graphics::{
    buffer::{BufferManager, GpuBuffer, StagingBuffer},
    descriptor_manager::{DescriptorSetCache, DescriptorSetKey, ResourceLifetimeTracker},
    mesh::MeshData,
    DrawCommand, GpuMesh, MeshAssetManager, RenderCommands, RenderContext, Vertex3D,
};
use praxis_math::{Mat4, Vec3};
use praxis_utils::{info, Result};
use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() -> Result<()> {
    // Initialize logging
    praxis_utils::init_logging();

    info!("=== Rendering Primitives Demo ===");
    info!("This demo showcases core rendering primitives:");
    info!("- Vertex3D with bytemuck for zero-copy GPU upload");
    info!("- MeshData and GpuMesh with staging buffer pattern");
    info!("- Generic GpuBuffer<T> and StagingBuffer<T> abstractions");
    info!("- BufferManager for centralized buffer management");
    info!("- Descriptor set caching with LRU eviction");
    info!("- Resource lifetime tracking");

    // Create window
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Rendering Primitives Demo")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)?,
    );

    // Create render context
    let rt = tokio::runtime::Runtime::new()?;
    let mut render_context = rt.block_on(RenderContext::new(window.clone()))?;

    info!("\n=== Demonstrating Vertex3D Structure ===");
    demonstrate_vertex_structure();

    info!("\n=== Demonstrating Mesh Creation ===");
    demonstrate_mesh_creation(&mut render_context)?;

    info!("\n=== Demonstrating Buffer Abstractions ===");
    demonstrate_buffer_abstractions(&render_context)?;

    info!("\n=== Demonstrating BufferManager ===");
    demonstrate_buffer_manager(&render_context)?;

    info!("\n=== Demonstrating Resource Lifetime Tracking ===");
    demonstrate_lifetime_tracking();

    info!("\n=== Starting Render Loop ===");
    info!("The demo will render a triangle using the created primitives.");
    info!("Press ESC or close window to exit.");

    // Create a simple triangle mesh
    create_demo_mesh(&mut render_context)?;

    // Camera setup
    let eye = Vec3::new(0.0, 0.0, 3.0);
    let center = Vec3::new(0.0, 0.0, 0.0);
    let up = Vec3::new(0.0, 1.0, 0.0);
    let view = Mat4::look_at_rh(eye, center, up);

    let aspect = 1280.0 / 720.0;
    let proj = Mat4::perspective_rh(std::f32::consts::PI / 4.0, aspect, 0.1, 100.0);

    // Render loop
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Window close requested");
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                ..
            } => {
                if event.physical_key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) {
                    info!("ESC pressed, exiting");
                    elwt.exit();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                // Handle resize
            }
            Event::AboutToWait => {
                // Render frame
                let draw_commands = vec![DrawCommand {
                    mesh_id: "demo_triangle".to_string(),
                    model: Mat4::IDENTITY,
                    texture_name: None,
                    material_properties: None,
                    material_instance_id: None,
                    bone_matrices: None,
                }];

                let render_commands = RenderCommands {
                    view,
                    proj,
                    draw_commands: &draw_commands,
                    lighting: None,
                };

                if let Err(e) = render_context.render(&render_commands) {
                    eprintln!("Render error: {}", e);
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}

fn demonstrate_vertex_structure() {
    info!("Creating Vertex3D instances...");

    // Basic vertex
    let vertex1 = Vertex3D::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    info!("  Basic vertex: position={:?}, color={:?}", vertex1.position, vertex1.color);

    // Vertex with UV
    let vertex2 = Vertex3D::with_uv([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0]);
    info!("  Vertex with UV: position={:?}, color={:?}, uv={:?}", 
        vertex2.position, vertex2.color, vertex2.uv);

    // Vertex with all attributes
    let vertex3 = Vertex3D::with_all(
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.5, 1.0],
    );
    info!("  Full vertex: position={:?}, normal={:?}, color={:?}, uv={:?}",
        vertex3.position, vertex3.normal, vertex3.color, vertex3.uv);

    // Demonstrate bytemuck zero-copy conversion
    info!("\nDemonstrating bytemuck zero-copy conversion:");
    let bytes = bytemuck::bytes_of(&vertex1);
    info!("  Vertex size: {} bytes", bytes.len());
    info!("  First 12 bytes (position): {:?}", &bytes[0..12]);
    info!("  Bytes 12-24 (normal): {:?}", &bytes[12..24]);
    info!("  Bytes 24-36 (color): {:?}", &bytes[24..36]);
    info!("  Total Vertex3D size: {} bytes", std::mem::size_of::<Vertex3D>());
}

fn demonstrate_mesh_creation(render_context: &mut RenderContext) -> Result<()> {
    info!("Creating MeshData...");

    // Create mesh data
    let mesh_data = MeshData::with_colors(
        vec![
            [0.0, 0.5, 0.0],   // Top
            [-0.5, -0.5, 0.0], // Bottom-left
            [0.5, -0.5, 0.0],  // Bottom-right
        ],
        vec![
            [1.0, 0.0, 0.0], // Red
            [0.0, 1.0, 0.0], // Green
            [0.0, 0.0, 1.0], // Blue
        ],
        vec![0, 1, 2],
    );

    info!("  Vertices: {}", mesh_data.positions.len());
    info!("  Indices: {}", mesh_data.indices.len());

    // Convert to Vertex3D
    let vertices = mesh_data.to_vertices();
    info!("  Converted to {} Vertex3D structures", vertices.len());

    // Upload to GPU using staging buffer pattern
    info!("Uploading mesh to GPU (with staging buffers)...");
    let gpu_mesh = GpuMesh::new(
        render_context.memory_allocator.clone(),
        render_context.command_buffer_allocator.clone(),
        render_context.graphics_queue.clone(),
        vertices,
        mesh_data.indices.clone(),
    )?;

    info!("  GPU mesh created:");
    info!("    Vertex count: {}", gpu_mesh.vertex_count);
    info!("    Index count: {}", gpu_mesh.index_count);
    info!("    Vertex buffer size: {} bytes", gpu_mesh.vertex_count as usize * std::mem::size_of::<Vertex3D>());
    info!("    Index buffer size: {} bytes", gpu_mesh.index_count as usize * std::mem::size_of::<u16>());

    // Register mesh with manager
    render_context
        .mesh_manager_mut()
        .register_mesh("demo_triangle", gpu_mesh);

    Ok(())
}

fn demonstrate_buffer_abstractions(render_context: &RenderContext) -> Result<()> {
    info!("Creating generic buffers...");

    // Create GpuBuffer<f32>
    let float_data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
    let gpu_buffer = GpuBuffer::from_data(
        render_context.memory_allocator.clone(),
        render_context.command_buffer_allocator.clone(),
        render_context.graphics_queue.clone(),
        BufferUsage::UNIFORM_BUFFER,
        &float_data,
    )?;

    info!("  GpuBuffer<f32> created:");
    info!("    Element count: {}", gpu_buffer.element_count());
    info!("    Size in bytes: {}", gpu_buffer.size_bytes());

    // Create StagingBuffer
    let staging_data = vec![10u32, 20, 30, 40];
    let staging_buffer = StagingBuffer::new(
        render_context.memory_allocator.clone(),
        &staging_data,
    )?;

    info!("  StagingBuffer<u32> created:");
    info!("    Element count: {}", staging_buffer.element_count());
    info!("    Size in bytes: {}", staging_buffer.size_bytes());

    // Demonstrate manual copy
    info!("\nDemonstrating manual buffer copy:");
    let device_buffer = GpuBuffer::new(
        render_context.memory_allocator.clone(),
        BufferUsage::STORAGE_BUFFER,
        staging_data.len() as u64,
    )?;

    device_buffer.copy_from_staging(
        render_context.command_buffer_allocator.clone(),
        render_context.graphics_queue.clone(),
        &staging_buffer,
    )?;

    info!("  Copied {} elements from staging to device buffer", staging_data.len());

    Ok(())
}

fn demonstrate_buffer_manager(render_context: &RenderContext) -> Result<()> {
    info!("Using BufferManager...");

    let mut manager = BufferManager::new(render_context.memory_allocator.clone());
    info!("  BufferManager created (frame: {})", manager.current_frame());

    // Create buffers through manager
    let data = vec![[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
    let buffer = manager.create_buffer_from_data(
        render_context.command_buffer_allocator.clone(),
        render_context.graphics_queue.clone(),
        BufferUsage::VERTEX_BUFFER,
        &data,
    )?;

    info!("  Created buffer through manager:");
    info!("    Element count: {}", buffer.element_count());

    // Advance frame
    manager.next_frame();
    info!("  Advanced to frame {}", manager.current_frame());

    Ok(())
}

fn demonstrate_lifetime_tracking() {
    info!("Demonstrating resource lifetime tracking...");

    let mut tracker = ResourceLifetimeTracker::new(2); // 2-frame grace period
    info!("  Created tracker with 2-frame grace period");

    // Mark resources as used
    tracker.mark_used(1);
    tracker.mark_used(2);
    info!("  Marked resources 1 and 2 as used (frame {})", tracker.current_frame());

    // Check if can be freed
    info!("  Can free resource 1? {}", tracker.can_free(1));
    info!("  Can free resource 2? {}", tracker.can_free(2));

    // Advance frames
    tracker.next_frame();
    info!("  Advanced to frame {}", tracker.current_frame());
    info!("  Can free resource 1? {}", tracker.can_free(1));

    tracker.next_frame();
    info!("  Advanced to frame {}", tracker.current_frame());
    info!("  Can free resource 1? {}", tracker.can_free(1));

    tracker.next_frame();
    info!("  Advanced to frame {}", tracker.current_frame());
    info!("  Can free resource 1? {} (grace period expired)", tracker.can_free(1));
}

fn create_demo_mesh(render_context: &mut RenderContext) -> Result<()> {
    // Create a colored triangle
    let vertices = vec![
        Vertex3D::with_all([0.0, 0.5, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.5, 1.0]),
        Vertex3D::with_all([-0.5, -0.5, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [0.0, 0.0]),
        Vertex3D::with_all([0.5, -0.5, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [1.0, 0.0]),
    ];

    let indices = vec![0, 1, 2];

    let gpu_mesh = GpuMesh::new(
        render_context.memory_allocator.clone(),
        render_context.command_buffer_allocator.clone(),
        render_context.graphics_queue.clone(),
        vertices,
        indices,
    )?;

    render_context
        .mesh_manager_mut()
        .register_mesh("demo_triangle", gpu_mesh);

    Ok(())
}
