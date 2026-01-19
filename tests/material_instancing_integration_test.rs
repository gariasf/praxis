//! Integration test for material instancing rendering.
//!
//! This test validates the complete material instancing pipeline:
//! - Creates a base material with shared textures
//! - Creates 50 material instances with different property overrides
//! - Renders the scene with all instances
//! - Verifies all instances share the same base textures
//! - Validates property overrides are applied correctly
//! - Checks descriptor set pooling efficiency
//!
//! # Requirements
//!
//! These tests require:
//! - Vulkan-capable GPU and drivers
//! - CMake (for shader compilation via vulkano-shaders)
//!
//! To install CMake:
//! - Windows: `winget install Kitware.CMake` or download from https://cmake.org/download/
//! - Linux: `sudo apt install cmake` or equivalent
//! - macOS: `brew install cmake`

use praxis_graphics::{
    colored_cube_mesh, DrawCommand, MaterialProperties, RenderCommands, RenderContext,
};
use praxis_math::{Mat4, Vec3};
use praxis_utils::{debug, info, Result};
use std::sync::Arc;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

/// Creates a headless window for testing (no actual display).
fn create_test_window() -> (Arc<winit::window::Window>, EventLoop<()>) {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = WindowBuilder::new()
        .with_title("Material Instancing Integration Test")
        .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
        .with_visible(false) // Hidden window for testing
        .build(&event_loop)
        .expect("Failed to create window");

    (Arc::new(window), event_loop)
}

/// Test fixture for material instancing tests with render context.
struct MaterialInstancingTestFixture {
    render_context: RenderContext,
}

impl MaterialInstancingTestFixture {
    /// Creates a new test fixture with a render context.
    fn new() -> Result<Self> {
        info!("Initializing material instancing integration test fixture");

        let (window, _event_loop) = create_test_window();
        let render_context = pollster::block_on(RenderContext::new(window))?;

        info!("Material instancing test fixture initialized successfully");

        Ok(Self { render_context })
    }

    /// Gets mutable reference to render context.
    fn context_mut(&mut self) -> &mut RenderContext {
        &mut self.render_context
    }

    /// Gets immutable reference to render context.
    fn context(&self) -> &RenderContext {
        &self.render_context
    }
}

#[test]
fn test_material_instancing_rendering() {
    praxis_utils::init_logging();
    info!("Starting material instancing rendering integration test");

    match run_material_instancing_test() {
        Ok(_) => {
            info!("Material instancing rendering test PASSED");
        }
        Err(e) => {
            panic!("Material instancing rendering test FAILED: {}", e);
        }
    }
}

fn run_material_instancing_test() -> Result<()> {
    // Initialize test fixture
    let mut fixture = MaterialInstancingTestFixture::new()?;
    let ctx = fixture.context_mut();

    // Step 1: Create base material
    info!("Step 1: Creating base material");

    // Load cube mesh
    ctx.mesh_manager_mut()
        .load_mesh("test_cube", colored_cube_mesh())?;

    // Get default white texture (created by RenderContext initialization)
    let white_texture = ctx
        .texture_manager()
        .get_texture("_default_white")
        .ok_or_else(|| praxis_utils::eyre::eyre!("Default white texture not found"))?
        .clone();

    // Create base material
    let base_material_id = "test_base_material";
    ctx.material_manager_mut()
        .create_material(base_material_id, white_texture.clone());

    info!(
        "Base material '{}' created with default white texture",
        base_material_id
    );

    // Step 2: Create 50 material instances with different properties
    info!("Step 2: Creating 50 material instances with property overrides");

    let num_instances = 50;
    let mut instance_ids = Vec::new();

    for i in 0..num_instances {
        let instance_id = format!("instance_{}", i);

        // Vary metallic and roughness properties
        let metallic = (i as f32 / num_instances as f32) * 0.9 + 0.1; // Range: 0.1 to 1.0
        let roughness = 1.0 - (i as f32 / num_instances as f32) * 0.8; // Range: 1.0 to 0.2

        // Vary color using HSV to RGB conversion
        let hue = (i as f32 / num_instances as f32) * 360.0;
        let color = hsv_to_rgb(hue, 0.8, 0.9);

        // Create instance with property overrides
        ctx.create_material_instance(&instance_id, base_material_id)?
            .override_properties(
                MaterialProperties::new()
                    .with_base_color([color.0, color.1, color.2, 1.0])
                    .with_metallic(metallic)
                    .with_roughness(roughness)
                    .with_emissive_strength(if i % 5 == 0 { 0.2 } else { 0.0 }),
            );

        instance_ids.push(instance_id);
    }

    info!("Created {} material instances", instance_ids.len());

    // Step 3: Verify instancing statistics
    info!("Step 3: Verifying material instancing statistics");

    let stats = ctx.material_instance_stats();

    assert_eq!(
        stats.total_instances, num_instances,
        "Total instances should be {}",
        num_instances
    );
    assert_eq!(
        stats.unique_base_materials, 1,
        "Should have exactly 1 unique base material"
    );
    assert_eq!(
        stats.instances_with_overrides, num_instances,
        "All instances should have property overrides"
    );
    assert_eq!(
        stats.avg_instances_per_base, num_instances as f32,
        "Average instances per base should be {}",
        num_instances
    );

    info!("Material Instancing Statistics:");
    info!("  Total instances: {}", stats.total_instances);
    info!("  Unique base materials: {}", stats.unique_base_materials);
    info!(
        "  Instances with overrides: {}",
        stats.instances_with_overrides
    );
    info!(
        "  Avg instances per base: {:.2}",
        stats.avg_instances_per_base
    );

    // Step 4: Verify all instances share the same base texture
    info!("Step 4: Verifying texture sharing");

    // Get base material to check its texture
    let base_material = ctx
        .material_manager()
        .get_material(base_material_id)
        .ok_or_else(|| praxis_utils::eyre::eyre!("Base material not found"))?;

    let base_texture_ptr = Arc::as_ptr(&base_material.albedo_texture().image);

    // Verify each instance shares the same base texture
    for instance_id in &instance_ids {
        let instance = ctx
            .material_instance_manager()
            .get_instance(instance_id)
            .ok_or_else(|| praxis_utils::eyre::eyre!("Instance '{}' not found", instance_id))?;

        let instance_texture_ptr = Arc::as_ptr(&instance.base_material().albedo_texture().image);

        assert_eq!(
            base_texture_ptr, instance_texture_ptr,
            "Instance '{}' should share the same base texture",
            instance_id
        );
    }

    info!(
        "All {} instances share the same base texture ✓",
        num_instances
    );

    // Step 5: Verify property overrides are applied correctly
    info!("Step 5: Verifying property overrides");

    for (i, instance_id) in instance_ids.iter().enumerate() {
        let instance = ctx
            .material_instance_manager()
            .get_instance(instance_id)
            .ok_or_else(|| praxis_utils::eyre::eyre!("Instance '{}' not found", instance_id))?;

        // Verify the instance has overrides
        assert!(
            instance.has_overrides(),
            "Instance '{}' should have property overrides",
            instance_id
        );

        // Verify specific properties
        let props = instance.properties();
        let expected_metallic = (i as f32 / num_instances as f32) * 0.9 + 0.1;
        let expected_roughness = 1.0 - (i as f32 / num_instances as f32) * 0.8;

        assert!(
            (props.metallic - expected_metallic).abs() < 0.001,
            "Instance '{}' metallic mismatch: expected {}, got {}",
            instance_id,
            expected_metallic,
            props.metallic
        );

        assert!(
            (props.roughness - expected_roughness).abs() < 0.001,
            "Instance '{}' roughness mismatch: expected {}, got {}",
            instance_id,
            expected_roughness,
            props.roughness
        );

        // Verify emissive strength
        let expected_emissive = if i % 5 == 0 { 0.2 } else { 0.0 };
        assert!(
            (props.emissive_strength - expected_emissive).abs() < 0.001,
            "Instance '{}' emissive_strength mismatch: expected {}, got {}",
            instance_id,
            expected_emissive,
            props.emissive_strength
        );
    }

    info!("All property overrides verified correctly ✓");

    // Step 6: Render scene with all instances
    info!("Step 6: Rendering scene with all material instances");

    // Create draw commands for all instances
    let mut draw_commands = Vec::new();

    // Arrange instances in a grid pattern (7x8 grid for 50 instances, with some empty spots)
    let grid_cols = 8;
    let spacing = 2.5;

    for (i, instance_id) in instance_ids.iter().enumerate() {
        let x = (i % grid_cols) as f32 - (grid_cols as f32 / 2.0);
        let z = (i / grid_cols) as f32 - 4.0;

        let position = Vec3::new(x * spacing, 0.0, z * spacing);
        let rotation_angle = (i as f32 * 0.1).sin() * 0.5;

        let model = Mat4::from_translation(position)
            * Mat4::from_rotation_y(rotation_angle)
            * Mat4::from_scale(Vec3::splat(0.8));

        draw_commands.push(DrawCommand {
            mesh_id: "test_cube".to_string(),
            model,
            texture_name: None,
            material_properties: None,
            material_instance_id: Some(instance_id.clone()),
            bone_matrices: None,
        });
    }

    // Set up camera matrices
    let eye = Vec3::new(0.0, 10.0, 20.0);
    let target = Vec3::new(0.0, 0.0, 0.0);
    let up = Vec3::new(0.0, 1.0, 0.0);
    let view = Mat4::look_at_rh(eye, target, up);

    let aspect_ratio = 800.0 / 600.0;
    let proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect_ratio, 0.1, 1000.0);

    // Render the scene
    let render_commands = RenderCommands {
        view,
        proj,
        draw_commands: &draw_commands,
        lighting: None,
    };

    // Record descriptor set pool size before rendering
    let descriptor_pool_size_before = ctx.descriptor_set_pool_size();
    debug!(
        "Descriptor set pool size before rendering: {}",
        descriptor_pool_size_before
    );

    ctx.render(&render_commands)?;

    info!(
        "Scene rendered successfully with {} draw commands",
        draw_commands.len()
    );

    // Step 7: Verify descriptor set pooling
    info!("Step 7: Verifying descriptor set pooling efficiency");

    let descriptor_pool_size_after = ctx.descriptor_set_pool_size();
    debug!(
        "Descriptor set pool size after rendering: {}",
        descriptor_pool_size_after
    );

    // With material instancing, we should have efficient descriptor set reuse.
    // All instances share the same base material texture, so we expect:
    // - Transform descriptor sets: one per unique texture (should be 1 for default white)
    // - Material descriptor sets: varies by property bytes, but with batching should be efficient

    // The pool should contain descriptor sets, but not 50 separate sets per instance
    // since instances can share descriptor sets when properties match
    assert!(
        descriptor_pool_size_after > 0,
        "Descriptor set pool should contain cached sets after rendering"
    );

    // Since we have 50 unique property combinations, we expect significantly fewer
    // descriptor sets than traditional rendering (which would need 50 full materials)
    info!(
        "Descriptor set pool contains {} cached sets for {} instances",
        descriptor_pool_size_after, num_instances
    );

    // Pool should be reasonably sized (not unbounded growth)
    assert!(
        descriptor_pool_size_after < num_instances * 3,
        "Descriptor set pool should not grow unbounded (found {} sets for {} instances)",
        descriptor_pool_size_after,
        num_instances
    );

    info!("Descriptor set pooling verified ✓");

    // Step 8: Render multiple frames to verify descriptor set reuse
    info!("Step 8: Rendering multiple frames to verify descriptor set reuse");

    for frame in 1..=5 {
        ctx.render(&render_commands)?;

        let pool_size = ctx.descriptor_set_pool_size();
        debug!("Frame {}: Descriptor set pool size = {}", frame, pool_size);

        // Pool size should stabilize after first frame
        if frame > 1 {
            assert_eq!(
                pool_size, descriptor_pool_size_after,
                "Descriptor set pool size should remain stable across frames"
            );
        }
    }

    info!("Multiple frame rendering verified - descriptor sets are being reused ✓");

    // Step 9: Test instance property modification
    info!("Step 9: Testing material instance property modification");

    let test_instance_id = &instance_ids[0];
    let instance = ctx
        .material_instance_manager_mut()
        .get_instance_mut(test_instance_id)
        .ok_or_else(|| praxis_utils::eyre::eyre!("Instance '{}' not found", test_instance_id))?;

    // Modify properties
    *instance = instance.clone().override_properties(
        MaterialProperties::new()
            .with_base_color([1.0, 0.0, 0.0, 1.0])
            .with_metallic(0.95)
            .with_roughness(0.05),
    );

    // Verify modification
    let modified_instance = ctx
        .material_instance_manager()
        .get_instance(test_instance_id)
        .ok_or_else(|| praxis_utils::eyre::eyre!("Instance '{}' not found", test_instance_id))?;

    let modified_props = modified_instance.properties();
    assert_eq!(modified_props.base_color, [1.0, 0.0, 0.0, 1.0]);
    assert!((modified_props.metallic - 0.95).abs() < 0.001);
    assert!((modified_props.roughness - 0.05).abs() < 0.001);

    info!("Material instance property modification verified ✓");

    // Step 10: Test instance removal
    info!("Step 10: Testing material instance removal");

    let remove_instance_id = instance_ids.last().unwrap();
    let removed = ctx
        .material_instance_manager_mut()
        .remove_instance(remove_instance_id);

    assert!(removed, "Should successfully remove instance");
    assert!(
        ctx.material_instance_manager()
            .get_instance(remove_instance_id)
            .is_none(),
        "Removed instance should not be accessible"
    );

    let updated_stats = ctx.material_instance_stats();
    assert_eq!(
        updated_stats.total_instances,
        num_instances - 1,
        "Instance count should decrease after removal"
    );

    info!("Material instance removal verified ✓");

    info!("✅ All material instancing integration tests passed!");
    Ok(())
}

/// Convert HSV color to RGB.
///
/// # Arguments
///
/// * `h` - Hue in degrees [0, 360]
/// * `s` - Saturation [0.0, 1.0]
/// * `v` - Value [0.0, 1.0]
///
/// # Returns
///
/// RGB color as (r, g, b) tuple [0.0, 1.0]
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h_prime as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        5 => (c, 0.0, x),
        _ => (c, x, 0.0),
    };

    (r + m, g + m, b + m)
}
