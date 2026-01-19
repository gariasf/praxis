//! Integration test for descriptor set caching LRU behavior.
//!
//! This test validates that:
//! 1. Descriptor sets are cached and reused across frames (zero allocations after frame 1)
//! 2. LRU eviction removes stale descriptor sets after 60+ frames of non-use
//! 3. The descriptor set pool correctly tracks frame usage
//!
//! # Test Methodology
//!
//! The test renders the same scene for 120 frames, measuring descriptor set allocations:
//! - Frame 1: Initial allocations create descriptor sets
//! - Frames 2-60: All descriptor sets reused (zero allocations)
//! - Frame 61+: Eviction check runs but sets remain (still in use)
//! - Then, render a different scene to make original sets stale
//! - After 60 more frames with different scene: Original sets evicted
//!
//! # Requirements
//!
//! - Vulkan-capable GPU and drivers
//! - Window system for swapchain creation

use praxis_graphics::{colored_cube_mesh, DrawCommand, RenderCommands, RenderContext};
use praxis_math::{Mat4, Vec3};
use praxis_utils::{debug, info, Result};
use std::sync::Arc;
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

/// Test fixture with window and render context.
struct DescriptorCacheTestFixture {
    _event_loop: EventLoop<()>,
    _window: Arc<Window>,
    render_context: RenderContext,
}

impl DescriptorCacheTestFixture {
    /// Creates a new test fixture with window and render context.
    async fn new() -> Result<Self> {
        info!("Initializing descriptor cache LRU test fixture");

        let event_loop = EventLoop::new()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

        let window = Arc::new(
            WindowBuilder::new()
                .with_title("Descriptor Cache LRU Test")
                .with_inner_size(LogicalSize::new(800, 600))
                .with_visible(false)
                .build(&event_loop)
                .map_err(|e| praxis_utils::eyre::eyre!("Failed to create window: {}", e))?,
        );

        let render_context = RenderContext::new(window.clone()).await?;

        info!("Test fixture initialized successfully");

        Ok(Self {
            _event_loop: event_loop,
            _window: window,
            render_context,
        })
    }
}

/// Creates a simple test scene with multiple objects sharing materials.
fn create_test_scene(offset: f32) -> Vec<DrawCommand> {
    let mut commands = Vec::new();

    // Create 10 objects in a line, using 2 different textures (5 objects per texture)
    // This should result in 2 transform descriptor sets and 2 material descriptor sets
    for i in 0..10 {
        let texture_name = if i < 5 {
            Some("texture_a".to_string())
        } else {
            Some("texture_b".to_string())
        };

        commands.push(DrawCommand {
            mesh_id: "cube".to_string(),
            model: Mat4::from_translation(Vec3::new(i as f32 * 3.0 + offset, 0.0, 0.0)),
            texture_name,
            material_properties: None,
            material_instance_id: None,
            bone_matrices: None,
        });
    }

    commands
}

/// Creates a different test scene to make the first scene's descriptor sets stale.
fn create_alternate_scene() -> Vec<DrawCommand> {
    let mut commands = Vec::new();

    // Create 10 objects with different textures (texture_c and texture_d)
    for i in 0..10 {
        let texture_name = if i < 5 {
            Some("texture_c".to_string())
        } else {
            Some("texture_d".to_string())
        };

        commands.push(DrawCommand {
            mesh_id: "cube".to_string(),
            model: Mat4::from_translation(Vec3::new(i as f32 * 3.0, 2.0, 0.0)),
            texture_name,
            material_properties: None,
            material_instance_id: None,
            bone_matrices: None,
        });
    }

    commands
}

/// Main integration test: Descriptor set caching with LRU eviction.
#[tokio::test]
async fn test_descriptor_cache_lru_behavior() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== Descriptor Set Cache LRU Integration Test ===");

    let mut fixture = DescriptorCacheTestFixture::new().await?;

    // Load test mesh
    info!("Loading test mesh");
    fixture
        .render_context
        .mesh_manager_mut()
        .load_mesh("cube", colored_cube_mesh())?;

    // Create test textures
    info!("Creating test textures");
    let texture_manager = fixture.render_context.texture_manager_mut();
    for texture_name in &["texture_a", "texture_b", "texture_c", "texture_d"] {
        // Create a simple 2x2 texture
        let data = vec![255u8; 4 * 4]; // RGBA white texture
        texture_manager.load_texture_from_bytes(texture_name, &data, 2, 2)?;
    }

    // Set up camera
    let view = Mat4::look_at_rh(
        Vec3::new(15.0, 10.0, 30.0),
        Vec3::new(15.0, 0.0, 0.0),
        Vec3::Y,
    );
    let projection = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 800.0 / 600.0, 0.1, 100.0);

    info!("Starting frame rendering loop");

    // Phase 1: Render same scene for 120 frames
    info!("\n=== Phase 1: Render same scene for 120 frames ===");

    let scene = create_test_scene(0.0);
    let mut frame1_pool_size = 0;
    let mut last_pool_size = 0;

    for frame in 0..120 {
        let cmds = RenderCommands {
            view,
            proj: projection,
            draw_commands: &scene,
            lighting: None,
        };

        fixture.render_context.render(&cmds)?;

        let pool_size = fixture.render_context.descriptor_set_pool_size();
        let frame_number = fixture.render_context.descriptor_set_pool_frame();

        if frame == 0 {
            frame1_pool_size = pool_size;
            info!(
                "Frame {}: Initial descriptor set pool size = {} (expected 4: 2 transform + 2 material)",
                frame + 1,
                pool_size
            );
        } else if frame % 20 == 0 || frame == 119 {
            debug!(
                "Frame {}: Pool size = {}, Frame number = {}",
                frame + 1,
                pool_size,
                frame_number
            );
        }

        last_pool_size = pool_size;
    }

    // Validation 1: Frame 1 should allocate descriptor sets
    info!("\n=== Validation 1: Initial Allocations ===");
    assert_eq!(
        frame1_pool_size, 4,
        "Frame 1 should allocate 4 descriptor sets (2 transform + 2 material), got {}",
        frame1_pool_size
    );
    info!("✓ Frame 1 allocated {} descriptor sets", frame1_pool_size);

    // Validation 2: Pool size should remain constant (reuse, no new allocations)
    info!("\n=== Validation 2: Descriptor Set Reuse ===");
    assert_eq!(
        last_pool_size, frame1_pool_size,
        "Pool size should remain constant after frame 1 (reuse), but changed from {} to {}",
        frame1_pool_size, last_pool_size
    );
    info!(
        "✓ Pool size remained constant at {} for 120 frames (100% reuse)",
        last_pool_size
    );

    // Phase 2: Render different scene for 70 frames to make original sets stale
    info!("\n=== Phase 2: Render different scene for 70 frames ===");
    info!("Switching to alternate scene to make original descriptor sets stale");

    let alternate_scene = create_alternate_scene();

    for frame in 120..190 {
        let cmds = RenderCommands {
            view,
            proj: projection,
            draw_commands: &alternate_scene,
            lighting: None,
        };

        fixture.render_context.render(&cmds)?;

        let pool_size = fixture.render_context.descriptor_set_pool_size();
        let frame_number = fixture.render_context.descriptor_set_pool_frame();

        if frame == 120 {
            info!(
                "Frame {}: Switched to alternate scene, pool size = {} (added 4 new sets for new textures)",
                frame + 1,
                pool_size
            );
        } else if frame % 20 == 0 || frame == 189 {
            debug!(
                "Frame {}: Pool size = {}, Frame number = {}",
                frame + 1,
                pool_size,
                frame_number
            );
        }
    }

    // Check pool size after 70 frames with alternate scene
    let pool_size_before_eviction = fixture.render_context.descriptor_set_pool_size();
    info!(
        "Pool size after 70 frames with alternate scene: {}",
        pool_size_before_eviction
    );

    // Validation 3: Original descriptor sets should be evicted
    info!("\n=== Validation 3: LRU Eviction ===");
    info!("Original descriptor sets (texture_a, texture_b) should be evicted after 60+ frames of non-use");

    // The pool should now only contain descriptor sets for the alternate scene
    // At frame 180 (60 frames after last use), eviction runs and removes stale sets
    // We need to continue rendering to trigger the eviction check
    for frame in 190..200 {
        let cmds = RenderCommands {
            view,
            proj: projection,
            draw_commands: &alternate_scene,
            lighting: None,
        };

        fixture.render_context.render(&cmds)?;

        if frame % 60 == 0 {
            let pool_size = fixture.render_context.descriptor_set_pool_size();
            debug!(
                "Frame {}: Pool size after eviction check = {}",
                frame + 1,
                pool_size
            );
        }
    }

    let pool_size_after_eviction = fixture.render_context.descriptor_set_pool_size();
    info!(
        "Pool size after eviction: {} (down from {})",
        pool_size_after_eviction, pool_size_before_eviction
    );

    // The original 4 descriptor sets should have been evicted, leaving only 4 for the alternate scene
    assert!(
        pool_size_after_eviction <= 4,
        "Expected pool size <= 4 after eviction (only alternate scene sets), got {}",
        pool_size_after_eviction
    );
    info!(
        "✓ Stale descriptor sets evicted, pool size reduced to {}",
        pool_size_after_eviction
    );

    // Phase 3: Verify original scene causes re-allocation
    info!("\n=== Phase 3: Verify Re-allocation After Eviction ===");
    info!("Rendering original scene again should re-allocate descriptor sets");

    let cmds = RenderCommands {
        view,
        proj: projection,
        draw_commands: &scene,
        lighting: None,
    };
    fixture.render_context.render(&cmds)?;

    let pool_size_after_realloc = fixture.render_context.descriptor_set_pool_size();
    info!(
        "Pool size after re-rendering original scene: {}",
        pool_size_after_realloc
    );

    // Should now have descriptor sets for both scenes
    assert!(
        pool_size_after_realloc >= pool_size_after_eviction,
        "Pool size should increase or stay the same after re-rendering original scene"
    );
    info!(
        "✓ Original scene descriptor sets re-allocated, pool size = {}",
        pool_size_after_realloc
    );

    info!("\n=== Descriptor Set Cache LRU Test PASSED ===");
    info!("Summary:");
    info!(
        "  ✓ Frame 1: {} descriptor sets allocated",
        frame1_pool_size
    );
    info!("  ✓ Frames 2-120: 100% descriptor set reuse (zero new allocations)");
    info!(
        "  ✓ After 60+ frames of non-use: {} stale sets evicted",
        pool_size_before_eviction - pool_size_after_eviction
    );
    info!("  ✓ Re-allocation works correctly after eviction");

    Ok(())
}

/// Test that verifies eviction threshold can be configured.
#[tokio::test]
async fn test_descriptor_cache_configurable_eviction() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== Configurable Eviction Threshold Test ===");

    let mut fixture = DescriptorCacheTestFixture::new().await?;

    // Load test resources
    fixture
        .render_context
        .mesh_manager_mut()
        .load_mesh("cube", colored_cube_mesh())?;

    let texture_manager = fixture.render_context.texture_manager_mut();
    for texture_name in &["texture_a", "texture_b"] {
        let data = vec![255u8; 4 * 4];
        texture_manager.load_texture_from_bytes(texture_name, &data, 2, 2)?;
    }

    // Set a shorter eviction threshold (30 frames instead of default 60)
    info!("Setting eviction threshold to 30 frames");
    let original_threshold = fixture
        .render_context
        .descriptor_set_pool_eviction_threshold();
    fixture
        .render_context
        .set_descriptor_set_pool_eviction_threshold(30);

    let new_threshold = fixture
        .render_context
        .descriptor_set_pool_eviction_threshold();
    assert_eq!(
        new_threshold, 30,
        "Eviction threshold should be 30, got {}",
        new_threshold
    );
    info!(
        "✓ Eviction threshold changed from {} to {}",
        original_threshold, new_threshold
    );

    info!("✓ Eviction threshold configuration test passed");

    Ok(())
}

/// Test that verifies manual pool clearing works.
#[tokio::test]
async fn test_descriptor_cache_manual_clear() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== Manual Pool Clear Test ===");

    let mut fixture = DescriptorCacheTestFixture::new().await?;

    // Load test resources
    fixture
        .render_context
        .mesh_manager_mut()
        .load_mesh("cube", colored_cube_mesh())?;

    let texture_manager = fixture.render_context.texture_manager_mut();
    let data = vec![255u8; 4 * 4];
    texture_manager.load_texture_from_bytes("texture_a", &data, 2, 2)?;

    // Render to allocate descriptor sets
    let view = Mat4::IDENTITY;
    let projection = Mat4::IDENTITY;
    let scene = create_test_scene(0.0);

    let cmds = RenderCommands {
        view,
        proj: projection,
        draw_commands: &scene[..2], // Just 2 objects
        lighting: None,
    };

    fixture.render_context.render(&cmds)?;

    let pool_size_before = fixture.render_context.descriptor_set_pool_size();
    info!("Pool size before clear: {}", pool_size_before);
    assert!(pool_size_before > 0, "Pool should have descriptor sets");

    // Clear the pool
    info!("Clearing descriptor set pool");
    fixture.render_context.clear_descriptor_set_pool();

    let pool_size_after = fixture.render_context.descriptor_set_pool_size();
    info!("Pool size after clear: {}", pool_size_after);

    assert_eq!(
        pool_size_after, 0,
        "Pool should be empty after clear, got {} sets",
        pool_size_after
    );
    info!("✓ Descriptor set pool cleared successfully");

    // Render again to verify re-allocation
    fixture.render_context.render(&cmds)?;

    let pool_size_realloc = fixture.render_context.descriptor_set_pool_size();
    info!("Pool size after re-render: {}", pool_size_realloc);

    assert!(
        pool_size_realloc > 0,
        "Pool should have descriptor sets after re-render"
    );
    info!(
        "✓ Descriptor sets re-allocated correctly ({} sets)",
        pool_size_realloc
    );

    Ok(())
}
