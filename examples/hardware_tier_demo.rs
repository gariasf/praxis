//! Hardware Tier Detection Demo
//!
//! This example demonstrates the hardware tier detection system that automatically
//! detects GPU capabilities and recommends quality presets.
//!
//! Features shown:
//! - GPU property detection (VRAM, vendor, device type)
//! - Hardware tier classification (Integrated/Mobile/MidRange/HighEnd)
//! - Quality preset recommendation
//! - Configuration preset application
//! - Feature support detection

use praxis_core::{Engine, EngineConfig};
use praxis_graphics::{
    colored_cube_mesh, DrawCommand, GpuVendor, HardwareTier, HardwareTierConfig,
    HardwareTierDetector, QualityPreset, RenderCommands,
};
use praxis_math::{Mat4, Vec3};
use praxis_utils::{info, Result};

fn main() -> Result<()> {
    praxis_utils::init_tracing();

    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("Hardware Tier Detection Demo");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    info!("This demo detects your GPU and recommends quality settings.");
    info!("");

    let config = EngineConfig {
        window_title: "Hardware Tier Detection Demo".to_string(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    };

    let mut engine = Engine::new(config)?;

    // Get the physical device from the render context
    let physical_device = engine.render_context().physical_device().clone();

    // Create hardware tier detector
    let detector = HardwareTierDetector::from_physical_device(&physical_device);

    // Display detailed hardware information
    info!("");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("GPU HARDWARE DETECTION");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    info!("{}", detector.summary());
    info!("");

    // Get recommended preset
    let preset = detector.recommended_preset();
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("RECOMMENDED QUALITY PRESET: {}", preset.name());
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");

    // Get configuration for the recommended preset
    let config = HardwareTierConfig::from_preset(preset);
    info!("{}", config.summary());
    info!("");

    // Show feature support based on tier
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("FEATURE SUPPORT");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    info!(
        "  Ray Tracing:             {}",
        if detector.supports_feature("ray_tracing") {
            "✓ Supported"
        } else {
            "✗ Not recommended"
        }
    );
    info!(
        "  Mesh Shaders:            {}",
        if detector.supports_feature("mesh_shaders") {
            "✓ Supported"
        } else {
            "✗ Not recommended"
        }
    );
    info!(
        "  Variable Rate Shading:   {}",
        if detector.supports_feature("variable_rate_shading") {
            "✓ Supported"
        } else {
            "✗ Not recommended"
        }
    );
    info!(
        "  Hi-Z Occlusion:          {}",
        if detector.supports_feature("hiz_occlusion") {
            "✓ Supported"
        } else {
            "✗ Not recommended"
        }
    );
    info!(
        "  Advanced Post-Processing: {}",
        if detector.supports_feature("advanced_post_processing") {
            "✓ Supported"
        } else {
            "✗ Not recommended"
        }
    );
    info!("");

    // Show tier-specific recommendations
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("TIER-SPECIFIC RECOMMENDATIONS");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");

    match detector.get_tier() {
        HardwareTier::Integrated => {
            info!("  Your integrated GPU is best suited for:");
            info!("  • Simple scenes with few objects");
            info!("  • Low-poly models and simplified shaders");
            info!("  • Minimal post-processing effects");
            info!("  • Target 30 FPS for smooth gameplay");
            info!("");
            info!("  Consider:");
            info!("  • Disabling shadows or using low-res shadow maps");
            info!("  • Using aggressive LOD bias (-0.5)");
            info!("  • Reducing view distance by 50%");
        }
        HardwareTier::Mobile => {
            info!("  Your mobile/laptop GPU can handle:");
            info!("  • Moderate scene complexity");
            info!("  • Basic post-processing (bloom, basic SSAO)");
            info!("  • Medium-quality shadows");
            info!("  • Target 60 FPS at 1080p");
            info!("");
            info!("  Consider:");
            info!("  • Using 1024x1024 shadow maps");
            info!("  • Enabling basic mesh streaming");
            info!("  • 2x MSAA for anti-aliasing");
        }
        HardwareTier::MidRange => {
            info!("  Your mid-range GPU is capable of:");
            info!("  • Complex scenes with many objects");
            info!("  • Most post-processing effects");
            info!("  • High-quality shadows with soft edges");
            info!("  • Target 60 FPS at 1440p");
            info!("");
            info!("  Consider:");
            info!("  • Using 2048x2048 shadow maps with 4 cascades");
            info!("  • Enabling Hi-Z occlusion culling");
            info!("  • SSAO with 32 samples");
            info!("  • Screen-space reflections");
        }
        HardwareTier::HighEnd => {
            info!("  Your high-end GPU can maximize quality:");
            info!("  • Very complex scenes (10,000+ objects)");
            info!("  • All post-processing effects enabled");
            info!("  • Ultra-quality shadows");
            info!("  • Target 60+ FPS at 4K");
            info!("");
            info!("  Consider:");
            info!("  • Using 4096x4096 shadow maps");
            info!("  • Maximum LOD detail (+0.5 bias)");
            info!("  • SSAO with 64 samples");
            info!("  • 8x MSAA or TAA for anti-aliasing");
            info!("  • Advanced effects like SSR and environment probes");
        }
    }
    info!("");

    // Show vendor-specific notes
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("VENDOR-SPECIFIC NOTES");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");

    match detector.vendor {
        GpuVendor::Nvidia => {
            info!("  NVIDIA GPU detected:");
            info!("  • Excellent Vulkan support and performance");
            info!("  • Strong compute shader performance for GPU culling");
            info!("  • Good texture compression support");
            info!("  • Ray tracing available on RTX series");
        }
        GpuVendor::Amd => {
            info!("  AMD GPU detected:");
            info!("  • Excellent Vulkan support");
            info!("  • Strong compute performance");
            info!("  • Good async compute capabilities");
            info!("  • Ray tracing available on RDNA 2+ (RX 6000 series+)");
        }
        GpuVendor::Intel => {
            info!("  Intel GPU detected:");
            info!("  • Focus on power efficiency");
            info!("  • Good for integrated graphics");
            info!("  • Arc series offers competitive discrete performance");
            info!("  • Newer drivers continuing to improve performance");
        }
        GpuVendor::Apple => {
            info!("  Apple Silicon GPU detected:");
            info!("  • Unified memory architecture");
            info!("  • Excellent power efficiency");
            info!("  • Strong Metal support (via MoltenVK for Vulkan)");
            info!("  • Consider Metal API for best performance");
        }
        GpuVendor::Arm | GpuVendor::Qualcomm => {
            info!("  Mobile GPU detected:");
            info!("  • Tile-based rendering architecture");
            info!("  • Focus on power efficiency");
            info!("  • Avoid bandwidth-heavy operations");
            info!("  • Use mobile-optimized rendering techniques");
        }
        GpuVendor::Unknown => {
            info!("  Unknown GPU vendor");
            info!("  • Using conservative settings");
            info!("  • Profile performance to fine-tune");
        }
    }
    info!("");

    // Compare all presets
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("QUALITY PRESET COMPARISON");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");

    let presets = [
        QualityPreset::Low,
        QualityPreset::Medium,
        QualityPreset::High,
        QualityPreset::Ultra,
    ];

    for preset in presets {
        let cfg = HardwareTierConfig::from_preset(preset);
        let is_recommended = preset == detector.recommended_preset();
        
        info!(
            "{:8} {}",
            preset.name(),
            if is_recommended { "← RECOMMENDED" } else { "" }
        );
        info!("  Shadows: {}x{}", cfg.shadow_resolution, cfg.shadow_resolution);
        info!("  MSAA: {}x", cfg.msaa_samples);
        info!("  Post-FX: {}", if cfg.enable_post_processing { "Yes" } else { "No" });
        info!("  SSAO: {}", if cfg.enable_ssao { "Yes" } else { "No" });
        info!("  SSR: {}", if cfg.enable_ssr { "Yes" } else { "No" });
        info!("  Target: {:.0} FPS", cfg.target_fps);
        info!("");
    }

    // Set up a simple scene to demonstrate the detector working
    let mesh_id = engine
        .render_context_mut()
        .mesh_manager_mut()
        .load_mesh("cube", colored_cube_mesh())?;

    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("Rendering demonstration scene...");
    info!("Press ESC to exit");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");

    // Run the engine with a rotating cube
    let mut rotation = 0.0f32;

    engine.run(move |_engine, dt| {
        rotation += dt * 0.5;

        let transform = Mat4::from_rotation_y(rotation) * Mat4::from_translation(Vec3::new(0.0, 0.0, -5.0));

        let commands = RenderCommands {
            view_matrix: Mat4::look_at_rh(
                Vec3::new(0.0, 2.0, 5.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ),
            projection_matrix: Mat4::perspective_rh(
                std::f32::consts::PI / 4.0,
                1280.0 / 720.0,
                0.1,
                100.0,
            ),
            draw_commands: vec![DrawCommand {
                mesh_id: mesh_id.clone(),
                transform,
                texture_id: None,
                material_properties: None,
            }],
            directional_lights: vec![],
            point_lights: vec![],
        };

        Ok(commands)
    })
}
