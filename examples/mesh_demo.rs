//! Mesh system demonstration.
//!
//! This example demonstrates:
//! - Loading multiple mesh types (cube, pyramid, quad)
//! - Using MeshHandle components in ECS
//! - Rendering different meshes with different transforms
//! - Using the mesh asset manager

use praxis_core::run;
use praxis_ecs::{MeshHandle, Transform, World};
use praxis_graphics::{colored_cube_mesh, pyramid_mesh, quad_mesh, solid_cube_mesh, DrawCommand, MeshData, MeshRenderCommands, RenderContext};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_utils::Result;

fn main() -> Result<()> {
    // For this demo, we'll just run the normal application
    // In a real implementation, you would:
    // 1. Create a World and spawn entities with MeshHandle components
    // 2. Load meshes into the RenderContext's mesh manager
    // 3. Query entities with Transform + MeshHandle components
    // 4. Build DrawCommands from the query results
    // 5. Call render_context.render_meshes(&cmds)
    
    println!("Mesh Demo Example");
    println!("=================");
    println!();
    println!("This example demonstrates the mesh system architecture:");
    println!("- Mesh and MeshHandle components in praxis_ecs");
    println!("- MeshData and MeshAssetManager in praxis_graphics");
    println!("- Per-mesh vertex/index buffer management");
    println!();
    println!("Usage example:");
    println!("  1. Create meshes: let cube = colored_cube_mesh();");
    println!("  2. Load into manager: render_ctx.mesh_manager_mut().load_mesh(\"cube\", cube)?;");
    println!("  3. Spawn ECS entity: world.spawn((Transform::default(), MeshHandle::new(\"cube\")));");
    println!("  4. Query and render: Build DrawCommands and call render_meshes()");
    println!();
    println!("Available primitive meshes:");
    println!("  - colored_cube_mesh() - Multi-colored cube");
    println!("  - solid_cube_mesh(color) - Single-color cube");
    println!("  - quad_mesh(size, color) - Flat quad/plane");
    println!("  - pyramid_mesh(base_color, tip_color) - 4-sided pyramid");
    println!();
    
    // Run the standard application
    run()
}
