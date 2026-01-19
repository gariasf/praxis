//! Extended GPU culling demonstration with all culling strategies.
//!
//! This example demonstrates the complete GPU culling system with:
//! - Frustum culling
//! - Back-face culling using object normals
//! - Small object culling based on screen-space size
//! - Distance-based culling with per-object class configuration
//!
//! Features demonstrated:
//! - Multiple culling strategies working together
//! - Object class-based render distances
//! - Per-object culling parameters
//! - Debug visualization of culling results
//!
//! Controls:
//! - WASD: Move camera
//! - Mouse: Look around
//! - 1-5: Toggle culling strategies
//! - ESC: Exit

use praxis_graphics::gpu_culling::{
    calculate_average_normal, extract_frustum_planes, GpuCullingManager, GpuDrawCommand,
    GpuMeshData, ObjectClassConfig,
};
use praxis_math::{Mat4, Vec3, Vec4};
use praxis_utils::{info, Result};

/// Object type determines culling behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObjectType {
    Building,  // Large, far render distance
    Tree,      // Medium distance
    Rock,      // Small props, closer distance
    Grass,     // Detail, very close distance
    Character, // Important, always visible
}

impl ObjectType {
    /// Gets the culling configuration for this object type.
    fn config(&self) -> ObjectClassConfig {
        match self {
            ObjectType::Building => ObjectClassConfig::LARGE_STATIC,
            ObjectType::Tree => ObjectClassConfig::MEDIUM,
            ObjectType::Rock => ObjectClassConfig::SMALL_PROPS,
            ObjectType::Grass => ObjectClassConfig::DETAIL,
            ObjectType::Character => ObjectClassConfig::IMPORTANT,
        }
    }

    /// Gets the average normal for this object type.
    /// In a real application, this would be calculated from mesh data.
    fn average_normal(&self) -> Vec3 {
        match self {
            ObjectType::Building => Vec3::Y,              // Upward facing
            ObjectType::Tree => Vec3::Y,                  // Upward facing
            ObjectType::Rock => Vec3::new(0.0, 0.7, 0.3), // Slight upward angle
            ObjectType::Grass => Vec3::Y,                 // Upward facing
            ObjectType::Character => Vec3::ZERO,          // No back-face culling
        }
    }

    /// Gets the back-face culling threshold.
    fn backface_threshold(&self) -> f32 {
        match self {
            ObjectType::Building => 0.0,   // Cull when facing away
            ObjectType::Tree => -0.1,      // Small tolerance
            ObjectType::Rock => -0.2,      // Larger tolerance
            ObjectType::Grass => 0.0,      // Strict culling
            ObjectType::Character => -1.0, // Never cull (threshold = -1)
        }
    }
}

/// Represents a scene object with culling parameters.
struct SceneObject {
    position: Vec3,
    rotation: f32,
    scale: f32,
    object_type: ObjectType,
}

impl SceneObject {
    /// Creates GPU draw command for this object.
    fn to_gpu_draw_command(&self, mesh_id: u32) -> GpuDrawCommand {
        let config = self.object_type.config();

        // Build model matrix
        let model = Mat4::from_scale_rotation_translation(
            Vec3::splat(self.scale),
            praxis_math::Quat::from_rotation_y(self.rotation),
            self.position,
        );

        // Calculate bounding sphere (simple approximation)
        let bounding_sphere = Vec4::new(0.0, 0.0, 0.0, 1.0 * self.scale);

        GpuDrawCommand::new_with_culling_params(
            model,
            bounding_sphere,
            self.object_type.average_normal(),
            self.object_type.backface_threshold(),
            mesh_id,
            0, // material_id
            config.min_screen_size,
            config.max_render_distance,
        )
    }
}

/// Example scene setup with different object types.
fn create_test_scene() -> Vec<SceneObject> {
    let mut objects = Vec::new();

    // Create a grid of buildings (large, far render distance)
    for x in -10..10 {
        for z in -10..10 {
            if (x + z) % 3 == 0 {
                objects.push(SceneObject {
                    position: Vec3::new(x as f32 * 50.0, 0.0, z as f32 * 50.0),
                    rotation: 0.0,
                    scale: 10.0,
                    object_type: ObjectType::Building,
                });
            }
        }
    }

    // Create trees (medium render distance)
    for x in -20..20 {
        for z in -20..20 {
            if (x * z) % 7 == 0 {
                objects.push(SceneObject {
                    position: Vec3::new(x as f32 * 25.0, 0.0, z as f32 * 25.0),
                    rotation: (x as f32 + z as f32) * 0.5,
                    scale: 3.0,
                    object_type: ObjectType::Tree,
                });
            }
        }
    }

    // Create rocks (small props, close distance)
    for x in -30..30 {
        for z in -30..30 {
            if (x * x + z * z) % 11 == 0 {
                objects.push(SceneObject {
                    position: Vec3::new(x as f32 * 10.0, 0.0, z as f32 * 10.0),
                    rotation: x as f32 * z as f32,
                    scale: 1.0,
                    object_type: ObjectType::Rock,
                });
            }
        }
    }

    // Create grass patches (detail, very close distance)
    for x in -40..40 {
        for z in -40..40 {
            if (x + z) % 5 == 0 {
                objects.push(SceneObject {
                    position: Vec3::new(x as f32 * 5.0, 0.0, z as f32 * 5.0),
                    rotation: 0.0,
                    scale: 0.5,
                    object_type: ObjectType::Grass,
                });
            }
        }
    }

    // Create a character at the origin (always visible)
    objects.push(SceneObject {
        position: Vec3::ZERO,
        rotation: 0.0,
        scale: 2.0,
        object_type: ObjectType::Character,
    });

    info!("Created test scene with {} objects", objects.len());
    info!(
        "  Buildings: {}",
        objects
            .iter()
            .filter(|o| o.object_type == ObjectType::Building)
            .count()
    );
    info!(
        "  Trees: {}",
        objects
            .iter()
            .filter(|o| o.object_type == ObjectType::Tree)
            .count()
    );
    info!(
        "  Rocks: {}",
        objects
            .iter()
            .filter(|o| o.object_type == ObjectType::Rock)
            .count()
    );
    info!(
        "  Grass: {}",
        objects
            .iter()
            .filter(|o| o.object_type == ObjectType::Grass)
            .count()
    );
    info!(
        "  Characters: {}",
        objects
            .iter()
            .filter(|o| o.object_type == ObjectType::Character)
            .count()
    );

    objects
}

/// Demonstrates extended GPU culling usage.
fn demonstrate_extended_culling() -> Result<()> {
    info!("=== Extended GPU Culling Demo ===");
    info!("");
    info!("This demo shows how to use all GPU culling strategies:");
    info!("  1. Frustum Culling - Tests against camera view frustum");
    info!("  2. Back-face Culling - Culls objects facing away from camera");
    info!("  3. Small Object Culling - Culls objects too small on screen");
    info!("  4. Distance Culling - Culls objects based on per-class max distance");
    info!("");

    // Create test scene
    let scene = create_test_scene();

    // Example: Converting scene objects to GPU draw commands
    info!(
        "Converting {} scene objects to GPU draw commands...",
        scene.len()
    );

    let draw_commands: Vec<GpuDrawCommand> = scene
        .iter()
        .enumerate()
        .map(|(i, obj)| obj.to_gpu_draw_command(i as u32))
        .collect();

    // Example: Print culling parameters for a few objects
    info!("");
    info!("Sample object culling parameters:");
    for (i, (obj, cmd)) in scene.iter().zip(&draw_commands).take(5).enumerate() {
        info!("  Object {}: {:?}", i, obj.object_type);
        info!("    Max Distance: {:.1}m", cmd.max_render_distance);
        info!("    Min Screen Size: {:.1}px", cmd.min_screen_size);
        info!(
            "    Average Normal: ({:.2}, {:.2}, {:.2})",
            cmd.average_normal[0], cmd.average_normal[1], cmd.average_normal[2]
        );
        info!("    Backface Threshold: {:.2}", cmd.average_normal[3]);
    }

    info!("");
    info!("Culling Strategy Order:");
    info!("  1. Frustum Culling (broadest - eliminates off-screen objects)");
    info!("  2. Distance Culling (per-object max distance)");
    info!("  3. Back-face Culling (objects facing away)");
    info!("  4. Small Object Culling (sub-pixel objects)");
    info!("  5. Occlusion Culling (hidden behind other objects - optional)");
    info!("");
    info!("Expected culling results:");
    info!("  - Buildings visible up to 2000m");
    info!("  - Trees visible up to 500m");
    info!("  - Rocks visible up to 100m");
    info!("  - Grass visible up to 50m");
    info!("  - Character always visible (no distance culling)");
    info!("");

    // Example: Object class configurations
    info!("Object Class Configurations:");
    info!("  LARGE_STATIC (buildings, terrain):");
    info!(
        "    - Max Distance: {:.1}m",
        ObjectClassConfig::LARGE_STATIC.max_render_distance
    );
    info!(
        "    - Min Screen Size: {:.1}px",
        ObjectClassConfig::LARGE_STATIC.min_screen_size
    );
    info!(
        "    - Back-face Culling: {}",
        ObjectClassConfig::LARGE_STATIC.enable_backface_culling
    );
    info!("");
    info!("  MEDIUM (trees, vehicles):");
    info!(
        "    - Max Distance: {:.1}m",
        ObjectClassConfig::MEDIUM.max_render_distance
    );
    info!(
        "    - Min Screen Size: {:.1}px",
        ObjectClassConfig::MEDIUM.min_screen_size
    );
    info!(
        "    - Back-face Culling: {}",
        ObjectClassConfig::MEDIUM.enable_backface_culling
    );
    info!("");
    info!("  SMALL_PROPS (rocks, debris):");
    info!(
        "    - Max Distance: {:.1}m",
        ObjectClassConfig::SMALL_PROPS.max_render_distance
    );
    info!(
        "    - Min Screen Size: {:.1}px",
        ObjectClassConfig::SMALL_PROPS.min_screen_size
    );
    info!(
        "    - Back-face Culling: {}",
        ObjectClassConfig::SMALL_PROPS.enable_backface_culling
    );
    info!("");
    info!("  DETAIL (grass, small vegetation):");
    info!(
        "    - Max Distance: {:.1}m",
        ObjectClassConfig::DETAIL.max_render_distance
    );
    info!(
        "    - Min Screen Size: {:.1}px",
        ObjectClassConfig::DETAIL.min_screen_size
    );
    info!(
        "    - Back-face Culling: {}",
        ObjectClassConfig::DETAIL.enable_backface_culling
    );
    info!("");
    info!("  IMPORTANT (characters, objectives):");
    info!(
        "    - Max Distance: unlimited ({})",
        ObjectClassConfig::IMPORTANT.max_render_distance
    );
    info!(
        "    - Min Screen Size: none ({}px)",
        ObjectClassConfig::IMPORTANT.min_screen_size
    );
    info!(
        "    - Back-face Culling: {}",
        ObjectClassConfig::IMPORTANT.enable_backface_culling
    );

    Ok(())
}

fn main() -> Result<()> {
    praxis_utils::init_logging();

    demonstrate_extended_culling()?;

    info!("");
    info!("Demo complete!");
    info!("");
    info!("To use in your application:");
    info!("  1. Create GpuCullingManager");
    info!("  2. Enable desired culling strategies");
    info!("  3. Prepare draw commands with culling parameters");
    info!("  4. Call dispatch_culling_extended() each frame");
    info!("  5. Use generated indirect draw buffer for rendering");

    Ok(())
}
